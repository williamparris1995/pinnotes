import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
export { invoke, listen };

export interface Note {
  id: string; content: string; color: string;
  x: number; y: number; w: number; h: number;
  snooze_minutes: number; created_at: string;
  completed_at: string | null; is_hidden: boolean; hidden_until: string | null;
}
