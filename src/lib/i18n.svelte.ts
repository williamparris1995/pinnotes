// 轻量自研 i18n(Svelte 5 runes)。模块级响应式状态用 const 对象(不能
// `export let locale = $state` —— Svelte 5 报 state_invalid_export)。
// i18n.locale 是可变字段,在模板里读取即响应式。默认 'en';initLocale 从 settings
// 读 'language'(缺省按 first_run_done 判定老用户→zh / 新用户→en)。
import { invoke } from './tauri';
import { en } from './locales/en';
import { zh } from './locales/zh';

export type Locale = 'en' | 'zh';
const dicts: Record<Locale, Record<string, string>> = { en, zh };

export const i18n = $state<{ locale: Locale }>({ locale: 'en' });

/** 翻译键 → 当前语言串;回退 en;再回退键本身。{var} 占位符替换。 */
export function t(key: string, vars?: Record<string, string | number>): string {
  let s = dicts[i18n.locale][key] ?? en[key] ?? key;
  if (vars) {
    for (const k in vars) {
      s = s.split(`{${k}}`).join(String(vars[k]));
    }
  }
  return s;
}

/** 从 settings 载入语言:language 命中用它;否则老用户(first_run_done)→ zh,新用户 → en。 */
export async function initLocale(): Promise<void> {
  try {
    const s = await invoke<Record<string, string>>('get_settings');
    const lang = s.language;
    if (lang === 'en' || lang === 'zh') {
      i18n.locale = lang;
    } else {
      i18n.locale = s.first_run_done === '1' ? 'zh' : 'en';
    }
  } catch {
    /* 保持默认 en */
  }
}

/** 切语言并持久化到 settings.language。 */
export async function setLocalePersist(l: Locale): Promise<void> {
  i18n.locale = l;
  try {
    await invoke('set_settings', { key: 'language', value: l });
  } catch {
    /* ignore */
  }
}

/** 仅切内存 locale(不持久化)——测试用。 */
export function setLocale(l: Locale): void {
  i18n.locale = l;
}
