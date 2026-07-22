import { render, screen, fireEvent } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import CompletedView from './completedView.svelte';

vi.mock('./tauri', () => ({ invoke: vi.fn(), listen: vi.fn() }));

describe('CompletedView', () => {
  beforeEach(() => vi.clearAllMocks());
  it('shows empty hint', async () => {
    const { invoke } = await import('./tauri');
    (invoke as any).mockResolvedValue([]);
    render(CompletedView);
    expect(await screen.findByText('暂无已完成项')).toBeTruthy();
  });
  it('renders rows and fires reactivate', async () => {
    const { invoke } = await import('./tauri');
    (invoke as any).mockResolvedValue([
      { id: 'a', content: '旧任务', color: 'pink', x: 0, y: 0, w: 0, h: 0, snooze_minutes: 2, created_at: '', completed_at: 'x', is_hidden: false, hidden_until: null },
    ]);
    render(CompletedView);
    await screen.findByText('旧任务');
    await fireEvent.click(screen.getByText('重新激活'));
    expect((invoke as any).mock.calls.some((c: any) => c[0] === 'reactivate')).toBe(true);
  });
});
