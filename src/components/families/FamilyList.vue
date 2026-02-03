<script setup lang="ts">
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Family } from "../../types";

const props = defineProps<{
  families: Family[];
  sortBy: "name" | "updated_at";
  sortOrder: "asc" | "desc";
}>();

const emit = defineEmits<{
  sort: [field: "name" | "updated_at"];
  view: [id: number];
  edit: [id: number];
  delete: [id: number];
}>();

// Store loaded photo URLs
const photoCache = ref<Record<number, string | null>>({});

async function loadPhoto(family: Family) {
  if (!family.photo_path) {
    photoCache.value[family.id] = null;
    return;
  }
  if (photoCache.value[family.id] !== undefined) {
    return; // Already loaded or loading
  }
  try {
    const base64 = await invoke<string>("get_photo_base64", {
      photoType: "families",
      filename: family.photo_path,
    });
    photoCache.value[family.id] = base64;
  } catch (e) {
    photoCache.value[family.id] = null;
  }
}

// Load photos when families change
watch(
  () => props.families,
  (families) => {
    families.forEach(loadPhoto);
  },
  { immediate: true }
);
</script>

<template>
  <div class="bg-white dark:bg-gray-800 rounded-lg shadow-sm overflow-hidden h-full flex flex-col">
    <!-- Desktop table view -->
    <div class="hidden md:flex flex-col h-full overflow-hidden">
      <div class="flex-1 overflow-y-auto">
        <table class="w-full table-fixed">
          <colgroup>
            <col class="w-[25%]" />
            <col class="w-[30%] hidden lg:table-column" />
            <col class="w-[25%]" />
            <col class="w-[20%]" />
          </colgroup>
          <thead class="bg-gray-50 dark:bg-gray-700 border-b dark:border-gray-600 sticky top-0 z-10">
            <tr>
              <th
                @click="emit('sort', 'name')"
                class="px-4 py-3 text-left text-sm font-medium text-gray-600 dark:text-gray-300 cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-600"
              >
                <div class="flex items-center gap-2">
                  Name
                  <svg
                    v-if="sortBy === 'name'"
                    class="w-4 h-4"
                    :class="{ 'rotate-180': sortOrder === 'desc' }"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 15l7-7 7 7" />
                  </svg>
                </div>
              </th>
              <th class="px-4 py-3 text-left text-sm font-medium text-gray-600 dark:text-gray-300 hidden lg:table-cell">
                Address
              </th>
              <th class="px-4 py-3 text-left text-sm font-medium text-gray-600 dark:text-gray-300">Contact</th>
              <th class="px-4 py-3 text-right text-sm font-medium text-gray-600 dark:text-gray-300">Actions</th>
            </tr>
          </thead>
          <tbody class="divide-y dark:divide-gray-700">
            <tr
              v-for="family in families"
              :key="family.id"
              class="hover:bg-gray-50 dark:hover:bg-gray-700 cursor-pointer"
              @click="emit('view', family.id)"
            >
              <td class="px-4 py-3 align-top">
                <div class="flex items-center gap-3">
                  <img
                    v-if="photoCache[family.id]"
                    :src="photoCache[family.id]!"
                    class="w-10 h-10 rounded-full object-cover flex-shrink-0"
                    alt=""
                  />
                  <div class="font-medium text-gray-900 dark:text-gray-100 truncate">{{ family.name }}</div>
                </div>
              </td>
              <td class="px-4 py-3 text-sm text-gray-600 dark:text-gray-400 hidden lg:table-cell align-top">
                <div v-if="family.mailing_name" class="font-medium truncate">{{ family.mailing_name }}</div>
                <div v-if="family.address" class="truncate">{{ family.address }}</div>
                <div v-if="family.city || family.state || family.zip" class="truncate">
                  {{ [family.city, family.state].filter(Boolean).join(", ") }} {{ family.zip }}
                </div>
              </td>
              <td class="px-4 py-3 text-sm text-gray-600 dark:text-gray-400 align-top">
                <div v-if="family.phone">{{ family.phone }}</div>
                <div v-if="family.email" class="text-primary-600 dark:text-primary-400 truncate">{{ family.email }}</div>
              </td>
              <td class="px-4 py-3 text-right align-top" @click.stop>
                <div class="flex items-center justify-end gap-2">
                  <button
                    @click="emit('edit', family.id)"
                    class="p-1 text-gray-400 hover:text-primary-600 dark:hover:text-primary-400"
                    title="Edit"
                  >
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
                    </svg>
                  </button>
                  <button
                    @click="emit('delete', family.id)"
                    class="p-1 text-gray-400 hover:text-red-600"
                    title="Delete"
                  >
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                    </svg>
                  </button>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- Mobile card view -->
    <div class="md:hidden divide-y dark:divide-gray-700 overflow-y-auto h-full">
      <div
        v-for="family in families"
        :key="family.id"
        class="p-4 hover:bg-gray-50 dark:hover:bg-gray-700 cursor-pointer"
        @click="emit('view', family.id)"
      >
        <div class="flex items-start gap-3">
          <img
            v-if="photoCache[family.id]"
            :src="photoCache[family.id]!"
            class="w-12 h-12 rounded-full object-cover flex-shrink-0"
            alt=""
          />
          <div class="flex-1 min-w-0">
            <div class="font-medium text-gray-900 dark:text-gray-100">{{ family.name }}</div>
            <div v-if="family.phone" class="text-sm text-gray-600 dark:text-gray-400 mt-1">{{ family.phone }}</div>
            <div v-if="family.email" class="text-sm text-primary-600 dark:text-primary-400 truncate">{{ family.email }}</div>
            <div v-if="family.city || family.state" class="text-sm text-gray-500 dark:text-gray-500 mt-1">
              {{ [family.city, family.state].filter(Boolean).join(", ") }}
            </div>
          </div>
          <div class="flex items-center gap-1" @click.stop>
            <button
              @click="emit('edit', family.id)"
              class="p-2 text-gray-400 hover:text-primary-600 dark:hover:text-primary-400"
              title="Edit"
            >
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
              </svg>
            </button>
            <button
              @click="emit('delete', family.id)"
              class="p-2 text-gray-400 hover:text-red-600"
              title="Delete"
            >
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
              </svg>
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
