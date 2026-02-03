<script setup lang="ts">
import { ref, watch } from "vue";
import type { Family } from "../../types";

const props = defineProps<{
  initialData: Partial<Family>;
  loading: boolean;
}>();

const emit = defineEmits<{
  submit: [data: Partial<Family>];
  cancel: [];
}>();

const formData = ref({
  family_id: "",
  name: "",
  mailing_name: "",
  address: "",
  city: "",
  state: "",
  zip: "",
  phone: "",
  email: "",
  notes: "",
});

watch(
  () => props.initialData,
  (newData) => {
    formData.value = {
      family_id: newData.family_id || "",
      name: newData.name || "",
      mailing_name: newData.mailing_name || "",
      address: newData.address || "",
      city: newData.city || "",
      state: newData.state || "",
      zip: newData.zip || "",
      phone: newData.phone || "",
      email: newData.email || "",
      notes: newData.notes || "",
    };
  },
  { immediate: true }
);

function handleSubmit() {
  emit("submit", { ...formData.value });
}
</script>

<template>
  <form @submit.prevent="handleSubmit" class="space-y-6">
    <div class="grid grid-cols-2 gap-4">
      <div>
        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
          Family ID <span class="text-red-500">*</span>
        </label>
        <input
          v-model="formData.family_id"
          type="text"
          required
          class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500 bg-white dark:bg-gray-700 dark:text-gray-100"
          placeholder="e.g., F001"
        />
      </div>
      <div>
        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
          Family Name <span class="text-red-500">*</span>
        </label>
        <input
          v-model="formData.name"
          type="text"
          required
          class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500 bg-white dark:bg-gray-700 dark:text-gray-100"
          placeholder="e.g., Smith"
        />
      </div>
    </div>

    <div>
      <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Mailing Name</label>
      <input
        v-model="formData.mailing_name"
        type="text"
        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500 bg-white dark:bg-gray-700 dark:text-gray-100"
        placeholder="e.g., John & Jane Smith"
      />
    </div>

    <div>
      <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Address</label>
      <input
        v-model="formData.address"
        type="text"
        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500 bg-white dark:bg-gray-700 dark:text-gray-100"
        placeholder="Street address"
      />
    </div>

    <div class="grid grid-cols-3 gap-4">
      <div>
        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">City</label>
        <input
          v-model="formData.city"
          type="text"
          class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500 bg-white dark:bg-gray-700 dark:text-gray-100"
        />
      </div>
      <div>
        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">State</label>
        <input
          v-model="formData.state"
          type="text"
          class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500 bg-white dark:bg-gray-700 dark:text-gray-100"
        />
      </div>
      <div>
        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">ZIP</label>
        <input
          v-model="formData.zip"
          type="text"
          class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500 bg-white dark:bg-gray-700 dark:text-gray-100"
        />
      </div>
    </div>

    <div class="grid grid-cols-2 gap-4">
      <div>
        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Phone</label>
        <input
          v-model="formData.phone"
          type="tel"
          class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500 bg-white dark:bg-gray-700 dark:text-gray-100"
          placeholder="(555) 123-4567"
        />
      </div>
      <div>
        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Email</label>
        <input
          v-model="formData.email"
          type="email"
          class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500 bg-white dark:bg-gray-700 dark:text-gray-100"
          placeholder="family@example.com"
        />
      </div>
    </div>

    <div>
      <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Notes</label>
      <textarea
        v-model="formData.notes"
        rows="3"
        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500 bg-white dark:bg-gray-700 dark:text-gray-100"
        placeholder="Additional notes..."
      ></textarea>
    </div>

    <div class="flex justify-end gap-3 pt-4 border-t dark:border-gray-700">
      <button
        type="button"
        @click="emit('cancel')"
        class="px-4 py-2 text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg"
      >
        Cancel
      </button>
      <button
        type="submit"
        :disabled="loading"
        class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
      >
        <span
          v-if="loading"
          class="animate-spin rounded-full h-4 w-4 border-2 border-white border-t-transparent"
        ></span>
        {{ loading ? "Saving..." : "Save Family" }}
      </button>
    </div>
  </form>
</template>
