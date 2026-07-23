import { render, screen, fireEvent } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import NoteView from './noteView.svelte';

vi.mock('./tauri', () => ({
  invoke: vi.fn(),
  listen: vi.fn(),
}));
vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: () => ({ startDragging: vi.fn() }),
}));

const NOTE = {
  id: 'n1', content: '提交季度报告', color: 'yellow',
  x: 0, y: 0, w: 240, h: 170, snooze_minutes: 2, created_at: '',
  completed_at: null, is_hidden: false, hidden_until: null,
};
const callsOf = (invoke: any): string[] => invoke.mock.calls.map((c: any) => c[0]);

describe('NoteView', () => {
  beforeEach(() => vi.clearAllMocks());
  afterEach(() => vi.useRealTimers());

  it('renders note, hides on click; 完成 defers complete_note and 撤销 cancels', async () => {
    const { invoke } = await import('./tauri');
    (invoke as any).mockResolvedValue({ ...NOTE });
    render(NoteView, { props: { id: 'n1' } });
    expect(await screen.findByDisplayValue('提交季度报告')).toBeTruthy();

    // 隐藏 still fires hide_note immediately.
    await fireEvent.click(screen.getByText('隐藏'));
    expect(callsOf(invoke as any)).toContain('hide_note');

    // 完成 no longer fires complete_note immediately — an undo affordance shows.
    await fireEvent.click(screen.getByText('完成'));
    expect(callsOf(invoke as any)).not.toContain('complete_note');
    expect(screen.getByText('撤销')).toBeTruthy();

    // 撤销 within the window cancels: complete_note still never fires.
    await fireEvent.click(screen.getByText('撤销'));
    expect(callsOf(invoke as any)).not.toContain('complete_note');
  });

  it('fires complete_note only after the 5s undo window elapses', async () => {
    vi.useFakeTimers();
    const { invoke } = await import('./tauri');
    (invoke as any).mockResolvedValue({ ...NOTE });
    render(NoteView, { props: { id: 'n1' } });
    await vi.waitFor(() => expect(screen.getByDisplayValue('提交季度报告')).toBeTruthy());

    await fireEvent.click(screen.getByText('完成'));
    expect(callsOf(invoke as any)).not.toContain('complete_note');

    await vi.advanceTimersByTimeAsync(5000);
    expect(callsOf(invoke as any)).toContain('complete_note');
  });
});
