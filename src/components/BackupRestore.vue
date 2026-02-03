<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import type { BackupManifest } from "../types";

const activeTab = ref<"backup" | "restore">("backup");
const loading = ref(false);
const error = ref<string | null>(null);
const success = ref<string | null>(null);

const backupResult = ref<BackupManifest | null>(null);
const restorePreview = ref<BackupManifest | null>(null);
const restoreFilePath = ref("");
const replaceExisting = ref(true);

async function createBackup() {
  const outputPath = await save({
    defaultPath: `church-directory-backup-${new Date().toISOString().split("T")[0]}.zip`,
    filters: [
      {
        name: "ZIP Files",
        extensions: ["zip"],
      },
    ],
  });

  if (!outputPath) return;

  loading.value = true;
  error.value = null;
  success.value = null;

  try {
    backupResult.value = await invoke<BackupManifest>("create_backup", {
      outputPath,
    });
    success.value = `Backup created successfully at ${outputPath}`;
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

async function selectRestoreFile() {
  const selected = await open({
    multiple: false,
    filters: [
      {
        name: "ZIP Files",
        extensions: ["zip"],
      },
    ],
  });

  if (selected && typeof selected === "string") {
    restoreFilePath.value = selected;
    error.value = null;
    success.value = null;

    loading.value = true;
    try {
      restorePreview.value = await invoke<BackupManifest>("preview_restore", {
        backupPath: selected,
      });
    } catch (e) {
      error.value = String(e);
      restorePreview.value = null;
    } finally {
      loading.value = false;
    }
  }
}

async function executeRestore() {
  if (!restoreFilePath.value) return;

  loading.value = true;
  error.value = null;
  success.value = null;

  try {
    await invoke("restore_backup", {
      backupPath: restoreFilePath.value,
      replaceExisting: replaceExisting.value,
    });
    success.value = "Restore completed successfully!";
    restoreFilePath.value = "";
    restorePreview.value = null;
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

function formatDate(dateString: string) {
  return new Date(dateString).toLocaleString();
}
</script>

<template>
  <div class="bg-white dark:bg-gray-800 rounded-lg shadow-sm">
    <!-- Tabs -->
    <div class="flex border-b dark:border-gray-700">
      <button
        @click="activeTab = 'backup'"
        class="flex-1 px-4 py-3 text-sm font-medium transition-colors"
        :class="
          activeTab === 'backup'
            ? 'text-primary-600 dark:text-primary-400 border-b-2 border-primary-600 dark:border-primary-400'
            : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300'
        "
      >
        Create Backup
      </button>
      <button
        @click="activeTab = 'restore'"
        class="flex-1 px-4 py-3 text-sm font-medium transition-colors"
        :class="
          activeTab === 'restore'
            ? 'text-primary-600 dark:text-primary-400 border-b-2 border-primary-600 dark:border-primary-400'
            : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300'
        "
      >
        Restore Backup
      </button>
    </div>

    <div class="p-6">
      <!-- Backup Tab -->
      <div v-if="activeTab === 'backup'">
        <p class="text-gray-600 dark:text-gray-400 mb-6">
          Create a backup of your entire directory, including all families, members, photos, and settings.
        </p>

        <div v-if="backupResult" class="bg-green-50 dark:bg-green-900/30 rounded-lg p-4 mb-6">
          <h3 class="font-medium text-green-800 dark:text-green-300 mb-2">Backup Created</h3>
          <dl class="grid grid-cols-2 gap-2 text-sm">
            <dt class="text-green-700 dark:text-green-400">Families:</dt>
            <dd class="text-green-900 dark:text-green-200">{{ backupResult.family_count }}</dd>
            <dt class="text-green-700 dark:text-green-400">Members:</dt>
            <dd class="text-green-900 dark:text-green-200">{{ backupResult.member_count }}</dd>
            <dt class="text-green-700 dark:text-green-400">Photos:</dt>
            <dd class="text-green-900 dark:text-green-200">{{ backupResult.photo_count }}</dd>
            <dt class="text-green-700 dark:text-green-400">Created:</dt>
            <dd class="text-green-900 dark:text-green-200">{{ formatDate(backupResult.created_at) }}</dd>
          </dl>
        </div>

        <button
          @click="createBackup"
          :disabled="loading"
          class="w-full px-6 py-3 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
        >
          <span
            v-if="loading"
            class="animate-spin rounded-full h-5 w-5 border-2 border-white border-t-transparent"
          ></span>
          <svg v-else class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 8h14M5 8a2 2 0 110-4h14a2 2 0 110 4M5 8v10a2 2 0 002 2h10a2 2 0 002-2V8m-9 4h4" />
          </svg>
          {{ loading ? "Creating Backup..." : "Create Backup" }}
        </button>
      </div>

      <!-- Restore Tab -->
      <div v-else>
        <p class="text-gray-600 dark:text-gray-400 mb-6">
          Restore your directory from a previous backup file.
        </p>

        <div
          @click="selectRestoreFile"
          class="border-2 border-dashed border-gray-300 dark:border-gray-600 rounded-lg p-6 text-center cursor-pointer hover:border-gray-400 dark:hover:border-gray-500 transition-colors mb-6"
        >
          <svg class="w-10 h-10 mx-auto text-gray-400 mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
          </svg>
          <p v-if="restoreFilePath" class="font-medium text-primary-600 dark:text-primary-400">
            {{ restoreFilePath.split(/[/\\]/).pop() }}
          </p>
          <p v-else class="text-gray-600 dark:text-gray-400">Click to select a backup file</p>
        </div>

        <div v-if="restorePreview" class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4 mb-6">
          <h3 class="font-medium dark:text-gray-200 mb-2">Backup Contents</h3>
          <dl class="grid grid-cols-2 gap-2 text-sm">
            <dt class="text-gray-500 dark:text-gray-400">Version:</dt>
            <dd class="dark:text-gray-200">{{ restorePreview.app_version }}</dd>
            <dt class="text-gray-500 dark:text-gray-400">Families:</dt>
            <dd class="dark:text-gray-200">{{ restorePreview.family_count }}</dd>
            <dt class="text-gray-500 dark:text-gray-400">Members:</dt>
            <dd class="dark:text-gray-200">{{ restorePreview.member_count }}</dd>
            <dt class="text-gray-500 dark:text-gray-400">Photos:</dt>
            <dd class="dark:text-gray-200">{{ restorePreview.photo_count }}</dd>
            <dt class="text-gray-500 dark:text-gray-400">Created:</dt>
            <dd class="dark:text-gray-200">{{ formatDate(restorePreview.created_at) }}</dd>
          </dl>

          <div class="mt-4 pt-4 border-t dark:border-gray-600">
            <label class="flex items-center gap-3">
              <input
                v-model="replaceExisting"
                type="checkbox"
                class="w-4 h-4 text-primary-600 border-gray-300 dark:border-gray-600 rounded focus:ring-primary-500"
              />
              <span class="text-sm text-gray-700 dark:text-gray-300">Replace existing data (uncheck to merge)</span>
            </label>
          </div>
        </div>

        <button
          @click="executeRestore"
          :disabled="loading || !restorePreview"
          class="w-full px-6 py-3 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
        >
          <span
            v-if="loading"
            class="animate-spin rounded-full h-5 w-5 border-2 border-white border-t-transparent"
          ></span>
          <svg v-else class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
          {{ loading ? "Restoring..." : "Restore Backup" }}
        </button>
      </div>

      <!-- Messages -->
      <div v-if="error" class="mt-4 p-4 bg-red-50 dark:bg-red-900/30 text-red-700 dark:text-red-400 rounded-lg">
        {{ error }}
      </div>
      <div v-if="success" class="mt-4 p-4 bg-green-50 dark:bg-green-900/30 text-green-700 dark:text-green-400 rounded-lg">
        {{ success }}
      </div>
    </div>
  </div>
</template>
