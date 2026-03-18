<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useLeadershipStore } from "../stores/leadership";
import type { Leadership } from "../types";

const store = useLeadershipStore();

const editingId = ref<number | null>(null);
const editMinistry = ref("");
const editNames = ref("");
const editSortOrder = ref(0);

const showNewForm = ref(false);
const newMinistry = ref("");
const newNames = ref("");
const newSortOrder = ref(0);

onMounted(() => {
  store.fetchLeadership();
});

function startAdd() {
  showNewForm.value = true;
  newMinistry.value = "";
  newNames.value = "";
  newSortOrder.value = store.entries.length;
}

async function saveNew() {
  if (!newMinistry.value.trim()) return;
  await store.createEntry(newMinistry.value.trim(), newNames.value.trim(), newSortOrder.value);
  showNewForm.value = false;
}

function cancelNew() {
  showNewForm.value = false;
}

function startEdit(entry: Leadership) {
  editingId.value = entry.id;
  editMinistry.value = entry.ministry;
  editNames.value = entry.names;
  editSortOrder.value = entry.sort_order;
}

async function saveEdit() {
  if (editingId.value === null || !editMinistry.value.trim()) return;
  await store.updateEntry(
    editingId.value,
    editMinistry.value.trim(),
    editNames.value.trim(),
    editSortOrder.value,
  );
  editingId.value = null;
}

function cancelEdit() {
  editingId.value = null;
}

async function remove(id: number) {
  await store.deleteEntry(id);
}
</script>

<template>
  <div class="p-6 h-full overflow-y-auto">
    <div class="flex items-center justify-between mb-6">
      <h1 class="text-2xl font-bold text-gray-800 dark:text-gray-100">Leadership</h1>
      <button
        v-if="!showNewForm"
        @click="startAdd"
        class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 flex items-center gap-2"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
        Add Ministry
      </button>
    </div>

    <!-- New ministry form -->
    <div v-if="showNewForm" class="bg-white dark:bg-gray-800 rounded-lg shadow-sm p-4 mb-4">
      <h2 class="text-lg font-semibold text-gray-800 dark:text-gray-100 mb-3">New Ministry</h2>
      <div class="space-y-3">
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Ministry Name</label>
          <input
            v-model="newMinistry"
            type="text"
            placeholder="e.g. Pastor, Elders, Deacons..."
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
          />
        </div>
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Names</label>
          <textarea
            v-model="newNames"
            rows="3"
            placeholder="One name per line"
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
          ></textarea>
        </div>
        <div class="flex gap-2">
          <button
            @click="saveNew"
            :disabled="!newMinistry.trim()"
            class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50"
          >
            Save
          </button>
          <button
            @click="cancelNew"
            class="px-4 py-2 text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg"
          >
            Cancel
          </button>
        </div>
      </div>
    </div>

    <!-- Error -->
    <div v-if="store.error" class="mb-4 p-4 bg-red-50 dark:bg-red-900/30 text-red-700 dark:text-red-400 rounded-lg">
      {{ store.error }}
    </div>

    <!-- Loading -->
    <div v-if="store.loading" class="text-center py-8 text-gray-500 dark:text-gray-400">
      Loading...
    </div>

    <!-- Empty state -->
    <div
      v-else-if="store.entries.length === 0 && !showNewForm"
      class="text-center py-12"
    >
      <svg class="w-16 h-16 mx-auto text-gray-300 dark:text-gray-600 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
      </svg>
      <p class="text-gray-500 dark:text-gray-400 mb-4">No ministries added yet</p>
      <button
        @click="startAdd"
        class="text-primary-600 dark:text-primary-400 hover:text-primary-700 dark:hover:text-primary-300"
      >
        Add your first ministry
      </button>
    </div>

    <!-- Ministry list -->
    <div v-else class="space-y-3">
      <div
        v-for="entry in store.entries"
        :key="entry.id"
        class="bg-white dark:bg-gray-800 rounded-lg shadow-sm"
      >
        <!-- Editing mode -->
        <div v-if="editingId === entry.id" class="p-4">
          <div class="space-y-3">
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Ministry Name</label>
              <input
                v-model="editMinistry"
                type="text"
                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
              />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Names</label>
              <textarea
                v-model="editNames"
                rows="3"
                placeholder="One name per line"
                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
              ></textarea>
            </div>
            <div class="flex gap-2">
              <button
                @click="saveEdit"
                :disabled="!editMinistry.trim()"
                class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50"
              >
                Save
              </button>
              <button
                @click="cancelEdit"
                class="px-4 py-2 text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg"
              >
                Cancel
              </button>
            </div>
          </div>
        </div>

        <!-- Display mode -->
        <div v-else class="p-4 flex items-start justify-between gap-4">
          <div class="flex-1 min-w-0">
            <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">{{ entry.ministry }}</h3>
            <div v-if="entry.names" class="mt-1 text-sm text-gray-600 dark:text-gray-400">
              <p v-for="(name, ni) in entry.names.split('\n').filter(n => n.trim())" :key="ni">
                {{ name.trim() }}
              </p>
            </div>
            <p v-else class="mt-1 text-sm text-gray-400 dark:text-gray-500 italic">No names added</p>
          </div>
          <div class="flex gap-1 shrink-0">
            <button
              @click="startEdit(entry)"
              class="p-2 text-gray-500 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg"
              title="Edit"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
              </svg>
            </button>
            <button
              @click="remove(entry.id)"
              class="p-2 text-red-500 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg"
              title="Delete"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
              </svg>
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
