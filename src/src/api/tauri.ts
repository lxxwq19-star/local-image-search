import { invoke } from '@tauri-apps/api/core';

// Types
export interface ImageResult {
  id: number;
  path: string;
  similarity: number;
  width?: number;
  height?: number;
}

export interface SearchResponse {
  query?: string;
  query_image?: string;
  count: number;
  images: ImageResult[];
}

export interface IndexStatus {
  indexed_count: number;
  index_size: number;
  status: string;
}

// API functions
export async function searchByText(query: string, topK = 50): Promise<SearchResponse> {
  return invoke('search_by_text', { query, top_k: topK });
}

export async function searchByImage(imagePath: string, topK = 50): Promise<SearchResponse> {
  return invoke('search_by_image', { image_path: imagePath, top_k: topK });
}

export async function indexImages(directory: string): Promise<string> {
  return invoke('index_images', { directory });
}

export async function getIndexStatus(): Promise<IndexStatus> {
  return invoke('get_index_status');
}

// Helper: convert file path to URL (file:// or app-data://)
export function pathToUrl(path: string): string {
  if (path.startsWith('http://') || path.startsWith('https://') || path.startsWith('data:')) {
    return path;
  }
  // Use file:// protocol
  return `file://${path}`;
}
