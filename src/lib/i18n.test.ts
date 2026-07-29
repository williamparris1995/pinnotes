import { describe, it, expect, vi, beforeEach } from 'vitest';

// 精确控制字典,以测试三级回退 (当前 locale → en → 键本身)。
vi.mock('./locales/en', () => ({
  en: { shared: 'EN', 'en.only': 'EN-only', greet: 'Hi {name}, {n} new' },
}));
vi.mock('./locales/zh', () => ({
  zh: { shared: '中', 'zh.only': '中-only' },
}));
vi.mock('./tauri', () => ({ invoke: vi.fn(), listen: vi.fn() }));

import { t, initLocale, setLocalePersist, setLocale, i18n } from './i18n.svelte';

describe('i18n.t()', () => {
  beforeEach(() => {
    setLocale('en');
    vi.clearAllMocks();
  });

  it('returns the current-locale string when the key exists', () => {
    setLocale('zh');
    expect(t('shared')).toBe('中');
  });

  it('falls back to en when the key is missing in the current locale', () => {
    setLocale('zh'); // zh 没有 'en.only'
    expect(t('en.only')).toBe('EN-only');
  });

  it('falls back to the key itself when missing everywhere', () => {
    expect(t('does.not.exist')).toBe('does.not.exist');
  });

  it('substitutes {var} placeholders', () => {
    setLocale('en');
    expect(t('greet', { name: 'Ada', n: 3 })).toBe('Hi Ada, 3 new');
  });
});

describe('i18n.initLocale()', () => {
  beforeEach(() => {
    setLocale('en');
    vi.clearAllMocks();
  });

  it('uses settings.language when it is a valid locale', async () => {
    const { invoke } = await import('./tauri');
    (invoke as any).mockResolvedValue({ language: 'zh' });
    await initLocale();
    expect(i18n.locale).toBe('zh');
  });

  it('defaults to zh for existing users (first_run_done=1) when language is invalid', async () => {
    const { invoke } = await import('./tauri');
    (invoke as any).mockResolvedValue({ language: '??', first_run_done: '1' });
    await initLocale();
    expect(i18n.locale).toBe('zh');
  });

  it('defaults to en for new users when language is invalid', async () => {
    const { invoke } = await import('./tauri');
    (invoke as any).mockResolvedValue({ language: '??', first_run_done: '0' });
    await initLocale();
    expect(i18n.locale).toBe('en');
  });

  it('leaves locale unchanged when invoke throws', async () => {
    const { invoke } = await import('./tauri');
    (invoke as any).mockRejectedValue(new Error('boom'));
    setLocale('zh'); // 预置非默认值,验证异常不会改动它
    await initLocale();
    expect(i18n.locale).toBe('zh');
  });
});

describe('i18n.setLocalePersist()', () => {
  beforeEach(() => {
    setLocale('en');
    vi.clearAllMocks();
  });

  it('switches locale and persists to settings.language', async () => {
    const { invoke } = await import('./tauri');
    (invoke as any).mockResolvedValue(undefined);
    await setLocalePersist('zh');
    expect(i18n.locale).toBe('zh');
    const setLangCall = (invoke as any).mock.calls.find(
      (c: any) => c[0] === 'set_settings' && c[1]?.key === 'language',
    );
    expect(setLangCall).toBeTruthy();
    expect(setLangCall[1]).toEqual({ key: 'language', value: 'zh' });
  });
});
