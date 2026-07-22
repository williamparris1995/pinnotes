import { render, screen, fireEvent } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import SettingsView from './settingsView.svelte';

vi.mock('./tauri', () => ({ invoke: vi.fn(), listen: vi.fn() }));

describe('SettingsView', () => {
  beforeEach(() => vi.clearAllMocks());
  it('loads settings on mount and fires set_settings / set_autostart', async () => {
    const { invoke } = await import('./tauri');
    (invoke as any).mockImplementation((cmd: string) =>
      cmd === 'get_autostart'
        ? Promise.resolve(true)
        : Promise.resolve({ default_snooze_minutes: '2' }),
    );
    render(SettingsView);
    // wait for onMount to finish loading settings
    await screen.findByText('5 分钟');

    await fireEvent.click(screen.getByText('5 分钟'));
    const setSettingsCall = (invoke as any).mock.calls.find((c: any) => c[0] === 'set_settings');
    expect(setSettingsCall).toBeTruthy();
    expect(setSettingsCall[1]).toEqual({ key: 'default_snooze_minutes', value: '5' });

    await fireEvent.click(screen.getByLabelText('开机自启'));
    const setAutostartCall = (invoke as any).mock.calls.find((c: any) => c[0] === 'set_autostart');
    expect(setAutostartCall).toBeTruthy();
    expect(setAutostartCall[1]).toEqual({ enabled: false });
  });
});
