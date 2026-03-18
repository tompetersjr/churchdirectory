import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Leadership } from "../types";

export const useLeadershipStore = defineStore("leadership", () => {
  const entries = ref<Leadership[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function fetchLeadership() {
    loading.value = true;
    error.value = null;
    try {
      entries.value = await invoke<Leadership[]>("get_leadership");
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function createEntry(ministry: string, names: string, sortOrder: number) {
    error.value = null;
    try {
      const entry = await invoke<Leadership>("create_leadership", {
        input: { ministry, names, sort_order: sortOrder },
      });
      entries.value.push(entry);
      return entry;
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  async function updateEntry(id: number, ministry: string, names: string, sortOrder: number) {
    error.value = null;
    try {
      const entry = await invoke<Leadership>("update_leadership", {
        id,
        input: { ministry, names, sort_order: sortOrder },
      });
      const idx = entries.value.findIndex((e) => e.id === id);
      if (idx !== -1) entries.value[idx] = entry;
      return entry;
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  async function deleteEntry(id: number) {
    error.value = null;
    try {
      await invoke("delete_leadership", { id });
      entries.value = entries.value.filter((e) => e.id !== id);
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  return {
    entries,
    loading,
    error,
    fetchLeadership,
    createEntry,
    updateEntry,
    deleteEntry,
  };
});
