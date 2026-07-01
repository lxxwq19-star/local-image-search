#!/usr/bin/env python3
"""
Persistent CLIP/EVA-CLIP encoding server.
Loads models ONCE, handles encode requests via TCP JSON protocol.

MODEL_VARIANT env var controls which model to load:
  MODEL_VARIANT=clip    → CLIP ViT-B/32 (ONNX, CPU) — default
  MODEL_VARIANT=eva02  → EVA02-CLIP-L-14-336 (PyTorch, GPU if available)

CLIP_MODELS_DIR env var overrides the base directory for model files.
  If set, models are loaded from {CLIP_MODELS_DIR}/siglip2-large/ and
  {CLIP_MODELS_DIR}/clip-large/ instead of {script_dir}/models/.

Protocol (line-delimited JSON):
  Request:  {"id": 1, "type": "encode_images", "paths": ["D:/1.jpg"]}
  Response: {"id": 1, "type": "encode_images_result", "results": [{"path": "...", "vector": [...]}]}

  Request:  {"id": 2, "type": "encode_text", "text": "a cat"}
  Response: {"id": 2, "type": "encode_text_result", "vector": [...]}

  Request:  {"id": 0, "type": "shutdown"}
  Response: {"id": 0, "type": "shutdown_ack"}

  Request:  {"id": 3, "type": "translate", "text": "中文"}
  Response: {"id": 3, "type": "translate_result", "translatedText": "chinese text"}

  Request:  {"id": 4, "type": "get_model_info"}
  Response: {"id": 4, "type": "model_info_result", "model": "eva02", "dim": 768, "img_size": 336, "gpu": true}
"""
import json
import sys
import os

# Fix: when launched from a GUI app (no console), sys.stdout/stderr can be None.
# Replace with a file-based logger so print() and tracebacks still work.
_log_dir = os.path.dirname(os.path.abspath(__file__))
_stdout_log = os.path.join(_log_dir, "clip_server.log")
_stderr_log = os.path.join(_log_dir, "clip_server_err.log")

if sys.stdout is None:
    try:
        sys.stdout = open(_stdout_log, "a", buffering=1, encoding="utf-8")
    except Exception:
        pass
if sys.stderr is None:
    try:
        sys.stderr = open(_stderr_log, "a", buffering=1, encoding="utf-8")
    except Exception:
        pass

import socket
import traceback
import threading
import time
import logging

# Force Winsock2 initialization before any CUDA/Torch imports.
# On some Windows systems, python.exe spawned without CREATE_NO_WINDOW
# has a race condition where Winsock fails during CUDA driver init.
# Pre-creating a socket forces WSAStartup() early.
try:
    _sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    _sock.close()
    del _sock
except Exception:
    pass  # ignore - this is just a pre-initialization attempt

import numpy as np
from PIL import Image

# ── File Logging ────────────────────────────────────────────────────────────
# Write logs to clip_server.log in the same directory as this script
_log_dir = os.path.dirname(os.path.abspath(__file__))
_log_file = os.path.join(_log_dir, "clip_server.log")

logging.basicConfig(
    level=logging.INFO,
    format='[%(asctime)s] %(message)s',
    datefmt='%Y-%m-%d %H:%M:%S',
    handlers=[
        logging.FileHandler(_log_file, encoding='utf-8'),
        logging.StreamHandler(sys.stdout),
    ]
)
_logger = logging.getLogger(__name__)

def log(msg):
    """Print to both stdout and log file."""
    print(msg, flush=True)
    _logger.info(msg)

# ── Model Selection ────────────────────────────────────────────────────────────
# Dual model setup:
#   SigLIP2  → image encoding (以图搜图, 1024维)
#   CLIP-L/14 → text encoding (语义搜图, 768维)
SERVER_HOST = '127.0.0.1'
SERVER_PORT = 8765

# Model base directory: can be overridden by CLIP_MODELS_DIR env var
# This allows Tauri to bundle models in Resources/models/ and tell clip_server where to find them
_script_dir = os.path.dirname(os.path.abspath(__file__))
MODELS_BASE_DIR = os.environ.get("CLIP_MODELS_DIR", os.path.join(_script_dir, "models"))

# ── Shared Constants (CLIP / EVA-CLIP use same normalization) ─────────────────
TEXT_MAX_LEN = 77
MEAN = np.array([0.48145466, 0.4578275, 0.40821073], dtype=np.float32)
STD  = np.array([0.26862954, 0.26130258, 0.27577711], dtype=np.float32)

# ── TTA (Test Time Augmentation) Config ─────────────────────────────────────
# 1 = disabled (single center crop, fastest)
# 5 = center + 4 corners (recommended, +3-5% accuracy)
# 10 = center + 4 corners + horizontal flip (best accuracy, ~2x slower)
TTA_CROPS = int(os.environ.get("TTA_CROPS", "5"))
TTA_FLIP  = os.environ.get("TTA_FLIP", "0") == "1"
print(f"[SERVER] TTA_CROPS={TTA_CROPS}, TTA_FLIP={TTA_FLIP}", flush=True)


# ════════════════════════════════════════════════════════════════════════
def get_best_device():
    """Select best available device: CUDA > MPS (Apple Silicon) > CPU."""
    import torch
    if torch.cuda.is_available():
        return "cuda"
    if hasattr(torch.backends, "mps") and torch.backends.mps.is_available():
        return "mps"
    return "cpu"


def get_dtype(device):
    """float16 on GPU devices (faster), float32 on CPU."""
    return torch.float16 if device in ("cuda", "mps") else torch.float32


# MODEL LOADING
# ════════════════════════════════════════════════════════════════════════

def load_clip_onnx():
    """Load CLIP ViT-B/32 via ONNX Runtime."""
    import onnxruntime as ort
    from tokenizers import Tokenizer

    model_dir = MODELS_BASE_DIR
    text_onnx   = os.path.join(model_dir, "clip_text.onnx")
    vision_onnx = os.path.join(model_dir, "clip_vision.onnx")
    tokenizer_path = os.path.join(model_dir, "tokenizer.json")

    if not os.path.exists(vision_onnx):
        raise FileNotFoundError(f"CLIP vision model not found: {vision_onnx}")
    if not os.path.exists(text_onnx):
        raise FileNotFoundError(f"CLIP text model not found: {text_onnx}")

    providers = ["CPUExecutionProvider"]
    text_session   = ort.InferenceSession(text_onnx, providers=providers)
    vision_session = ort.InferenceSession(vision_onnx, providers=providers)
    tokenizer      = Tokenizer.from_file(tokenizer_path)

    print(f"[CLIP] ✅ ONNX models loaded (IMG_SIZE=224)", flush=True)
    return {
        "type": "clip_onnx",
        "text_session": text_session,
        "vision_session": vision_session,
        "tokenizer": tokenizer,
        "img_size": 224,
        "dim": 512,
        "gpu": False,
    }


def load_eva02_pytorch():
    """Load EVA02-CLIP-L-14-336 via transformers/PyTorch."""
    try:
        import torch
        from transformers import CLIPModel, CLIPProcessor
    except ImportError as e:
        raise RuntimeError(
            f"PyTorch/transformers not installed. "
            f"Install with: pip install torch torchvision transformers accelerate\n"
            f"Error: {e}"
        )

    eva_dir = os.path.join(MODELS_BASE_DIR, "eva02-l14")
    if not os.path.isdir(eva_dir):
        raise FileNotFoundError(
            f"EVA02 model directory not found: {eva_dir}\n"
            f"Run: python download_eva02.py"
        )

    # Detect GPU
    device = get_best_device()
    dtype  = get_dtype(device)

    print(f"[EVA02] Loading from {eva_dir} ...", flush=True)
    print(f"[EVA02] Device: {device}, dtype: {dtype}", flush=True)

    model     = CLIPModel.from_pretrained(eva_dir, dtype=dtype)
    processor = CLIPProcessor.from_pretrained(eva_dir)

    model = model.to(device)
    model.eval()

    # Check if half precision worked
    if device == "cuda":
        vram_mb = torch.cuda.memory_allocated() / 1024 / 1024
        print(f"[EVA02] ✅ Model loaded on GPU ({vram_mb:.0f} MB VRAM used)", flush=True)
    else:
        print(f"[EVA02] ✅ Model loaded on CPU", flush=True)

    return {
        "type": "eva02_pytorch",
        "model": model,
        "processor": processor,
        "device": device,
        "dtype": dtype,
        "img_size": 336,
        "dim": 768,
        "gpu": device == "cuda",
    }

def load_clip_large_pytorch():
    """Load CLIP-ViT-Large-Patch14 via transformers/PyTorch."""
    try:
        import torch
        from transformers import CLIPModel, CLIPProcessor
    except ImportError as e:
        raise RuntimeError(
            f"PyTorch/transformers not installed. "
            f"Install with: pip install torch torchvision transformers accelerate\n"
            f"Error: {e}"
        )

    clip_dir = os.path.join(MODELS_BASE_DIR, "clip-large")
    if not os.path.isdir(clip_dir):
        raise FileNotFoundError(
            f"CLIP-L/14 model directory not found: {clip_dir}\n"
            f"Download from: https://hf-mirror.com/openai/clip-vit-large-patch14"
        )

    # Detect GPU
    device = get_best_device()
    dtype  = get_dtype(device)

    print(f"[CLIP-L] Loading from {clip_dir} ...", flush=True)
    print(f"[CLIP-L] Device: {device}, dtype: {dtype}", flush=True)

    model     = CLIPModel.from_pretrained(clip_dir, dtype=dtype)
    processor = CLIPProcessor.from_pretrained(clip_dir, backend="torchvision")

    model = model.to(device)
    model.eval()

    if device == "cuda":
        vram_mb = torch.cuda.memory_allocated() / 1024 / 1024
        print(f"[CLIP-L] ✅ Model loaded on GPU ({vram_mb:.0f} MB VRAM used)", flush=True)
    else:
        print(f"[CLIP-L] ✅ Model loaded on CPU", flush=True)

    return {
        "type": "clip_large_pytorch",
        "model": model,
        "processor": processor,
        "device": device,
        "dtype": dtype,
        "img_size": 224,
        "dim": 768,
        "gpu": device == "cuda",
    }


def load_siglip2_pytorch():
    """Load SigLIP2-Large-Patch16-256 via transformers/PyTorch."""
    try:
        import torch
        from transformers import AutoModel, AutoProcessor
    except ImportError as e:
        raise RuntimeError(
            f"PyTorch/transformers not installed. "
            f"Install with: pip install torch torchvision transformers accelerate\n"
            f"Error: {e}"
        )

    siglip_dir = os.path.join(MODELS_BASE_DIR, "siglip2-large")
    if not os.path.isdir(siglip_dir):
        raise FileNotFoundError(
            f"SigLIP2 model directory not found: {siglip_dir}\n"
            f"Download from: https://huggingface.co/google/siglip2-large-patch16-256"
        )

    # Detect GPU
    device = get_best_device()
    dtype  = get_dtype(device)

    print(f"[SigLIP2] Loading from {siglip_dir} ...", flush=True)
    print(f"[SigLIP2] Device: {device}, dtype: {dtype}", flush=True)

    model     = AutoModel.from_pretrained(siglip_dir, dtype=dtype)
    processor = AutoProcessor.from_pretrained(siglip_dir, backend="torchvision")

    model = model.to(device)
    model.eval()

    if device == "cuda":
        vram_mb = torch.cuda.memory_allocated() / 1024 / 1024
        print(f"[SigLIP2] ✅ Model loaded on GPU ({vram_mb:.0f} MB VRAM used)", flush=True)
    else:
        print(f"[SigLIP2] ✅ Model loaded on CPU", flush=True)

    return {
        "type": "siglip2_pytorch",
        "model": model,
        "processor": processor,
        "device": device,
        "dtype": dtype,
        "img_size": 256,
        "dim": 1024,  # SigLIP2-Large outputs 1024-dim
        "gpu": device == "cuda",
    }


# ── Load dual models at startup ──────────────────────────────────────────────────
_loader_log = []

def _load_model(loader_fn, name):
    """Load a model with timing and error reporting."""
    import time
    t0 = time.time()
    try:
        m = loader_fn()
        elapsed = time.time() - t0
        _loader_log.append(f"[{name}] ✅ Loaded in {elapsed:.1f}s — {m['type']} (dim={m['dim']}, gpu={m['gpu']})")
        return m
    except Exception as e:
        _loader_log.append(f"[{name}] ❌ Failed: {e}")
        return None

print(f"[SERVER] Loading dual models (SigLIP2 + CLIP-L/14) ...", flush=True)

# Pre-check torch/cuda to force any Winsock/Winsock2 init errors early
try:
    import torch
    _ = torch.cuda.is_available()
except Exception:
    pass

# Load models sequentially to avoid CUDA memory race conditions
print(f"[SERVER] Loading SigLIP2 model...", flush=True)
MODEL_SIGLIP = _load_model(load_siglip2_pytorch, "SigLIP2")

print(f"[SERVER] Loading CLIP-L/14 model...", flush=True)
MODEL_CLIP_L = _load_model(load_clip_large_pytorch, "CLIP-L/14")

# Fallback to ONNX for text if CLIP-L/14 failed
if MODEL_CLIP_L is None:
    print(f"[SERVER] CLIP-L/14 failed, falling back to ONNX CLIP for text encoding", flush=True)
    MODEL_CLIP_L = _load_model(load_clip_onnx, "CLIP-ONNX")

for line in _loader_log:
    print(f"  {line}", flush=True)

if MODEL_SIGLIP is None:
    print(f"[SERVER] ⚠️  Both models failed to load! Search will not work.", flush=True)


# ════════════════════════════════════════════════════════════════════════
# TTA PREPROCESSING
# ════════════════════════════════════════════════════════════════════════

def generate_crop_boxes(w, h, img_size, n_crops):
    """
    Generate crop boxes for TTA.
    n_crops=1  → center only
    n_crops=5  → center + 4 corners
    n_crops=10 → center + 4 corners + flip (handled outside)
    """
    if n_crops <= 1:
        # Center crop
        return [(0.5, 0.5)]

    # Resize so shorter side = img_size
    scale = img_size / min(w, h)
    new_w, new_h = int(w * scale), int(h * scale)

    boxes = []
    if n_crops >= 5:
        # center + 4 corners
        offsets = [(0.5, 0.5), (0.0, 0.0), (0.0, 1.0), (1.0, 0.0), (1.0, 1.0)]
        boxes = offsets[:min(n_crops, 5)]
    else:
        boxes = [(0.5, 0.5)]

    return boxes


def preprocess_crop(img, cx, cy, img_size):
    """Crop image at (cx, cy) normalized coordinate, resize to (img_size, img_size)."""
    w, h = img.size
    scale = img_size / min(w, h)
    new_w, new_h = int(w * scale), int(h * scale)
    img_r = img.resize((new_w, new_h), Image.Resampling.BICUBIC)

    left = int((new_w - img_size) * cx)
    top  = int((new_h - img_size) * cy)
    left = max(0, min(left, new_w - img_size))
    top  = max(0, min(top,  new_h - img_size))
    return img_r.crop((left, top, left + img_size, top + img_size))


def numpy_to_tensor(img, img_size):
    """Convert PIL image → normalized numpy array (C, H, W)."""
    arr = np.array(img, dtype=np.float32) / 255.0
    arr = (arr - MEAN) / STD
    arr = np.transpose(arr, (2, 0, 1))  # (C, H, W)
    return arr


# ════════════════════════════════════════════════════════════════════════
# ENCODE: TEXT (uses CLIP-L/14)
# ════════════════════════════════════════════════════════════════════════

def encode_text_clip_large(text: str) -> np.ndarray:
    """Encode text with CLIP-L/14 → normalized vector."""
    print(f"[ENCODE_TEXT] text={text[:60]!r}", flush=True)
    m = MODEL_CLIP_L
    if m is None:
        raise RuntimeError("CLIP-L/14 model not available")

    if m["type"] == "clip_onnx":
        from tokenizers import Tokenizer
        enc = m["tokenizer"].encode(text)
        ids = enc.ids[:TEXT_MAX_LEN] + [0] * (TEXT_MAX_LEN - len(enc.ids))
        out = m["text_session"].run(None, {"input_ids": np.array([ids], dtype=np.int64)})[0][0]
        vec = out[0] if len(out.shape) == 2 else out
    else:
        import torch
        inputs = m["processor"](text=text, return_tensors="pt", padding=True)
        inputs = {k: v.to(m["device"]) for k, v in inputs.items()}
        with torch.no_grad():
            # Use get_text_features() which returns a tensor directly
            text_emb = m["model"].get_text_features(**inputs)
            # Safe .float() - handle both tensor and output object
            if isinstance(text_emb, torch.Tensor):
                if m["dtype"] == torch.float16:
                    text_emb = text_emb.float()
                vec = text_emb[0].cpu().numpy()
            else:
                # Fallback: extract from output object
                if hasattr(text_emb, "text_embeds"):
                    t = text_emb.text_embeds
                elif hasattr(text_emb, "pooler_output"):
                    t = text_emb.pooler_output
                else:
                    t = text_emb.last_hidden_state[:, 0]
                if m["dtype"] == torch.float16:
                    t = t.float()
                vec = t[0].cpu().numpy()

    vec_n = l2_normalize(vec)
    print(f"[ENCODE_TEXT] vec_dim={len(vec_n)}, norm={np.linalg.norm(vec_n):.4f}", flush=True)
    return vec_n


# ════════════════════════════════════════════════════════════════════════
# ENCODE: IMAGE (uses SigLIP2 with TTA)
# ════════════════════════════════════════════════════════════════════════

def encode_image_siglip2(image_path: str) -> np.ndarray:
    """Encode single image with SigLIP2 → normalized 1024-dim vector (with TTA)."""
    m = MODEL_SIGLIP
    if m is None:
        raise RuntimeError("SigLIP2 model not available")

    import torch
    model     = m["model"]
    processor = m["processor"]
    device    = m["device"]
    img_size  = m["img_size"]

    try:
        img = Image.open(image_path).convert("RGB")
    except Exception as e:
        raise RuntimeError(f"Cannot open image {image_path}: {e}")

    w, h = img.size
    n_crops = min(TTA_CROPS, 5)
    boxes = generate_crop_boxes(w, h, img_size, n_crops)

    embs = []
    for (cx, cy) in boxes:
        crop = preprocess_crop(img, cx, cy, img_size)
        inputs = processor(images=crop, return_tensors="pt")
        inputs = {k: v.to(device) for k, v in inputs.items()}

        with torch.no_grad():
            # Use get_image_features() which returns a tensor directly
            img_emb = model.get_image_features(**inputs)
            # Safe .float() - handle both tensor and output object
            if isinstance(img_emb, torch.Tensor):
                if m["dtype"] == torch.float16:
                    img_emb = img_emb.float()
                vec = img_emb[0].cpu().numpy()
            else:
                # Fallback: extract from output object
                if hasattr(img_emb, "image_embeds"):
                    t = img_emb.image_embeds
                elif hasattr(img_emb, "pooler_output"):
                    t = img_emb.pooler_output
                else:
                    t = img_emb.last_hidden_state[:, 0]
                if m["dtype"] == torch.float16:
                    t = t.float()
                vec = t[0].cpu().numpy()
        embs.append(vec)  # already normalized by SigLIP2

        if TTA_FLIP:
            flip = crop.transpose(Image.FLIP_LEFT_RIGHT)
            inputs2 = processor(images=flip, return_tensors="pt")
            inputs2 = {k: v.to(device) for k, v in inputs2.items()}
            with torch.no_grad():
                img_emb2 = model.get_image_features(**inputs2)
                if isinstance(img_emb2, torch.Tensor):
                    if m["dtype"] == torch.float16:
                        img_emb2 = img_emb2.float()
                    vec2 = img_emb2[0].cpu().numpy()
                else:
                    if hasattr(img_emb2, "image_embeds"):
                        t2 = img_emb2.image_embeds
                    elif hasattr(img_emb2, "pooler_output"):
                        t2 = img_emb2.pooler_output
                    else:
                        t2 = img_emb2.last_hidden_state[:, 0]
                    if m["dtype"] == torch.float16:
                        t2 = t2.float()
                    vec2 = t2[0].cpu().numpy()
            embs.append(vec2)

    return l2_normalize(np.mean(embs, axis=0))


def encode_images_siglip2_batch(paths: list) -> list:
    """Encode multiple images with SigLIP2 using parallel preprocessing + GPU batch.
    Returns list of (path, vector) or (path, error_msg).
    Uses ThreadPoolExecutor to load and crop images in parallel,
    then runs ONE GPU forward pass for all TTA crops combined.
    """
    import torch
    from concurrent.futures import ThreadPoolExecutor, as_completed

    m = MODEL_SIGLIP
    if m is None:
        return [(p, None, "SigLIP2 not available") for p in paths]

    model     = m["model"]
    processor = m["processor"]
    device    = m["device"]
    img_size  = m["img_size"]
    n_crops = min(TTA_CROPS, 5)
    TTA_FLIP_ENABLED = TTA_FLIP

    def _preprocess_image(pidx: int, p: str):
        """Load and preprocess one image."""
        try:
            img = Image.open(p).convert("RGB")
            w, h = img.size
            boxes = generate_crop_boxes(w, h, img_size, n_crops)
            crops = []
            for cx, cy in boxes:
                crop = preprocess_crop(img, cx, cy, img_size)
                crops.append(crop)
                if TTA_FLIP_ENABLED:
                    crops.append(crop.transpose(Image.FLIP_LEFT_RIGHT))
            return (pidx, {"path": p, "crops": crops, "error": None})
        except Exception as e:
            return (pidx, {"path": p, "crops": [], "error": str(e)})

    # Step 1: Parallel image loading and TTA cropping
    n_workers = min(8, len(paths))
    img_meta = [None] * len(paths)
    with ThreadPoolExecutor(max_workers=n_workers) as executor:
        futures = [executor.submit(_preprocess_image, i, p) for i, p in enumerate(paths)]
        for f in as_completed(futures):
            pidx, meta = f.result()
            img_meta[pidx] = meta

    # Step 2: Collect all crops in index order
    all_crops = []
    for pidx, meta in enumerate(img_meta):
        if meta["error"]:
            continue
        for crop in meta["crops"]:
            all_crops.append((pidx, crop))

    if not all_crops:
        return [(meta["path"], None, meta["error"]) for meta in img_meta]

    # Step 3: GPU batch encoding
    all_pils = [c[1] for c in all_crops]
    try:
        batch_inputs = processor(images=all_pils, return_tensors="pt")
        batch_inputs = {k: v.to(device) for k, v in batch_inputs.items()}

        with torch.no_grad():
            # Use get_image_features() which returns a tensor directly
            batch_emb = model.get_image_features(**batch_inputs)
            # Safe .float() - handle both tensor and output object
            if isinstance(batch_emb, torch.Tensor):
                if m["dtype"] == torch.float16:
                    batch_emb = batch_emb.float()
                batch_vecs = batch_emb.cpu().numpy()
            else:
                # Fallback: extract from output object
                if hasattr(batch_emb, "image_embeds"):
                    b = batch_emb.image_embeds
                elif hasattr(batch_emb, "pooler_output"):
                    b = batch_emb.pooler_output
                else:
                    b = batch_emb.last_hidden_state[:, 0]
                if m["dtype"] == torch.float16:
                    b = b.float()
                batch_vecs = b.cpu().numpy()
    except Exception as e:
        import traceback
        traceback.print_exc()
        print(f"[BATCH] GPU batch failed ({e}), falling back...", flush=True)
        return [(p, encode_image_siglip2(p).tolist(), None)
                for p in paths if encode_image_siglip2(p) is not None]

    # Step 4: Split results per image, average TTA crops
    results = []
    vec_idx = 0
    for pidx, meta in enumerate(img_meta):
        if meta["error"]:
            results.append((meta["path"], None, meta["error"]))
            continue
        n = len(meta["crops"])
        if n == 0:
            results.append((meta["path"], None, "No crops"))
            continue
        img_vecs = batch_vecs[vec_idx:vec_idx + n]
        vec_idx += n
        avg = l2_normalize(np.mean(img_vecs, axis=0))
        results.append((meta["path"], avg.tolist(), None))

    return results


# ============================================================
# BATCH ENCODING (for indexing — generates both vectors)
# ============================================================

def encode_both_batch(paths: list) -> list:
    """Encode images with BOTH models in GPU batches.
    Returns list of dicts with siglip2_vector, cliplarge_vector, error.
    Uses GPU batch processing for SigLIP2 + CLIP-L/14.
    """
    total = len(paths)
    print(f"[BATCH] Dual encoding {total} images (GPU + parallel I/O)...", flush=True)

    # SigLIP2 batch encoding (parallel I/O + GPU batch)
    siglip2_results = encode_images_siglip2_batch(paths)
    siglip2_map = {r[0]: r for r in siglip2_results}

    # CLIP-L/14 image encoding (batched)
    m = MODEL_CLIP_L
    cliplarge_map = {}
    if m and m["type"] != "clip_onnx":
        import torch
        from concurrent.futures import ThreadPoolExecutor, as_completed
        model = m["model"]
        processor = m["processor"]
        device = m["device"]

        def _try_open(p):
            try:
                img = Image.open(p).convert("RGB")
                return (p, img, None)
            except Exception as e:
                return (p, None, str(e))

        # Parallel image loading
        valid_imgs = []
        with ThreadPoolExecutor(max_workers=min(8, len(paths))) as ex:
            futures = {ex.submit(_try_open, p): p for p in paths}
            for f in as_completed(futures):
                p, img, err = f.result()
                if img is not None:
                    valid_imgs.append((p, img))
                else:
                    cliplarge_map[p] = (p, None, err)

        if valid_imgs:
            try:
                batch_pils = [img for _, img in valid_imgs]
                inputs = processor(images=batch_pils, return_tensors="pt")
                inputs = {k: v.to(device) for k, v in inputs.items()}
                with torch.no_grad():
                    # Use get_image_features() which returns a tensor directly
                    embs = model.get_image_features(**inputs)
                    # Safe .float() - handle both tensor and output object
                    if isinstance(embs, torch.Tensor):
                        if m["dtype"] == torch.float16:
                            embs = embs.float()
                        vecs = embs.cpu().numpy()
                    else:
                        # Fallback: extract from output object
                        if hasattr(embs, "image_embeds"):
                            b = embs.image_embeds
                        elif hasattr(embs, "pooler_output"):
                            b = embs.pooler_output
                        else:
                            b = embs.last_hidden_state[:, 0]
                        if m["dtype"] == torch.float16:
                            b = b.float()
                        vecs = b.cpu().numpy()
                    for i, (p, _) in enumerate(valid_imgs):
                        cliplarge_map[p] = (p, l2_normalize(vecs[i]).tolist(), None)
            except Exception as e:
                print(f"[BATCH] CLIP-L batch failed ({e}), fallback sequential...", flush=True)
                for p, img in valid_imgs:
                    try:
                        inp = processor(images=img, return_tensors="pt")
                        inp = {k: v.to(device) for k, v in inp.items()}
                        with torch.no_grad():
                            emb = model.get_image_features(**inp)
                            if isinstance(emb, torch.Tensor):
                                if m["dtype"] == torch.float16:
                                    emb = emb.float()
                                cliplarge_map[p] = (p, l2_normalize(emb[0].cpu().numpy()).tolist(), None)
                            else:
                                if hasattr(emb, "image_embeds"):
                                    t = emb.image_embeds
                                elif hasattr(emb, "pooler_output"):
                                    t = emb.pooler_output
                                else:
                                    t = emb.last_hidden_state[:, 0]
                                if m["dtype"] == torch.float16:
                                    t = t.float()
                                cliplarge_map[p] = (p, l2_normalize(t[0].cpu().numpy()).tolist(), None)
                    except Exception as e2:
                        cliplarge_map[p] = (p, None, str(e2))

    # Merge results
    results = []
    for p in paths:
        s_res = siglip2_map.get(p, (p, None, "Missing"))
        c_res = cliplarge_map.get(p, (p, None, None))
        err = s_res[2] or c_res[2]
        results.append({"path": p, "siglip2_vector": s_res[1], "cliplarge_vector": c_res[1], "error": err})

    print(f"[BATCH] Done: {total} images", flush=True)
    return results


def encode_image(image_path: str) -> np.ndarray:
    """Backward compat: alias for encode_image_siglip2."""
    return encode_image_siglip2(image_path)


def encode_text(text: str) -> np.ndarray:
    """Backward compat: alias for encode_text_clip_large."""
    return encode_text_clip_large(text)


def encode_images_batch(paths: list) -> list:
    """Backward compat: batch encode with SigLIP2 only."""
    results = []
    for p in paths:
        try:
            vec = encode_image_siglip2(p)
            results.append({"path": p, "vector": vec.tolist()})
        except Exception as e:
            results.append({"path": p, "error": str(e)})
    return results

# ════════════════════════════════════════════════════════════════════════
# L2 NORMALIZE
# ════════════════════════════════════════════════════════════════════════

def l2_normalize(v):
    """L2 normalize a vector."""
    norm = np.linalg.norm(v)
    return v / norm if norm > 0 else v


# ════════════════════════════════════════════════════════════════════════
# TRANSLATE
# ════════════════════════════════════════════════════════════════════════

def translate_text(text):
    """Translate Chinese to English using dictionary + MyMemory API."""
    if not has_cjk(text):
        return text
    # Try dictionary first
    d = {"搜索": "search", "图片": "image", "照片": "photo", "风景": "landscape",
         "人物": "person", "动物": "animal", "猫": "cat", "狗": "dog",
         "汽车": "car", "房子": "house", "树": "tree", "花": "flower",
         "天空": "sky", "大海": "sea", "山": "mountain", "食物": "food",
         "红色": "red", "蓝色": "blue", "绿色": "green", "黄色": "yellow",
         "白色": "white", "黑色": "black", "大": "big", "小": "small"}
    for cn, en in d.items():
        text = text.replace(cn, en)
    if not has_cjk(text):
        return text
    # MyMemory API
    try:
        import urllib.request, urllib.parse
        params = urllib.parse.urlencode({"q": text, "langpair": "zh|en"})
        url = f"https://api.mymemory.translated.net/get?{params}"
        req = urllib.request.Request(url, headers={"User-Agent": "Mozilla/5.0"})
        with urllib.request.urlopen(req, timeout=3) as r:
            data = json.loads(r.read().decode("utf-8"))
            if data.get("responseStatus") == 200:
                return data["responseData"]["translatedText"]
    except Exception:
        pass
    return text


def has_cjk(s):
    """Check if string contains CJK characters."""
    import unicodedata
    for c in s:
        if 'CJK' in unicodedata.category(c) or '\u4e00' <= c <= '\u9fff':
            return True
    return False


# ════════════════════════════════════════════════════════════════════════
# MODEL INFO
# ════════════════════════════════════════════════════════════════════════

def get_model_info():
    """Return current model info for both models."""
    info = {"mode": "dual", "tta_crops": TTA_CROPS, "tta_flip": TTA_FLIP}
    if MODEL_SIGLIP:
        info["siglip2"] = {"type": MODEL_SIGLIP["type"], "dim": MODEL_SIGLIP["dim"],
                           "img_size": MODEL_SIGLIP["img_size"], "gpu": MODEL_SIGLIP["gpu"]}
    else:
        info["siglip2"] = None
    if MODEL_CLIP_L:
        info["cliplarge"] = {"type": MODEL_CLIP_L["type"], "dim": MODEL_CLIP_L["dim"],
                             "img_size": MODEL_CLIP_L["img_size"], "gpu": MODEL_CLIP_L["gpu"]}
    else:
        info["cliplarge"] = None
    return info


# ════════════════════════════════════════════════════════════════════════
# TCP SERVER
# ════════════════════════════════════════════════════════════════════════

def handle_client(conn, addr):
    """Handle one TCP client connection."""
    buf = ""
    try:
        while True:
            data = conn.recv(4096)
            if not data:
                break
            buf += data.decode("utf-8", errors="replace")
            while "\n" in buf:
                line, buf = buf.split("\n", 1)
                line = line.strip()
                if not line or not line.startswith("{"):
                    continue
                try:
                    req = json.loads(line)
                except json.JSONDecodeError:
                    continue
                req_id = req.get("id", 0)
                req_type = req.get("type", "")
                try:
                    if req_type in ("encode_text", "encode_text_clip_large"):
                        text = req.get("text", "")
                        text = translate_text(text)
                        vec = encode_text_clip_large(text)
                        resp = {"id": req_id, "type": "encode_text_result", "vector": vec.tolist()}
                    elif req_type == "encode_images":
                        paths = req.get("paths", [])
                        results = encode_images_batch(paths)
                        resp = {"id": req_id, "type": "encode_images_result", "results": results}
                    elif req_type == "encode_image_siglip2":
                        path = req.get("path", "")
                        if not path:
                            paths = req.get("paths", [])
                            path = paths[0] if paths else ""
                        vec = encode_image_siglip2(path)
                        resp = {"id": req_id, "type": "encode_image_siglip2_result", "vector": vec.tolist()}
                    elif req_type == "encode_both":
                        paths = req.get("paths", [])
                        results = encode_both_batch(paths)
                        resp = {"id": req_id, "type": "encode_both_result", "results": results}
                    elif req_type == "translate":
                        text = req.get("text", "")
                        translated = translate_text(text)
                        resp = {"id": req_id, "type": "translate_result", "translatedText": translated}
                    elif req_type == "get_model_info":
                        info = get_model_info()
                        resp = {"id": req_id, "type": "model_info_result", **info}
                    elif req_type == "shutdown":
                        resp = {"id": 0, "type": "shutdown_ack"}
                        conn.sendall((json.dumps(resp) + "\n").encode("utf-8"))
                        print("[TCP] Shutdown requested.", flush=True)
                        os._exit(0)
                    else:
                        resp = {"id": req_id, "type": "error", "error": "Unknown: " + req_type}
                except Exception as e:
                    import traceback
                    tb = traceback.format_exc()
                    print(f"[TCP] Error {req_type}: {e}\n{tb}", flush=True)
                    resp = {"id": req_id, "type": "error", "error": str(e)}
                try:
                    conn.sendall((json.dumps(resp) + "\n").encode("utf-8"))
                except Exception:
                    break
    except Exception:
        pass
    finally:
        conn.close()


def main():
    serversocket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    serversocket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    serversocket.bind((SERVER_HOST, SERVER_PORT))
    serversocket.listen(5)
    serversocket.settimeout(30)
    print(f"[SERVER] TCP server listening on {SERVER_HOST}:{SERVER_PORT}", flush=True)
    print(f"[SERVER] TTA_CROPS={TTA_CROPS}, TTA_FLIP={TTA_FLIP}", flush=True)
    while True:
        try:
            conn, addr = serversocket.accept()
            print(f"[TCP] Client connected: {addr}", flush=True)
            threading.Thread(target=handle_client, args=(conn, addr), daemon=True).start()
        except socket.timeout:
            continue
        except Exception:
            break

if __name__ == "__main__":
    # Startup diagnostic: print torch/CUDA/MPS status
    try:
        import torch
        print(f"[STARTUP] torch version: {torch.__version__}", flush=True)
        # CUDA
        cuda_ok = torch.cuda.is_available()
        print(f"[STARTUP] CUDA available: {cuda_ok}", flush=True)
        if cuda_ok:
            print(f"[STARTUP] CUDA version: {torch.version.cuda}", flush=True)
            print(f"[STARTUP] GPU: {torch.cuda.get_device_name(0)}", flush=True)
            print(f"[STARTUP] GPU memory: {torch.cuda.get_device_properties(0).total_memory / 1024**3:.1f} GB", flush=True)
        # MPS (Apple Silicon Mac)
        mps_ok = hasattr(torch.backends, "mps") and torch.backends.mps.is_available()
        print(f"[STARTUP] MPS available: {mps_ok}", flush=True)
        if mps_ok:
            print(f"[STARTUP] MPS: Apple Silicon GPU enabled (float16)", flush=True)
        if not cuda_ok and not mps_ok:
            print(f"[STARTUP] *** CPU only - models will run slower ***", flush=True)
    except Exception as e:
        print(f"[STARTUP] Diagnostic error: {e}", flush=True)

    main()
