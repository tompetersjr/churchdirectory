import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Settings, Theme } from "../types";

const defaultSettings: Settings = {
  church_name: "Our Church",
  church_logo_path: undefined,
  theme: "system",
  default_layout: "grid",
  page_size: "letter",
  include_photos: true,
  include_contact_info: true,
  include_address: true,
};

function applyTheme(theme: Theme) {
  const root = document.documentElement;

  if (theme === "dark") {
    root.classList.add("dark");
  } else if (theme === "light") {
    root.classList.remove("dark");
  } else {
    // System preference
    const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
    if (prefersDark) {
      root.classList.add("dark");
    } else {
      root.classList.remove("dark");
    }
  }
}

export const useSettingsStore = defineStore("settings", () => {
  const settings = ref<Settings>({ ...defaultSettings });
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function fetchSettings() {
    loading.value = true;
    error.value = null;
    try {
      const loaded = await invoke<Settings | null>("get_settings");
      if (loaded) {
        settings.value = { ...defaultSettings, ...loaded };
      }
      applyTheme(settings.value.theme);
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function saveSettings(newSettings: Partial<Settings>) {
    loading.value = true;
    error.value = null;
    try {
      const updated = { ...settings.value, ...newSettings };
      await invoke("save_settings", { settings: updated });
      settings.value = updated;
      applyTheme(updated.theme);
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function setChurchLogo(filePath: string) {
    loading.value = true;
    error.value = null;
    try {
      const savedPath = await invoke<string>("save_church_logo", { filePath });
      settings.value.church_logo_path = savedPath;
      await saveSettings(settings.value);
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  function resetToDefaults() {
    settings.value = { ...defaultSettings };
  }

  function initTheme() {
    applyTheme(settings.value.theme);
    // Listen for system preference changes
    window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change", () => {
      if (settings.value.theme === "system") {
        applyTheme("system");
      }
    });
  }

  return {
    settings,
    loading,
    error,
    fetchSettings,
    saveSettings,
    setChurchLogo,
    resetToDefaults,
    initTheme,
  };
});
