import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Family, Member, FamilyWithMembers } from "../types";

export const useFamiliesStore = defineStore("families", () => {
  const families = ref<Family[]>([]);
  const currentFamily = ref<FamilyWithMembers | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const searchQuery = ref("");
  const sortBy = ref<"name" | "updated_at">("name");
  const sortOrder = ref<"asc" | "desc">("asc");

  const filteredFamilies = computed(() => {
    let result = [...families.value];

    if (searchQuery.value) {
      const query = searchQuery.value.toLowerCase();
      result = result.filter(
        (f) =>
          f.name.toLowerCase().includes(query) ||
          f.family_id.toLowerCase().includes(query) ||
          f.address?.toLowerCase().includes(query) ||
          f.email?.toLowerCase().includes(query)
      );
    }

    result.sort((a, b) => {
      let aVal: string | number = "";
      let bVal: string | number = "";

      switch (sortBy.value) {
        case "name":
          aVal = a.name.toLowerCase();
          bVal = b.name.toLowerCase();
          break;
        case "updated_at":
          aVal = new Date(a.updated_at).getTime();
          bVal = new Date(b.updated_at).getTime();
          break;
      }

      if (aVal < bVal) return sortOrder.value === "asc" ? -1 : 1;
      if (aVal > bVal) return sortOrder.value === "asc" ? 1 : -1;
      return 0;
    });

    return result;
  });

  async function fetchFamilies() {
    loading.value = true;
    error.value = null;
    try {
      families.value = await invoke<Family[]>("get_families");
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function fetchFamily(id: number) {
    loading.value = true;
    error.value = null;
    try {
      currentFamily.value = await invoke<FamilyWithMembers>("get_family", {
        id,
      });
    } catch (e) {
      error.value = String(e);
      currentFamily.value = null;
    } finally {
      loading.value = false;
    }
  }

  async function createFamily(family: Omit<Family, "id" | "created_at" | "updated_at">) {
    loading.value = true;
    error.value = null;
    try {
      const id = await invoke<number>("create_family", { family });
      await fetchFamilies();
      return id;
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function updateFamily(id: number, family: Partial<Family>) {
    loading.value = true;
    error.value = null;
    try {
      await invoke("update_family", { id, family });
      await fetchFamilies();
      if (currentFamily.value?.id === id) {
        await fetchFamily(id);
      }
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function deleteFamily(id: number) {
    loading.value = true;
    error.value = null;
    try {
      await invoke("delete_family", { id });
      await fetchFamilies();
      if (currentFamily.value?.id === id) {
        currentFamily.value = null;
      }
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function createMember(member: Omit<Member, "id" | "created_at" | "updated_at">) {
    loading.value = true;
    error.value = null;
    try {
      const id = await invoke<number>("create_member", { member });
      if (currentFamily.value?.id === member.family_id) {
        await fetchFamily(member.family_id);
      }
      return id;
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function updateMember(id: number, member: Partial<Member>) {
    loading.value = true;
    error.value = null;
    try {
      await invoke("update_member", { id, member });
      if (currentFamily.value) {
        await fetchFamily(currentFamily.value.id);
      }
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function deleteMember(id: number) {
    loading.value = true;
    error.value = null;
    try {
      await invoke("delete_member", { id });
      if (currentFamily.value) {
        await fetchFamily(currentFamily.value.id);
      }
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function getMember(id: number): Promise<Member> {
    return await invoke<Member>("get_member", { id });
  }

  async function saveFamilyPhoto(familyId: number, sourcePath: string): Promise<string> {
    const photoPath = await invoke<string>("save_family_photo", {
      familyId,
      sourcePath,
    });
    if (currentFamily.value?.id === familyId) {
      await fetchFamily(familyId);
    }
    await fetchFamilies();
    return photoPath;
  }

  async function removeFamilyPhoto(familyId: number) {
    if (currentFamily.value?.photo_path) {
      await invoke("delete_photo", { photoPath: currentFamily.value.photo_path });
    }
    await invoke("update_family", { id: familyId, family: { photo_path: null } });
    if (currentFamily.value?.id === familyId) {
      await fetchFamily(familyId);
    }
    await fetchFamilies();
  }

  async function getPhotosDir(): Promise<string> {
    return await invoke<string>("get_photo_path", { relativePath: "families" });
  }

  async function cropFamilyPhotoToMember(
    familyId: number,
    memberId: number,
    cropData: { x: number; y: number; width: number; height: number },
  ): Promise<string> {
    const photoPath = await invoke<string>("crop_family_photo_to_member", {
      familyId,
      memberId,
      x: cropData.x,
      y: cropData.y,
      width: cropData.width,
      height: cropData.height,
    });
    if (currentFamily.value?.id === familyId) {
      await fetchFamily(familyId);
    }
    return photoPath;
  }

  return {
    families,
    currentFamily,
    loading,
    error,
    searchQuery,
    sortBy,
    sortOrder,
    filteredFamilies,
    fetchFamilies,
    fetchFamily,
    createFamily,
    updateFamily,
    deleteFamily,
    createMember,
    updateMember,
    deleteMember,
    getMember,
    saveFamilyPhoto,
    removeFamilyPhoto,
    getPhotosDir,
    cropFamilyPhotoToMember,
  };
});
