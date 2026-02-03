<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { useSettingsStore } from "../stores/settings";
import type { PdfOptions } from "../types";

const emit = defineEmits<{
  complete: [path: string];
}>();

const settingsStore = useSettingsStore();

const loading = ref(false);
const error = ref<string | null>(null);
const familyCount = ref(0);

const options = ref<PdfOptions>({
  layout: "grid",
  page_size: "letter",
  include_photos: true,
  include_contact_info: true,
  include_address: true,
  include_cover: true,
  include_toc: true,
  church_name: "",
  church_logo_path: undefined,
});

onMounted(async () => {
  await settingsStore.fetchSettings();
  options.value = {
    ...options.value,
    layout: settingsStore.settings.default_layout,
    page_size: settingsStore.settings.page_size,
    include_photos: settingsStore.settings.include_photos,
    include_contact_info: settingsStore.settings.include_contact_info,
    include_address: settingsStore.settings.include_address,
    church_name: settingsStore.settings.church_name,
    church_logo_path: settingsStore.settings.church_logo_path,
  };

  try {
    familyCount.value = await invoke<number>("get_family_count");
  } catch (e) {
    console.error("Failed to get family count:", e);
  }
});

async function generatePdf() {
  const outputPath = await save({
    defaultPath: "church-directory.pdf",
    filters: [
      {
        name: "PDF Files",
        extensions: ["pdf"],
      },
    ],
  });

  if (!outputPath) return;

  loading.value = true;
  error.value = null;

  try {
    const path = await invoke<string>("generate_pdf", {
      options: options.value,
      outputPath,
    });
    emit("complete", path);
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}
</script>

<template>
  <div class="bg-white dark:bg-gray-800 rounded-lg shadow-sm p-6">
    <div v-if="familyCount === 0" class="text-center py-8">
      <svg class="w-16 h-16 mx-auto text-gray-300 dark:text-gray-600 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
      </svg>
      <p class="text-gray-500 dark:text-gray-400 mb-4">No families to include in the directory</p>
      <router-link to="/import" class="text-primary-600 dark:text-primary-400 hover:text-primary-700 dark:hover:text-primary-300">
        Import data first
      </router-link>
    </div>

    <div v-else>
      <div class="mb-6">
        <p class="text-gray-600 dark:text-gray-400">
          Generate a PDF directory with <strong class="dark:text-gray-200">{{ familyCount }}</strong> families.
        </p>
      </div>

      <!-- Church Name -->
      <div class="mb-6">
        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
          Church Name
        </label>
        <input
          v-model="options.church_name"
          type="text"
          class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500 bg-white dark:bg-gray-700 dark:text-gray-100"
          placeholder="Enter church name"
        />
      </div>

      <!-- Layout Options -->
      <div class="mb-6">
        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">Layout</label>
        <div class="grid grid-cols-2 gap-4">
          <label
            class="flex items-center gap-3 p-4 border rounded-lg cursor-pointer"
            :class="
              options.layout === 'grid'
                ? 'border-primary-500 bg-primary-50 dark:bg-primary-900/30'
                : 'dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700'
            "
          >
            <input
              v-model="options.layout"
              type="radio"
              value="grid"
              class="sr-only"
            />
            <div class="flex-1">
              <p class="font-medium dark:text-gray-200">Grid</p>
              <p class="text-sm text-gray-500 dark:text-gray-400">2 columns per page</p>
            </div>
          </label>
          <label
            class="flex items-center gap-3 p-4 border rounded-lg cursor-pointer"
            :class="
              options.layout === 'list'
                ? 'border-primary-500 bg-primary-50 dark:bg-primary-900/30'
                : 'dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700'
            "
          >
            <input
              v-model="options.layout"
              type="radio"
              value="list"
              class="sr-only"
            />
            <div class="flex-1">
              <p class="font-medium dark:text-gray-200">List</p>
              <p class="text-sm text-gray-500 dark:text-gray-400">1 column per page</p>
            </div>
          </label>
        </div>
      </div>

      <!-- Page Size -->
      <div class="mb-6">
        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
          Page Size
        </label>
        <select
          v-model="options.page_size"
          class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500 bg-white dark:bg-gray-700 dark:text-gray-100"
        >
          <option value="letter">Letter (8.5" x 11")</option>
          <option value="a4">A4 (210mm x 297mm)</option>
        </select>
      </div>

      <!-- Content Options -->
      <div class="mb-6">
        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          Content Options
        </label>
        <div class="space-y-3">
          <label class="flex items-center gap-3">
            <input
              v-model="options.include_cover"
              type="checkbox"
              class="w-4 h-4 text-primary-600 border-gray-300 dark:border-gray-600 rounded focus:ring-primary-500"
            />
            <span class="text-sm text-gray-700 dark:text-gray-300">Include cover page</span>
          </label>
          <label class="flex items-center gap-3">
            <input
              v-model="options.include_toc"
              type="checkbox"
              class="w-4 h-4 text-primary-600 border-gray-300 dark:border-gray-600 rounded focus:ring-primary-500"
            />
            <span class="text-sm text-gray-700 dark:text-gray-300">Include table of contents</span>
          </label>
          <label class="flex items-center gap-3">
            <input
              v-model="options.include_photos"
              type="checkbox"
              class="w-4 h-4 text-primary-600 border-gray-300 dark:border-gray-600 rounded focus:ring-primary-500"
            />
            <span class="text-sm text-gray-700 dark:text-gray-300">Include photos</span>
          </label>
          <label class="flex items-center gap-3">
            <input
              v-model="options.include_address"
              type="checkbox"
              class="w-4 h-4 text-primary-600 border-gray-300 dark:border-gray-600 rounded focus:ring-primary-500"
            />
            <span class="text-sm text-gray-700 dark:text-gray-300">Include addresses</span>
          </label>
          <label class="flex items-center gap-3">
            <input
              v-model="options.include_contact_info"
              type="checkbox"
              class="w-4 h-4 text-primary-600 border-gray-300 dark:border-gray-600 rounded focus:ring-primary-500"
            />
            <span class="text-sm text-gray-700 dark:text-gray-300">Include contact information</span>
          </label>
        </div>
      </div>

      <!-- Error -->
      <div v-if="error" class="mb-6 p-4 bg-red-50 dark:bg-red-900/30 text-red-700 dark:text-red-400 rounded-lg">
        {{ error }}
      </div>

      <!-- Generate Button -->
      <button
        @click="generatePdf"
        :disabled="loading"
        class="w-full px-6 py-3 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
      >
        <span
          v-if="loading"
          class="animate-spin rounded-full h-5 w-5 border-2 border-white border-t-transparent"
        ></span>
        <svg v-else class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
        </svg>
        {{ loading ? "Generating..." : "Generate PDF" }}
      </button>
    </div>
  </div>
</template>
