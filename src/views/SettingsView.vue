<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useSettingsStore } from "../stores/settings";
import { open } from "@tauri-apps/plugin-dialog";

const settingsStore = useSettingsStore();
const saving = ref(false);
const saved = ref(false);

const formData = ref({
  church_name: "",
  theme: "system" as "system" | "light" | "dark",
  default_layout: "grid" as "grid" | "list",
  page_size: "letter" as "letter" | "a4",
  include_photos: true,
  include_contact_info: true,
  include_address: true,
});

onMounted(async () => {
  await settingsStore.fetchSettings();
  formData.value = {
    church_name: settingsStore.settings.church_name,
    theme: settingsStore.settings.theme || "system",
    default_layout: settingsStore.settings.default_layout as "grid" | "list",
    page_size: settingsStore.settings.page_size as "letter" | "a4",
    include_photos: settingsStore.settings.include_photos,
    include_contact_info: settingsStore.settings.include_contact_info,
    include_address: settingsStore.settings.include_address,
  };
});

async function saveSettings() {
  saving.value = true;
  saved.value = false;
  try {
    await settingsStore.saveSettings(formData.value);
    saved.value = true;
    setTimeout(() => {
      saved.value = false;
    }, 3000);
  } finally {
    saving.value = false;
  }
}

async function selectLogo() {
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
    await settingsStore.setChurchLogo(selected);
  }
}
</script>

<template>
  <div class="p-6 h-full overflow-y-auto">
    <h1 class="text-2xl font-bold text-gray-800 dark:text-gray-100 mb-6">Settings</h1>

    <div class="bg-white dark:bg-gray-800 rounded-lg shadow-sm p-6">
      <form @submit.prevent="saveSettings" class="space-y-6">
        <!-- Church Information -->
        <div>
          <h2 class="text-lg font-semibold text-gray-800 dark:text-gray-200 mb-4">Church Information</h2>

          <div class="space-y-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                Church Name
              </label>
              <input
                v-model="formData.church_name"
                type="text"
                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500 bg-white dark:bg-gray-700 dark:text-gray-100"
                placeholder="Enter church name"
              />
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                Church Logo
              </label>
              <div class="flex items-center gap-4">
                <div
                  v-if="settingsStore.settings.church_logo_path"
                  class="w-16 h-16 bg-gray-100 dark:bg-gray-700 rounded-lg"
                >
                  <!-- Logo preview would go here -->
                </div>
                <button
                  type="button"
                  @click="selectLogo"
                  class="px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 dark:text-gray-300"
                >
                  {{ settingsStore.settings.church_logo_path ? "Change Logo" : "Select Logo" }}
                </button>
              </div>
            </div>
          </div>
        </div>

        <!-- Appearance -->
        <div class="border-t dark:border-gray-700 pt-6">
          <h2 class="text-lg font-semibold text-gray-800 dark:text-gray-200 mb-4">Appearance</h2>

          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Theme
            </label>
            <select
              v-model="formData.theme"
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500 bg-white dark:bg-gray-700 dark:text-gray-100"
            >
              <option value="system">System</option>
              <option value="light">Light</option>
              <option value="dark">Dark</option>
            </select>
            <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
              Choose how the app looks. System will match your device settings.
            </p>
          </div>
        </div>

        <!-- PDF Defaults -->
        <div class="border-t dark:border-gray-700 pt-6">
          <h2 class="text-lg font-semibold text-gray-800 dark:text-gray-200 mb-4">PDF Defaults</h2>

          <div class="space-y-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                Default Layout
              </label>
              <select
                v-model="formData.default_layout"
                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500 bg-white dark:bg-gray-700 dark:text-gray-100"
              >
                <option value="grid">Grid (2 columns)</option>
                <option value="list">List (1 column)</option>
              </select>
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                Page Size
              </label>
              <select
                v-model="formData.page_size"
                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500 bg-white dark:bg-gray-700 dark:text-gray-100"
              >
                <option value="letter">Letter (8.5" x 11")</option>
                <option value="a4">A4 (210mm x 297mm)</option>
              </select>
            </div>

            <div class="space-y-3">
              <label class="flex items-center gap-3">
                <input
                  v-model="formData.include_photos"
                  type="checkbox"
                  class="w-4 h-4 text-primary-600 border-gray-300 dark:border-gray-600 rounded focus:ring-primary-500"
                />
                <span class="text-sm text-gray-700 dark:text-gray-300">Include photos by default</span>
              </label>

              <label class="flex items-center gap-3">
                <input
                  v-model="formData.include_contact_info"
                  type="checkbox"
                  class="w-4 h-4 text-primary-600 border-gray-300 dark:border-gray-600 rounded focus:ring-primary-500"
                />
                <span class="text-sm text-gray-700 dark:text-gray-300">Include contact information by default</span>
              </label>

              <label class="flex items-center gap-3">
                <input
                  v-model="formData.include_address"
                  type="checkbox"
                  class="w-4 h-4 text-primary-600 border-gray-300 dark:border-gray-600 rounded focus:ring-primary-500"
                />
                <span class="text-sm text-gray-700 dark:text-gray-300">Include addresses by default</span>
              </label>
            </div>
          </div>
        </div>

        <!-- Submit -->
        <div class="flex items-center justify-between pt-4 border-t dark:border-gray-700">
          <div>
            <span v-if="saved" class="text-green-600 text-sm">Settings saved!</span>
          </div>
          <button
            type="submit"
            :disabled="saving"
            class="px-6 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
          >
            <span v-if="saving" class="animate-spin rounded-full h-4 w-4 border-2 border-white border-t-transparent"></span>
            {{ saving ? "Saving..." : "Save Settings" }}
          </button>
        </div>
      </form>
    </div>
  </div>
</template>
