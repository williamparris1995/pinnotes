import { render, screen, fireEvent } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import NoteView from './noteView.svelte';
import { setLocale } from './i18n.svelte';

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
  completed_at: null, is_hidden: false, hidden_until: null, markdown: false,
};
const callsOf = (invoke: any): string[] => invoke.mock.calls.map((c: any) => c[0]);

describe('NoteView', () => {
  beforeEach(() => {
    setLocale('zh');
    vi.clearAllMocks();
  });
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

  it('persists edited content via edit_note on focusout', async () => {
    const { invoke } = await import('./tauri');
    (invoke as any).mockResolvedValue({ ...NOTE });
    render(NoteView, { props: { id: 'n1' } });
    const ta = await screen.findByDisplayValue('提交季度报告');
    await fireEvent.input(ta, { target: { value: '已修改内容' } });
    await fireEvent.focusOut(ta);
    expect(callsOf(invoke as any)).toContain('edit_note');
  });

  it('changes color via set_color when a different color dot is clicked', async () => {
    const { invoke } = await import('./tauri');
    (invoke as any).mockResolvedValue({ ...NOTE }); // color = yellow
    render(NoteView, { props: { id: 'n1' } });
    await screen.findByDisplayValue('提交季度报告');
    await fireEvent.click(screen.getByLabelText('颜色：粉'));
    expect(callsOf(invoke as any)).toContain('set_color');
  });

  it('cycles snooze minutes via set_snooze', async () => {
    const { invoke } = await import('./tauri');
    (invoke as any).mockResolvedValue({ ...NOTE }); // snooze_minutes = 2
    render(NoteView, { props: { id: 'n1' } });
    const btn = await screen.findByText('2分');
    await fireEvent.click(btn);
    expect(callsOf(invoke as any)).toContain('set_snooze');
    expect(await screen.findByText('5分')).toBeTruthy();
  });

  it('toggles markdown via set_markdown when M↓ clicked', async () => {
    const { invoke } = await import('./tauri');
    (invoke as any).mockResolvedValue({ ...NOTE }); // markdown false
    render(NoteView, { props: { id: 'n1' } });
    await screen.findByDisplayValue('提交季度报告');
    await fireEvent.click(screen.getByLabelText('Markdown 格式'));
    expect(callsOf(invoke as any)).toContain('set_markdown');
  });

  it('renders sanitized HTML (no textarea) when markdown on with content', async () => {
    const { invoke } = await import('./tauri');
    (invoke as any).mockResolvedValue({ ...NOTE, markdown: true, content: '**加粗**' });
    render(NoteView, { props: { id: 'n1' } });
    // 渲染态:无 <textarea>(源码态),有渲染的 <strong>加粗</strong>
    await vi.waitFor(() => expect(document.querySelector('textarea')).toBeNull());
    expect(await screen.findByText('加粗')).toBeTruthy();
  });

  it('enters edit mode showing source when rendered body clicked', async () => {
    const { invoke } = await import('./tauri');
    (invoke as any).mockResolvedValue({ ...NOTE, markdown: true, content: '**加粗**' });
    render(NoteView, { props: { id: 'n1' } });
    await vi.waitFor(() => expect(document.querySelector('textarea')).toBeNull());
    await fireEvent.click(await screen.findByText('加粗'));
    // 点渲染态 → 进编辑:textarea 出现,显示源码 **加粗**
    expect(await screen.findByDisplayValue('**加粗**')).toBeTruthy();
  });

  it('shows loadError when get_note rejects', async () => {
    const { invoke } = await import('./tauri');
    (invoke as any).mockRejectedValue(new Error('no note'));
    render(NoteView, { props: { id: 'missing' } });
    expect(await screen.findByText('无法加载便签')).toBeTruthy();
  });
});
