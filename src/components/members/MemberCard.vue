<script setup lang="ts">
import { ref, watch, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Member } from "../../types";

const props = defineProps<{
  member: Member;
  hasFamilyPhoto?: boolean;
}>();

const emit = defineEmits<{
  edit: [id: number];
  delete: [id: number];
  cropFromFamily: [id: number];
}>();

const photoSrc = ref<string | null>(null);

async function loadPhoto() {
  if (!props.member.photo_path) {
    photoSrc.value = null;
    return;
  }
  try {
    const base64 = await invoke<string>("get_photo_base64", {
      photoType: "members",
      filename: props.member.photo_path,
    });
    photoSrc.value = base64;
  } catch (e) {
    console.error("Failed to load member photo:", e);
    photoSrc.value = null;
  }
}

watch(() => props.member.photo_path, loadPhoto, { immediate: true });

// Format date for display - handles both full dates and month-day only
function formatDate(dateStr: string): string {
  if (!dateStr) return "";

  // Check if it's just month-day (MM-DD format)
  if (/^\d{2}-\d{2}$/.test(dateStr)) {
    const [month, day] = dateStr.split("-").map(Number);
    const monthNames = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    return `${monthNames[month - 1]} ${day}`;
  }

  // Full date (YYYY-MM-DD format)
  try {
    const date = new Date(dateStr + "T00:00:00");
    return date.toLocaleDateString("en-US", { month: "short", day: "numeric", year: "numeric" });
  } catch {
    return dateStr;
  }
}

const formattedBirthDate = computed(() => props.member.birth_date ? formatDate(props.member.birth_date) : null);
const formattedWeddingDate = computed(() => props.member.wedding_date ? formatDate(props.member.wedding_date) : null);
</script>

<template>
  <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4 flex flex-col h-full">
    <!-- Top section: Photo and Info -->
    <div class="flex items-start gap-4">
      <!-- Member Photo with change button -->
      <div class="relative flex-shrink-0">
        <div class="w-16 h-16 bg-gray-200 dark:bg-gray-600 rounded-full flex items-center justify-center overflow-hidden">
          <img
            v-if="photoSrc"
            :src="photoSrc"
            class="w-full h-full object-cover"
            alt="Member photo"
          />
          <svg v-else class="w-8 h-8 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
          </svg>
        </div>
        <!-- Change avatar button -->
        <button
          v-if="hasFamilyPhoto"
          @click="emit('cropFromFamily', member.id)"
          class="absolute -bottom-1 -right-1 p-1 bg-white dark:bg-gray-600 rounded-full shadow hover:bg-gray-100 dark:hover:bg-gray-500 border border-gray-200 dark:border-gray-500"
          title="Set avatar from family photo"
        >
          <svg class="w-3.5 h-3.5 text-gray-600 dark:text-gray-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 9a2 2 0 012-2h.93a2 2 0 001.664-.89l.812-1.22A2 2 0 0110.07 4h3.86a2 2 0 011.664.89l.812 1.22A2 2 0 0018.07 7H19a2 2 0 012 2v9a2 2 0 01-2 2H5a2 2 0 01-2-2V9z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 13a3 3 0 11-6 0 3 3 0 016 0z" />
          </svg>
        </button>
      </div>

      <!-- Member Info -->
      <div class="flex-1 min-w-0">
        <h3 class="font-medium text-gray-900 dark:text-gray-100 text-lg">
          {{ member.first_name }} {{ member.last_name }}
        </h3>
        <p v-if="member.phone" class="text-sm text-gray-600 dark:text-gray-400 mt-0.5">{{ member.phone }}</p>
        <p v-if="member.role" class="text-sm text-gray-500 dark:text-gray-400 mt-1">{{ member.role }}</p>
      </div>
    </div>

    <!-- Email - Full width -->
    <p v-if="member.email" class="text-sm text-primary-600 dark:text-primary-400 mt-2 break-all">{{ member.email }}</p>

    <!-- Dates -->
    <div v-if="formattedBirthDate || formattedWeddingDate" class="mt-2 text-xs text-gray-500 dark:text-gray-400 space-y-0.5">
      <p v-if="formattedBirthDate">
        <span class="font-medium">Birthday:</span> {{ formattedBirthDate }}
      </p>
      <p v-if="formattedWeddingDate">
        <span class="font-medium">Anniversary:</span> {{ formattedWeddingDate }}
      </p>
    </div>

    <!-- Spacer to push footer to bottom -->
    <div class="flex-grow"></div>

    <!-- Bottom section: Action buttons -->
    <div class="flex justify-end gap-2 mt-3 pt-3 border-t border-gray-200 dark:border-gray-600">
      <button
        @click="emit('delete', member.id)"
        class="flex items-center gap-1.5 px-3 py-1.5 text-sm text-red-600 dark:text-red-400 bg-white dark:bg-gray-800 border border-red-200 dark:border-red-800 rounded-md hover:bg-red-50 dark:hover:bg-red-900/30 hover:border-red-300 dark:hover:border-red-700 transition-colors"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
        </svg>
        Delete
      </button>
      <button
        @click="emit('edit', member.id)"
        class="flex items-center gap-1.5 px-3 py-1.5 text-sm text-primary-600 dark:text-primary-400 bg-white dark:bg-gray-800 border border-primary-200 dark:border-primary-800 rounded-md hover:bg-primary-50 dark:hover:bg-primary-900/30 hover:border-primary-300 dark:hover:border-primary-700 transition-colors"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
        </svg>
        Edit
      </button>
    </div>
  </div>
</template>
