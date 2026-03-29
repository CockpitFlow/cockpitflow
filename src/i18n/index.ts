import en from './en.json';
import ro from './ro.json';

const translations: Record<string, any> = { en, ro };

let currentLang = $state(localStorage.getItem('scb-lang') || 'en');

export function setLanguage(lang: string) {
  currentLang = lang;
  localStorage.setItem('scb-lang', lang);
}

export function getLanguage(): string {
  return currentLang;
}

/**
 * Translate a key like "nav.dashboard" to the current language.
 * Falls back to English if key not found.
 * Supports {variable} interpolation.
 */
export function t(key: string, params?: Record<string, string | number>): string {
  const keys = key.split('.');
  let value: any = translations[currentLang];

  for (const k of keys) {
    if (value && typeof value === 'object' && k in value) {
      value = value[k];
    } else {
      // Fallback to English
      value = translations['en'];
      for (const fk of keys) {
        if (value && typeof value === 'object' && fk in value) {
          value = value[fk];
        } else {
          return key; // Key not found at all
        }
      }
      break;
    }
  }

  if (typeof value !== 'string') return key;

  // Interpolate {params}
  if (params) {
    for (const [k, v] of Object.entries(params)) {
      value = value.replace(`{${k}}`, String(v));
    }
  }

  return value;
}

/** Available languages with display names */
export const availableLanguages = [
  { code: 'en', name: 'English' },
  { code: 'ro', name: 'Română' },
  { code: 'de', name: 'Deutsch' },
  { code: 'fr', name: 'Français' },
  { code: 'es', name: 'Español' },
  { code: 'pt', name: 'Português' },
  { code: 'it', name: 'Italiano' },
  { code: 'pl', name: 'Polski' },
  { code: 'nl', name: 'Nederlands' },
  { code: 'ru', name: 'Русский' },
  { code: 'zh', name: '中文' },
  { code: 'ja', name: '日本語' },
  { code: 'ko', name: '한국어' },
];
