import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import en from "@/locales/en.json";
import zhCN from "@/locales/zh-CN.json";

const STORAGE_KEY = "varman.locale";

function detectLanguage(): string {
  const stored = window.localStorage.getItem(STORAGE_KEY);
  if (stored) {
    return stored;
  }

  const browser = window.navigator.language.toLowerCase();
  return browser.startsWith("zh") ? "zh-CN" : "en";
}

void i18n.use(initReactI18next).init({
  resources: {
    en: { translation: en },
    "zh-CN": { translation: zhCN },
  },
  lng: detectLanguage(),
  fallbackLng: "zh-CN",
  interpolation: {
    escapeValue: false,
  },
});

export type SupportedLocale = "en" | "zh-CN";

export function setLocale(locale: SupportedLocale) {
  window.localStorage.setItem(STORAGE_KEY, locale);
  void i18n.changeLanguage(locale);
}

export default i18n;
