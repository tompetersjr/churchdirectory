import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Staff } from "../types";

export const useStaffStore = defineStore("staff", () => {
  const entries = ref<Staff[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  const pastor = computed(() => entries.value.find((e) => e.role === "pastor"));
  const elders = computed(() => entries.value.filter((e) => e.role === "elder"));
  const staffMembers = computed(() => entries.value.filter((e) => e.role === "staff"));

  async function fetchStaff() {
    loading.value = true;
    error.value = null;
    try {
      entries.value = await invoke<Staff[]>("get_staff");
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function createEntry(name: string, title: string, role: string, sortOrder: number) {
    error.value = null;
    try {
      const entry = await invoke<Staff>("create_staff", {
        input: { name, title, role, sort_order: sortOrder },
      });
      entries.value.push(entry);
      return entry;
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  async function updateEntry(id: number, name: string, title: string, role: string, sortOrder: number) {
    error.value = null;
    try {
      const entry = await invoke<Staff>("update_staff", {
        id,
        input: { name, title, role, sort_order: sortOrder },
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
      await invoke("delete_staff", { id });
      entries.value = entries.value.filter((e) => e.id !== id);
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  async function uploadPhoto(id: number, filePath: string) {
    error.value = null;
    try {
      const savedPath = await invoke<string>("save_staff_photo", {
        staffId: id,
        sourcePath: filePath,
      });
      const idx = entries.value.findIndex((e) => e.id === id);
      if (idx !== -1) entries.value[idx].photo_path = savedPath;
      return savedPath;
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  async function removePhoto(id: number) {
    error.value = null;
    try {
      const entry = entries.value.find((e) => e.id === id);
      if (entry?.photo_path) {
        await invoke("delete_photo", { photoPath: `staff/${entry.photo_path}` });
      }
      await invoke("update_staff", {
        id,
        input: { name: entry!.name, title: entry!.title, role: entry!.role, sort_order: entry!.sort_order },
      });
      // Clear photo_path locally — the update_staff doesn't clear it, so we do it via a direct approach
      const idx = entries.value.findIndex((e) => e.id === id);
      if (idx !== -1) entries.value[idx].photo_path = undefined;
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  return {
    entries,
    loading,
    error,
    pastor,
    elders,
    staffMembers,
    fetchStaff,
    createEntry,
    updateEntry,
    deleteEntry,
    uploadPhoto,
    removePhoto,
  };
});
