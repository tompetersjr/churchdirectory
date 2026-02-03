<script setup lang="ts">
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { ImportPreview, ImportResult } from "../types";

const emit = defineEmits<{
  complete: [result: ImportResult];
}>();

const step = ref(1);
const filePath = ref("");
const fileName = ref("");
const loading = ref(false);
const error = ref<string | null>(null);
const preview = ref<ImportPreview | null>(null);
const updateDuplicates = ref(true);

const totalSteps = 4;

async function selectFile() {
  const selected = await open({
    multiple: false,
    filters: [
      {
        name: "Excel Files",
        extensions: ["xlsx", "xls"],
      },
    ],
  });

  if (selected && typeof selected === "string") {
    filePath.value = selected;
    fileName.value = selected.split(/[/\\]/).pop() || selected;
    error.value = null;
  }
}

async function loadPreview() {
  if (!filePath.value) return;

  loading.value = true;
  error.value = null;

  try {
    preview.value = await invoke<ImportPreview>("preview_import", {
      filePath: filePath.value,
    });
    step.value = 2;
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

async function executeImport() {
  if (!filePath.value) return;

  loading.value = true;
  error.value = null;

  try {
    const result = await invoke<ImportResult>("execute_import", {
      filePath: filePath.value,
      updateDuplicates: updateDuplicates.value,
    });
    emit("complete", result);
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

function goBack() {
  if (step.value > 1) {
    step.value--;
  }
}

function goNext() {
  if (step.value === 1) {
    loadPreview();
  } else if (step.value === 2) {
    step.value = 3;
  } else if (step.value === 3) {
    step.value = 4;
  } else if (step.value === 4) {
    executeImport();
  }
}

const canProceed = computed(() => {
  if (step.value === 1) return !!filePath.value;
  if (step.value === 2) return !!preview.value;
  if (step.value === 3) return true;
  if (step.value === 4) return true;
  return false;
});
</script>

<template>
  <div class="bg-white dark:bg-gray-800 rounded-lg shadow-sm p-6">
    <!-- Progress Steps -->
    <div class="flex items-center justify-between mb-8">
      <div
        v-for="s in totalSteps"
        :key="s"
        class="flex items-center"
        :class="{ 'flex-1': s < totalSteps }"
      >
        <div
          class="w-8 h-8 rounded-full flex items-center justify-center text-sm font-medium"
          :class="
            s <= step
              ? 'bg-primary-600 text-white'
              : 'bg-gray-200 dark:bg-gray-700 text-gray-500 dark:text-gray-400'
          "
        >
          {{ s }}
        </div>
        <div
          v-if="s < totalSteps"
          class="flex-1 h-1 mx-2"
          :class="s < step ? 'bg-primary-600' : 'bg-gray-200 dark:bg-gray-700'"
        ></div>
      </div>
    </div>

    <!-- Step 1: File Selection -->
    <div v-if="step === 1">
      <h2 class="text-lg font-semibold dark:text-gray-100 mb-4">Select File</h2>
      <p class="text-gray-600 dark:text-gray-400 mb-6">
        Choose an Excel file (.xlsx) containing your directory data. The file should have columns for Family ID, Family Name, First Name, Last Name, and optionally Address, Phone, Email, and Role.
      </p>

      <div
        @click="selectFile"
        class="border-2 border-dashed border-gray-300 dark:border-gray-600 rounded-lg p-8 text-center cursor-pointer hover:border-gray-400 dark:hover:border-gray-500 transition-colors"
      >
        <svg class="w-12 h-12 mx-auto text-gray-400 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
        </svg>
        <p v-if="fileName" class="font-medium text-primary-600 dark:text-primary-400">{{ fileName }}</p>
        <p v-else class="text-gray-600 dark:text-gray-400">Click to select an Excel file</p>
      </div>
    </div>

    <!-- Step 2: Preview -->
    <div v-else-if="step === 2 && preview">
      <h2 class="text-lg font-semibold dark:text-gray-100 mb-4">Preview Import</h2>

      <div class="grid grid-cols-2 gap-4 mb-6">
        <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
          <p class="text-2xl font-bold text-primary-600 dark:text-primary-400">{{ preview.total_families }}</p>
          <p class="text-sm text-gray-600 dark:text-gray-400">Families Found</p>
        </div>
        <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
          <p class="text-2xl font-bold text-primary-600 dark:text-primary-400">{{ preview.total_members }}</p>
          <p class="text-sm text-gray-600 dark:text-gray-400">Members Found</p>
        </div>
      </div>

      <div class="max-h-64 overflow-y-auto border dark:border-gray-700 rounded-lg">
        <table class="w-full text-sm">
          <thead class="bg-gray-50 dark:bg-gray-700 sticky top-0">
            <tr>
              <th class="px-3 py-2 text-left dark:text-gray-300">Family ID</th>
              <th class="px-3 py-2 text-left dark:text-gray-300">Name</th>
              <th class="px-3 py-2 text-left dark:text-gray-300">Members</th>
              <th class="px-3 py-2 text-left dark:text-gray-300">Status</th>
            </tr>
          </thead>
          <tbody class="divide-y dark:divide-gray-700">
            <tr v-for="family in preview.families" :key="family.family_id" class="dark:text-gray-300">
              <td class="px-3 py-2">{{ family.family_id }}</td>
              <td class="px-3 py-2">{{ family.name }}</td>
              <td class="px-3 py-2">{{ family.members.length }}</td>
              <td class="px-3 py-2">
                <span
                  v-if="family.is_duplicate"
                  class="px-2 py-1 text-xs rounded-full bg-yellow-100 dark:bg-yellow-900/30 text-yellow-700 dark:text-yellow-400"
                >
                  Duplicate
                </span>
                <span
                  v-else
                  class="px-2 py-1 text-xs rounded-full bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400"
                >
                  New
                </span>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- Step 3: Duplicate Handling -->
    <div v-else-if="step === 3 && preview">
      <h2 class="text-lg font-semibold dark:text-gray-100 mb-4">Handle Duplicates</h2>

      <div v-if="preview.duplicates.length === 0" class="text-center py-8">
        <svg class="w-12 h-12 mx-auto text-green-500 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        <p class="text-gray-600 dark:text-gray-400">No duplicates found. All families are new.</p>
      </div>

      <div v-else>
        <p class="text-gray-600 dark:text-gray-400 mb-4">
          {{ preview.duplicates.length }} duplicate(s) found. Choose how to handle them:
        </p>

        <div class="space-y-3 mb-6">
          <label class="flex items-start gap-3 p-4 border dark:border-gray-700 rounded-lg cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-700">
            <input
              v-model="updateDuplicates"
              type="radio"
              :value="true"
              class="mt-1"
            />
            <div>
              <p class="font-medium dark:text-gray-200">Update existing records</p>
              <p class="text-sm text-gray-500 dark:text-gray-400">Merge imported data with existing families</p>
            </div>
          </label>
          <label class="flex items-start gap-3 p-4 border dark:border-gray-700 rounded-lg cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-700">
            <input
              v-model="updateDuplicates"
              type="radio"
              :value="false"
              class="mt-1"
            />
            <div>
              <p class="font-medium dark:text-gray-200">Skip duplicates</p>
              <p class="text-sm text-gray-500 dark:text-gray-400">Only import new families</p>
            </div>
          </label>
        </div>

        <div class="border dark:border-gray-700 rounded-lg overflow-hidden">
          <table class="w-full text-sm">
            <thead class="bg-gray-50 dark:bg-gray-700">
              <tr>
                <th class="px-3 py-2 text-left dark:text-gray-300">Import</th>
                <th class="px-3 py-2 text-left dark:text-gray-300">Existing</th>
                <th class="px-3 py-2 text-left dark:text-gray-300">Match Type</th>
              </tr>
            </thead>
            <tbody class="divide-y dark:divide-gray-700">
              <tr v-for="dup in preview.duplicates" :key="dup.import_family_id" class="dark:text-gray-300">
                <td class="px-3 py-2">
                  {{ dup.import_family_id }} - {{ dup.import_name }}
                </td>
                <td class="px-3 py-2">{{ dup.existing_name }}</td>
                <td class="px-3 py-2">
                  <span class="px-2 py-1 text-xs rounded-full bg-gray-100 dark:bg-gray-600 dark:text-gray-300">
                    {{ dup.match_type === "id" ? "Family ID" : "Name" }}
                  </span>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>

    <!-- Step 4: Confirm -->
    <div v-else-if="step === 4 && preview">
      <h2 class="text-lg font-semibold dark:text-gray-100 mb-4">Confirm Import</h2>

      <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-6 mb-6">
        <h3 class="font-medium dark:text-gray-200 mb-4">Import Summary</h3>
        <dl class="grid grid-cols-2 gap-4 text-sm">
          <div>
            <dt class="text-gray-500 dark:text-gray-400">File</dt>
            <dd class="font-medium dark:text-gray-200">{{ fileName }}</dd>
          </div>
          <div>
            <dt class="text-gray-500 dark:text-gray-400">Families</dt>
            <dd class="font-medium dark:text-gray-200">{{ preview.total_families }}</dd>
          </div>
          <div>
            <dt class="text-gray-500 dark:text-gray-400">Members</dt>
            <dd class="font-medium dark:text-gray-200">{{ preview.total_members }}</dd>
          </div>
          <div>
            <dt class="text-gray-500 dark:text-gray-400">Duplicates</dt>
            <dd class="font-medium dark:text-gray-200">
              {{ preview.duplicates.length }}
              <span class="text-gray-500 dark:text-gray-400">
                ({{ updateDuplicates ? "will update" : "will skip" }})
              </span>
            </dd>
          </div>
        </dl>
      </div>

      <p class="text-gray-600 dark:text-gray-400">
        Click "Import" to proceed. This action cannot be undone.
      </p>
    </div>

    <!-- Error Message -->
    <div v-if="error" class="mt-4 p-4 bg-red-50 dark:bg-red-900/30 text-red-700 dark:text-red-400 rounded-lg">
      {{ error }}
    </div>

    <!-- Navigation -->
    <div class="flex justify-between mt-8 pt-4 border-t dark:border-gray-700">
      <button
        v-if="step > 1"
        @click="goBack"
        :disabled="loading"
        class="px-4 py-2 text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg disabled:opacity-50"
      >
        Back
      </button>
      <div v-else></div>

      <button
        @click="goNext"
        :disabled="!canProceed || loading"
        class="px-6 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
      >
        <span
          v-if="loading"
          class="animate-spin rounded-full h-4 w-4 border-2 border-white border-t-transparent"
        ></span>
        {{ step === 4 ? "Import" : "Continue" }}
      </button>
    </div>
  </div>
</template>
