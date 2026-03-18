<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useStaffStore } from "../stores/staff";
import type { Staff } from "../types";

const store = useStaffStore();

const editingId = ref<number | null>(null);
const editName = ref("");
const editTitle = ref("");
const editRole = ref<string>("staff");

const showNewForm = ref(false);
const newName = ref("");
const newTitle = ref("");
const newRole = ref<string>("staff");

const photoPreviews = ref<Record<number, string | null>>({});

onMounted(async () => {
  await store.fetchStaff();
  for (const entry of store.entries) {
    if (entry.photo_path) {
      await loadPhotoPreview(entry.id, entry.photo_path);
    }
  }
});

async function loadPhotoPreview(id: number, filename: string) {
  try {
    const base64 = await invoke<string>("get_photo_base64", {
      photoType: "staff",
      filename,
    });
    photoPreviews.value[id] = base64;
  } catch {
    photoPreviews.value[id] = null;
  }
}

function startAdd(role: string) {
  showNewForm.value = true;
  newName.value = "";
  newTitle.value = "";
  newRole.value = role;
}

async function saveNew() {
  if (!newName.value.trim()) return;
  const sortOrder = newRole.value === "pastor" ? 0 : newRole.value === "elder" ? 10 : 20;
  const entry = await store.createEntry(newName.value.trim(), newTitle.value.trim(), newRole.value, sortOrder);
  showNewForm.value = false;
  return entry;
}

function cancelNew() {
  showNewForm.value = false;
}

function startEdit(entry: Staff) {
  editingId.value = entry.id;
  editName.value = entry.name;
  editTitle.value = entry.title;
  editRole.value = entry.role;
}

async function saveEdit() {
  if (editingId.value === null || !editName.value.trim()) return;
  const existing = store.entries.find((e) => e.id === editingId.value);
  await store.updateEntry(editingId.value, editName.value.trim(), editTitle.value.trim(), editRole.value, existing?.sort_order ?? 0);
  editingId.value = null;
}

function cancelEdit() {
  editingId.value = null;
}

async function remove(id: number) {
  await store.deleteEntry(id);
  delete photoPreviews.value[id];
}

async function uploadPhoto(id: number) {
  const filePath = await open({
    multiple: false,
    filters: [{ name: "Images", extensions: ["png", "jpg", "jpeg", "webp"] }],
  });
  if (!filePath || typeof filePath !== "string") return;
  const savedPath = await store.uploadPhoto(id, filePath);
  if (savedPath) {
    await loadPhotoPreview(id, savedPath);
  }
}

async function removePhoto(id: number) {
  await store.removePhoto(id);
  photoPreviews.value[id] = null;
}

const hasPastor = () => !!store.pastor;
</script>

<template>
  <div class="p-6 h-full overflow-y-auto">
    <h1 class="text-2xl font-bold text-gray-800 dark:text-gray-100 mb-6">Pastor, Elders & Staff</h1>

    <!-- Error -->
    <div v-if="store.error" class="mb-4 p-4 bg-red-50 dark:bg-red-900/30 text-red-700 dark:text-red-400 rounded-lg">
      {{ store.error }}
    </div>

    <!-- Loading -->
    <div v-if="store.loading" class="text-center py-8 text-gray-500 dark:text-gray-400">Loading...</div>

    <template v-else>
      <!-- New form -->
      <div v-if="showNewForm" class="bg-white dark:bg-gray-800 rounded-lg shadow-sm p-4 mb-6">
        <h2 class="text-lg font-semibold text-gray-800 dark:text-gray-100 mb-3">
          Add {{ newRole === 'pastor' ? 'Pastor' : newRole === 'elder' ? 'Elder' : 'Staff Member' }}
        </h2>
        <div class="space-y-3">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Name</label>
            <input
              v-model="newName"
              type="text"
              placeholder="Full name"
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Title</label>
            <input
              v-model="newTitle"
              type="text"
              placeholder="e.g. Senior Pastor, Youth Director..."
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
              @keyup.enter="saveNew"
            />
          </div>
          <div class="flex gap-2">
            <button
              @click="saveNew"
              :disabled="!newName.trim()"
              class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50"
            >
              Save
            </button>
            <button @click="cancelNew" class="px-4 py-2 text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg">
              Cancel
            </button>
          </div>
        </div>
      </div>

      <!-- Pastor Section -->
      <section class="mb-8">
        <div class="flex items-center justify-between mb-3">
          <h2 class="text-lg font-semibold text-gray-800 dark:text-gray-100">Pastor</h2>
          <button
            v-if="!hasPastor() && !showNewForm"
            @click="startAdd('pastor')"
            class="px-3 py-1.5 text-sm bg-primary-600 text-white rounded-lg hover:bg-primary-700 flex items-center gap-1"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
            Add Pastor
          </button>
        </div>
        <div v-if="store.pastor" class="bg-white dark:bg-gray-800 rounded-lg shadow-sm">
          <!-- Editing -->
          <div v-if="editingId === store.pastor.id" class="p-4">
            <div class="space-y-3">
              <input
                v-model="editName"
                type="text"
                placeholder="Name"
                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
              />
              <input
                v-model="editTitle"
                type="text"
                placeholder="Title"
                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
                @keyup.enter="saveEdit"
              />
              <div class="flex gap-2">
                <button @click="saveEdit" :disabled="!editName.trim()" class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50">Save</button>
                <button @click="cancelEdit" class="px-4 py-2 text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg">Cancel</button>
              </div>
            </div>
          </div>
          <!-- Display -->
          <div v-else class="p-4 flex items-center gap-4">
            <div
              class="w-24 aspect-[4/6] rounded-lg overflow-hidden bg-gray-100 dark:bg-gray-700 flex items-center justify-center shrink-0 cursor-pointer"
              @click="uploadPhoto(store.pastor.id)"
              :title="photoPreviews[store.pastor.id] ? 'Change photo' : 'Add photo'"
            >
              <img v-if="photoPreviews[store.pastor.id]" :src="photoPreviews[store.pastor.id]!" class="w-full h-full object-cover" />
              <svg v-else class="w-8 h-8 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
              </svg>
            </div>
            <div class="flex-1 min-w-0">
              <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">{{ store.pastor.name }}</h3>
              <p class="text-sm text-gray-500 dark:text-gray-400">{{ store.pastor.title || 'Pastor' }}</p>
            </div>
            <div class="flex gap-1 shrink-0">
              <button v-if="photoPreviews[store.pastor.id]" @click="removePhoto(store.pastor.id)" class="p-2 text-gray-500 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg" title="Remove photo">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
                </svg>
              </button>
              <button @click="startEdit(store.pastor)" class="p-2 text-gray-500 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg" title="Edit">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                </svg>
              </button>
              <button @click="remove(store.pastor.id)" class="p-2 text-red-500 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg" title="Delete">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                </svg>
              </button>
            </div>
          </div>
        </div>
        <div v-else-if="!showNewForm" class="text-sm text-gray-400 dark:text-gray-500 italic">No pastor added</div>
      </section>

      <!-- Elders Section -->
      <section class="mb-8">
        <div class="flex items-center justify-between mb-3">
          <h2 class="text-lg font-semibold text-gray-800 dark:text-gray-100">Elders</h2>
          <button
            v-if="!showNewForm"
            @click="startAdd('elder')"
            class="px-3 py-1.5 text-sm bg-primary-600 text-white rounded-lg hover:bg-primary-700 flex items-center gap-1"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
            Add Elder
          </button>
        </div>
        <div v-if="store.elders.length" class="space-y-2">
          <div v-for="entry in store.elders" :key="entry.id" class="bg-white dark:bg-gray-800 rounded-lg shadow-sm">
            <!-- Editing -->
            <div v-if="editingId === entry.id" class="p-4">
              <div class="space-y-3">
                <input
                  v-model="editName"
                  type="text"
                  class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
                  @keyup.enter="saveEdit"
                />
                <div class="flex gap-2">
                  <button @click="saveEdit" :disabled="!editName.trim()" class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50">Save</button>
                  <button @click="cancelEdit" class="px-4 py-2 text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg">Cancel</button>
                </div>
              </div>
            </div>
            <!-- Display -->
            <div v-else class="p-4 flex items-center gap-4">
              <div
                class="w-20 aspect-[4/6] rounded-lg overflow-hidden bg-gray-100 dark:bg-gray-700 flex items-center justify-center shrink-0 cursor-pointer"
                @click="uploadPhoto(entry.id)"
                :title="photoPreviews[entry.id] ? 'Change photo' : 'Add photo'"
              >
                <img v-if="photoPreviews[entry.id]" :src="photoPreviews[entry.id]!" class="w-full h-full object-cover" />
                <svg v-else class="w-6 h-6 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
                </svg>
              </div>
              <div class="flex-1 min-w-0">
                <h3 class="font-medium text-gray-900 dark:text-gray-100">{{ entry.name }}</h3>
                <p v-if="entry.title" class="text-sm text-gray-500 dark:text-gray-400">{{ entry.title }}</p>
              </div>
              <div class="flex gap-1 shrink-0">
                <button v-if="photoPreviews[entry.id]" @click="removePhoto(entry.id)" class="p-2 text-gray-500 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg" title="Remove photo">
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
                  </svg>
                </button>
                <button @click="startEdit(entry)" class="p-2 text-gray-500 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg" title="Edit">
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                  </svg>
                </button>
                <button @click="remove(entry.id)" class="p-2 text-red-500 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg" title="Delete">
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                  </svg>
                </button>
              </div>
            </div>
          </div>
        </div>
        <div v-else-if="!showNewForm" class="text-sm text-gray-400 dark:text-gray-500 italic">No elders added</div>
      </section>

      <!-- Staff Section -->
      <section>
        <div class="flex items-center justify-between mb-3">
          <h2 class="text-lg font-semibold text-gray-800 dark:text-gray-100">Staff</h2>
          <button
            v-if="!showNewForm"
            @click="startAdd('staff')"
            class="px-3 py-1.5 text-sm bg-primary-600 text-white rounded-lg hover:bg-primary-700 flex items-center gap-1"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
            Add Staff
          </button>
        </div>
        <div v-if="store.staffMembers.length" class="space-y-2">
          <div v-for="entry in store.staffMembers" :key="entry.id" class="bg-white dark:bg-gray-800 rounded-lg shadow-sm">
            <!-- Editing -->
            <div v-if="editingId === entry.id" class="p-4">
              <div class="space-y-3">
                <input
                  v-model="editName"
                  type="text"
                  class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
                  @keyup.enter="saveEdit"
                />
                <div class="flex gap-2">
                  <button @click="saveEdit" :disabled="!editName.trim()" class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50">Save</button>
                  <button @click="cancelEdit" class="px-4 py-2 text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg">Cancel</button>
                </div>
              </div>
            </div>
            <!-- Display -->
            <div v-else class="p-4 flex items-center justify-between gap-4">
              <div class="flex-1 min-w-0">
                <h3 class="font-medium text-gray-900 dark:text-gray-100">{{ entry.name }}</h3>
                <p v-if="entry.title" class="text-sm text-gray-500 dark:text-gray-400">{{ entry.title }}</p>
              </div>
              <div class="flex gap-1 shrink-0">
                <button @click="startEdit(entry)" class="p-2 text-gray-500 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg" title="Edit">
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                  </svg>
                </button>
                <button @click="remove(entry.id)" class="p-2 text-red-500 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg" title="Delete">
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                  </svg>
                </button>
              </div>
            </div>
          </div>
        </div>
        <div v-else-if="!showNewForm" class="text-sm text-gray-400 dark:text-gray-500 italic">No staff added</div>
      </section>
    </template>
  </div>
</template>
