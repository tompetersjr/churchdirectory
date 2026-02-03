<script setup lang="ts">
import { ref } from "vue";
import ImportWizard from "../components/ImportWizard.vue";

const importComplete = ref(false);
const importResult = ref<any>(null);

function handleImportComplete(result: any) {
  importResult.value = result;
  importComplete.value = true;
}

function startNewImport() {
  importComplete.value = false;
  importResult.value = null;
}
</script>

<template>
  <div class="p-6 h-full overflow-y-auto">
    <h1 class="text-2xl font-bold text-gray-800 dark:text-gray-100 mb-6">Import Data</h1>

    <div v-if="importComplete && importResult" class="bg-white dark:bg-gray-800 rounded-lg shadow-sm p-6">
      <div class="text-center">
        <div class="w-16 h-16 bg-green-100 dark:bg-green-900/30 rounded-full flex items-center justify-center mx-auto mb-4">
          <svg class="w-8 h-8 text-green-600 dark:text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
        </div>
        <h2 class="text-xl font-semibold dark:text-gray-100 mb-4">Import Complete!</h2>
        <div class="text-left bg-gray-50 dark:bg-gray-700 rounded-lg p-4 mb-6">
          <div class="grid grid-cols-2 gap-4 text-sm">
            <div>
              <span class="text-gray-500 dark:text-gray-400">Families Created:</span>
              <span class="ml-2 font-medium dark:text-gray-200">{{ importResult.families_created }}</span>
            </div>
            <div>
              <span class="text-gray-500 dark:text-gray-400">Families Updated:</span>
              <span class="ml-2 font-medium dark:text-gray-200">{{ importResult.families_updated }}</span>
            </div>
            <div>
              <span class="text-gray-500 dark:text-gray-400">Members Created:</span>
              <span class="ml-2 font-medium dark:text-gray-200">{{ importResult.members_created }}</span>
            </div>
            <div>
              <span class="text-gray-500 dark:text-gray-400">Members Updated:</span>
              <span class="ml-2 font-medium dark:text-gray-200">{{ importResult.members_updated }}</span>
            </div>
          </div>
          <div v-if="importResult.errors?.length" class="mt-4">
            <p class="text-red-600 dark:text-red-400 font-medium mb-2">Errors:</p>
            <ul class="text-sm text-red-600 dark:text-red-400 space-y-1">
              <li v-for="(error, index) in importResult.errors" :key="index">
                {{ error }}
              </li>
            </ul>
          </div>
        </div>
        <div class="flex justify-center gap-4">
          <router-link
            to="/"
            class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700"
          >
            View Directory
          </router-link>
          <button
            @click="startNewImport"
            class="px-4 py-2 text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg"
          >
            Import Another File
          </button>
        </div>
      </div>
    </div>

    <ImportWizard v-else @complete="handleImportComplete" />
  </div>
</template>
