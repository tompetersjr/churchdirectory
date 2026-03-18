<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
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
  church_name: "",
  cover_image_path: undefined,
  cover_title_line1: undefined,
  cover_title_line2: undefined,
  cover_title_color: "#FFFFFF",
  pastor_letter: undefined,
  mission_statement: undefined,
  first_page_markdown: undefined,
  back_cover_image_path: undefined,
  celebration_image_path: undefined,
});

// Image previews (base64 data URIs)
const imagePreviews = ref<Record<string, string | null>>({});

const imageSlots = [
  { key: "cover_image", label: "Front Cover", description: "The front of the booklet" },
  { key: "back_cover_image", label: "Back Cover", description: "The back of the booklet" },
  { key: "celebration_image", label: "Celebrations", description: "Header image for birthdays & anniversaries" },
];

onMounted(async () => {
  await settingsStore.fetchSettings();
  options.value = {
    church_name: settingsStore.settings.church_name,
    cover_image_path: settingsStore.settings.cover_image_path,
    cover_title_line1: settingsStore.settings.cover_title_line1,
    cover_title_line2: settingsStore.settings.cover_title_line2,
    cover_title_color: settingsStore.settings.cover_title_color || "#FFFFFF",
    pastor_letter: settingsStore.settings.pastor_letter,
    mission_statement: settingsStore.settings.mission_statement,
    first_page_markdown: settingsStore.settings.first_page_markdown,
    back_cover_image_path: settingsStore.settings.back_cover_image_path,
    celebration_image_path: settingsStore.settings.celebration_image_path,
  };

  try {
    familyCount.value = await invoke<number>("get_family_count");
  } catch (e) {
    console.error("Failed to get family count:", e);
  }

  // Load existing image previews
  for (const slot of imageSlots) {
    const settingKey = `${slot.key}_path` as keyof PdfOptions;
    const filename = options.value[settingKey];
    if (filename) {
      await loadImagePreview(slot.key, filename);
    }
  }
});

async function loadImagePreview(imageName: string, filename: string) {
  try {
    const base64 = await invoke<string>("get_photo_base64", {
      photoType: "directory",
      filename,
    });
    imagePreviews.value[imageName] = base64;
  } catch (e) {
    imagePreviews.value[imageName] = null;
  }
}

async function uploadImage(imageName: string) {
  const filePath = await open({
    multiple: false,
    filters: [
      {
        name: "Images",
        extensions: ["png", "jpg", "jpeg", "webp"],
      },
    ],
  });

  if (!filePath || typeof filePath !== "string") return;

  try {
    await settingsStore.setDirectoryImage(filePath, imageName);
    const settingKey = `${imageName}_path` as keyof PdfOptions;
    const savedPath = settingsStore.settings[settingKey as keyof typeof settingsStore.settings] as string;
    (options.value as Record<string, unknown>)[settingKey] = savedPath;
    if (savedPath) {
      await loadImagePreview(imageName, savedPath);
    }
  } catch (e) {
    error.value = `Failed to upload image: ${e}`;
  }
}

async function removeImage(imageName: string) {
  const settingKey = `${imageName}_path` as keyof PdfOptions;
  (options.value as Record<string, unknown>)[settingKey] = undefined;
  imagePreviews.value[imageName] = null;
  await settingsStore.saveSettings({ [settingKey]: undefined });
}

async function saveCoverTitle() {
  try {
    await settingsStore.saveSettings({
      cover_title_line1: options.value.cover_title_line1,
      cover_title_line2: options.value.cover_title_line2,
      cover_title_color: options.value.cover_title_color,
    });
  } catch (e) {
    error.value = `Failed to save cover title: ${e}`;
  }
}

async function savePastorLetter() {
  try {
    await settingsStore.saveSettings({ pastor_letter: options.value.pastor_letter });
  } catch (e) {
    error.value = `Failed to save: ${e}`;
  }
}

async function saveMissionStatement() {
  try {
    await settingsStore.saveSettings({ mission_statement: options.value.mission_statement });
  } catch (e) {
    error.value = `Failed to save: ${e}`;
  }
}

async function generatePdf() {
  const outputPath = await save({
    defaultPath: "church-directory-booklet.pdf",
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
      <!-- Booklet Info -->
      <div class="mb-6 p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
        <p class="text-sm text-blue-700 dark:text-blue-300">
          Generates a saddle-stitched booklet on 8.5" x 14" (US Legal) paper with <strong>{{ familyCount }}</strong> families.
          Pages are imposed for duplex printing and folding.
          Images should be 7" x 8.5" (portrait) or a 7:8.5 aspect ratio for best results.
        </p>
      </div>

      <!-- Directory Images -->
      <div class="mb-6">
        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
          Booklet Images
        </label>
        <div class="grid grid-cols-3 gap-3">
          <div
            v-for="slot in imageSlots"
            :key="slot.key"
            class="p-2 border border-gray-200 dark:border-gray-700 rounded-lg"
          >
            <!-- Thumbnail -->
            <div class="w-full aspect-[7/8.5] rounded overflow-hidden bg-gray-100 dark:bg-gray-700 flex items-center justify-center mb-2">
              <img
                v-if="imagePreviews[slot.key]"
                :src="imagePreviews[slot.key]!"
                class="w-full h-full object-cover"
                alt=""
              />
              <svg v-else class="w-10 h-10 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
              </svg>
            </div>

            <!-- Label & Description -->
            <p class="text-sm font-medium text-gray-900 dark:text-gray-100">{{ slot.label }}</p>
            <p class="text-xs text-gray-500 dark:text-gray-400 mb-2">{{ slot.description }}</p>

            <!-- Actions -->
            <div class="flex gap-2">
              <button
                @click="uploadImage(slot.key)"
                class="px-3 py-1.5 text-sm bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded hover:bg-gray-200 dark:hover:bg-gray-600"
              >
                {{ imagePreviews[slot.key] ? 'Change' : 'Upload' }}
              </button>
              <button
                v-if="imagePreviews[slot.key]"
                @click="removeImage(slot.key)"
                class="px-3 py-1.5 text-sm text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 rounded"
              >
                Remove
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Cover Title -->
      <div class="mb-6">
        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
          Cover Title
        </label>
        <p class="text-xs text-gray-500 dark:text-gray-400 mb-2">
          Two lines of title text displayed on the front cover. The current month and year will be added automatically as a third line.
        </p>
        <input
          v-model="options.cover_title_line1"
          type="text"
          placeholder="Line 1 (e.g. church name)"
          class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm focus:ring-2 focus:ring-primary-500 focus:border-primary-500 mb-2"
        />
        <input
          v-model="options.cover_title_line2"
          type="text"
          placeholder="Line 2 (e.g. Photo Directory)"
          class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
        />
        <div class="mt-3 flex items-center gap-3">
          <label class="text-sm text-gray-600 dark:text-gray-400">Text Color</label>
          <div class="relative">
            <input
              v-model="options.cover_title_color"
              type="color"
              class="sr-only peer"
              :id="'cover-color-picker'"
            />
            <label
              :for="'cover-color-picker'"
              class="flex items-center gap-2 cursor-pointer px-3 py-1.5 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 hover:bg-gray-50 dark:hover:bg-gray-600 transition-colors"
            >
              <span
                class="w-6 h-6 rounded-full border-2 border-gray-300 dark:border-gray-500 shadow-inner"
                :style="{ backgroundColor: options.cover_title_color || '#FFFFFF' }"
              ></span>
              <span class="text-sm text-gray-700 dark:text-gray-300 font-mono uppercase">{{ options.cover_title_color || '#FFFFFF' }}</span>
            </label>
          </div>
          <div class="flex gap-1">
            <button
              v-for="preset in ['#FFFFFF', '#000000', '#1E3A5F', '#8B4513', '#2F4F2F', '#4A0E2E']"
              :key="preset"
              @click="options.cover_title_color = preset"
              class="w-6 h-6 rounded-full border-2 transition-all"
              :class="options.cover_title_color === preset ? 'border-primary-500 scale-110' : 'border-gray-300 dark:border-gray-500 hover:scale-105'"
              :style="{ backgroundColor: preset }"
              :title="preset"
            ></button>
          </div>
        </div>
        <button
          v-if="options.cover_title_line1 || options.cover_title_line2"
          @click="saveCoverTitle"
          class="mt-3 px-3 py-1.5 text-sm bg-primary-600 text-white rounded hover:bg-primary-700"
        >
          Save
        </button>
      </div>

      <!-- First Inside Page -->
      <div class="mb-6">
        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
          First Inside Page
        </label>

        <!-- Letter From the Pastor -->
        <div class="mb-4">
          <label class="block text-sm text-gray-600 dark:text-gray-400 mb-1">
            Letter From the Pastor
          </label>
          <textarea
            v-model="options.pastor_letter"
            rows="6"
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
          ></textarea>
          <button
            v-if="options.pastor_letter"
            @click="savePastorLetter"
            class="mt-2 px-3 py-1.5 text-sm bg-primary-600 text-white rounded hover:bg-primary-700"
          >
            Save
          </button>
        </div>

        <!-- Mission Statement -->
        <div>
          <label class="block text-sm text-gray-600 dark:text-gray-400 mb-1">
            Mission Statement
          </label>
          <textarea
            v-model="options.mission_statement"
            rows="4"
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
          ></textarea>
          <button
            v-if="options.mission_statement"
            @click="saveMissionStatement"
            class="mt-2 px-3 py-1.5 text-sm bg-primary-600 text-white rounded hover:bg-primary-700"
          >
            Save
          </button>
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
        {{ loading ? "Generating..." : "Generate Booklet PDF" }}
      </button>
    </div>
  </div>
</template>
