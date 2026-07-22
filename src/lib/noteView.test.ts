import { render, screen, fireEvent } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import NoteView from './noteView.svelte';

vi.mock('./tauri', () => ({
  invoke: vi.fn(),
  listen: vi.fn(),
}));

describe('NoteView', () => {
  beforeEach(() => vi.clearAllMocks());
  it('renders note and fires hide/complete', async () => {
    const { invoke } = await import('./tauri');
    (invoke as any).mockResolvedValue({
      id: 'n1', content: '提交季度报告', color: 'yellow',
      x: 0, y: 0, w: 240, h: 170, snooze_minutes: 2, created_at: '',
      completed_at: null, is_hidden: false, hidden_until: null,
    });
    render(NoteView, { props: { id: 'n1' } });
    expect(await screen.findByText('提交季度报告')).toBeTruthy();
    await fireEvent.click(screen.getByText('隐藏'));
    await fireEvent.click(screen.getByText('✓ 完成'));
    expect((invoke as any).mock.calls.map((c: any) => c[0])).toContain('hide_note');
    expect((invoke as any).mock.calls.map((c: any) => c[0])).toContain('complete_note');
  });
});
