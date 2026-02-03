<script setup lang="ts">
import { ref, watch } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";

const props = defineProps<{
  currentPhoto?: string;
  photoType?: string;
  label?: string;
}>();

const emit = defineEmits<{
  select: [path: string];
  remove: [];
}>();

const photoSrc = ref<string | null>(null);
const isDragOver = ref(false);
const loading = ref(false);

async function loadPhoto() {
  if (!props.currentPhoto || !props.photoType) {
    photoSrc.value = null;
    return;
  }

  loading.value = true;
  try {
    const base64 = await invoke<string>("get_photo_base64", {
      photoType: props.photoType,
      filename: props.currentPhoto,
    });
    photoSrc.value = base64;
  } catch (e) {
    console.error("Failed to load photo:", e);
    photoSrc.value = null;
  } finally {
    loading.value = false;
  }
}

watch(
  () => props.currentPhoto,
  () => loadPhoto(),
  { immediate: true }
);

async function selectFile() {
  const selected = await open({
    multiple: false,
    filters: [
      {
        name: "Images",
        extensions: ["png", "jpg", "jpeg", "gif", "webp"],
      },
    ],
  });

  if (selected && typeof selected === "string") {
    emit("select", selected);
  }
}

function handleDragOver(event: DragEvent) {
  event.preventDefault();
  isDragOver.value = true;
}

function handleDragLeave() {
  isDragOver.value = false;
}

function handleDrop(event: DragEvent) {
  event.preventDefault();
  isDragOver.value = false;

  const files = event.dataTransfer?.files;
  if (files && files.length > 0) {
    const file = files[0];
    if (file.type.startsWith("image/")) {
      // In Tauri, we'd need to handle this differently
      // For now, we'll use the file dialog
      selectFile();
    }
  }
}
</script>

<template>
  <div>
    <label v-if="label" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
      {{ label }}
    </label>

    <div v-if="currentPhoto" class="relative inline-block">
      <div class="max-w-48 rounded-lg overflow-hidden bg-gray-100 dark:bg-gray-700">
        <div v-if="loading" class="w-32 h-32 flex items-center justify-center">
          <div class="animate-spin rounded-full h-6 w-6 border-b-2 border-primary-600"></div>
        </div>
        <img
          v-else-if="photoSrc"
          :src="photoSrc"
          class="max-w-full h-auto"
          alt="Photo"
        />
        <div v-else class="w-32 h-32 flex items-center justify-center text-gray-400">
          <svg class="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
          </svg>
        </div>
      </div>
      <div class="absolute -top-2 -right-2 flex gap-1">
        <button
          @click="selectFile"
          class="p-1 bg-white dark:bg-gray-600 rounded-full shadow hover:bg-gray-100 dark:hover:bg-gray-500 border border-gray-200 dark:border-gray-500"
          title="Change photo"
        >
          <svg class="w-4 h-4 text-gray-600 dark:text-gray-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
          </svg>
        </button>
        <button
          @click="emit('remove')"
          class="p-1 bg-white dark:bg-gray-600 rounded-full shadow hover:bg-red-50 dark:hover:bg-gray-500 border border-gray-200 dark:border-gray-500"
          title="Remove photo"
        >
          <svg class="w-4 h-4 text-red-600 dark:text-gray-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>
    </div>

    <div
      v-else
      @click="selectFile"
      @dragover="handleDragOver"
      @dragleave="handleDragLeave"
      @drop="handleDrop"
      class="border-2 border-dashed rounded-lg p-6 text-center cursor-pointer transition-colors"
      :class="
        isDragOver
          ? 'border-primary-500 bg-primary-50 dark:bg-primary-900/30'
          : 'border-gray-300 dark:border-gray-600 hover:border-gray-400 dark:hover:border-gray-500'
      "
    >
      <svg class="w-10 h-10 mx-auto text-gray-400 mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
      </svg>
      <p class="text-sm text-gray-600 dark:text-gray-400">
        Click to select or drag and drop
      </p>
      <p class="text-xs text-gray-400 mt-1">
        PNG, JPG, GIF up to 10MB
      </p>
    </div>
  </div>
</template>
