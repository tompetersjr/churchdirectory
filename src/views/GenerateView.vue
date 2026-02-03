<script setup lang="ts">
import { ref } from "vue";
import PdfGenerator from "../components/PdfGenerator.vue";

const generationComplete = ref(false);
const outputPath = ref("");

function handleGenerationComplete(path: string) {
  outputPath.value = path;
  generationComplete.value = true;
}

function startNewGeneration() {
  generationComplete.value = false;
  outputPath.value = "";
}
</script>

<template>
  <div class="p-6 h-full overflow-y-auto">
    <h1 class="text-2xl font-bold text-gray-800 dark:text-gray-100 mb-6">Generate PDF Directory</h1>

    <div v-if="generationComplete" class="bg-white dark:bg-gray-800 rounded-lg shadow-sm p-6">
      <div class="text-center">
        <div class="w-16 h-16 bg-green-100 dark:bg-green-900/30 rounded-full flex items-center justify-center mx-auto mb-4">
          <svg class="w-8 h-8 text-green-600 dark:text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
        </div>
        <h2 class="text-xl font-semibold dark:text-gray-100 mb-2">PDF Generated Successfully!</h2>
        <p class="text-gray-600 dark:text-gray-400 mb-6 break-all">Saved to: {{ outputPath }}</p>
        <div class="flex justify-center gap-4">
          <button
            @click="startNewGeneration"
            class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700"
          >
            Generate Another
          </button>
          <router-link
            to="/"
            class="px-4 py-2 text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg"
          >
            Back to Directory
          </router-link>
        </div>
      </div>
    </div>

    <PdfGenerator v-else @complete="handleGenerationComplete" />
  </div>
</template>
