<script setup lang="ts">
import { ref, watch } from "vue";
import type { Member } from "../../types";

const props = defineProps<{
  initialData: Partial<Member>;
  loading: boolean;
}>();

const emit = defineEmits<{
  submit: [data: Partial<Member>];
  cancel: [];
}>();

const formData = ref({
  first_name: "",
  last_name: "",
  role: "",
  birth_date: "",
  wedding_date: "",
  phone: "",
  email: "",
  notes: "",
  sort_order: 0,
});

watch(
  () => props.initialData,
  (newData) => {
    formData.value = {
      first_name: newData.first_name || "",
      last_name: newData.last_name || "",
      role: newData.role || "",
      birth_date: newData.birth_date || "",
      wedding_date: newData.wedding_date || "",
      phone: newData.phone || "",
      email: newData.email || "",
      notes: newData.notes || "",
      sort_order: newData.sort_order || 0,
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
        <label class="block text-sm font-medium text-gray-700 mb-1">
          First Name <span class="text-red-500">*</span>
        </label>
        <input
          v-model="formData.first_name"
          type="text"
          required
          class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
        />
      </div>
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">
          Last Name <span class="text-red-500">*</span>
        </label>
        <input
          v-model="formData.last_name"
          type="text"
          required
          class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
        />
      </div>
    </div>

    <div class="grid grid-cols-2 gap-4">
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">Role</label>
        <select
          v-model="formData.role"
          class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
        >
          <option value="">Select role...</option>
          <option value="Head of Household">Head of Household</option>
          <option value="Spouse">Spouse</option>
          <option value="Child">Child</option>
          <option value="Other">Other</option>
        </select>
      </div>
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">Birth Date</label>
        <input
          v-model="formData.birth_date"
          type="text"
          placeholder="MM-DD or YYYY-MM-DD"
          class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
        />
        <p class="text-xs text-gray-500 mt-1">Use MM-DD for month/day only</p>
      </div>
    </div>

    <div>
      <label class="block text-sm font-medium text-gray-700 mb-1">Wedding Date / Anniversary</label>
      <input
        v-model="formData.wedding_date"
        type="text"
        placeholder="MM-DD or YYYY-MM-DD"
        class="w-48 px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
      />
      <p class="text-xs text-gray-500 mt-1">Use MM-DD for month/day only</p>
    </div>

    <div class="grid grid-cols-2 gap-4">
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">Phone</label>
        <input
          v-model="formData.phone"
          type="tel"
          class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
          placeholder="(555) 123-4567"
        />
      </div>
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">Email</label>
        <input
          v-model="formData.email"
          type="email"
          class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
          placeholder="name@example.com"
        />
      </div>
    </div>

    <div>
      <label class="block text-sm font-medium text-gray-700 mb-1">Sort Order</label>
      <input
        v-model.number="formData.sort_order"
        type="number"
        min="0"
        class="w-24 px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
      />
      <p class="text-xs text-gray-500 mt-1">Lower numbers appear first</p>
    </div>

    <div>
      <label class="block text-sm font-medium text-gray-700 mb-1">Notes</label>
      <textarea
        v-model="formData.notes"
        rows="3"
        class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
        placeholder="Additional notes..."
      ></textarea>
    </div>

    <div class="flex justify-end gap-3 pt-4 border-t">
      <button
        type="button"
        @click="emit('cancel')"
        class="px-4 py-2 text-gray-600 hover:bg-gray-100 rounded-lg"
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
        {{ loading ? "Saving..." : "Save Member" }}
      </button>
    </div>
  </form>
</template>
