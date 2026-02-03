<script setup lang="ts">
defineProps<{
  show: boolean;
  title: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  danger?: boolean;
}>();

const emit = defineEmits<{
  confirm: [];
  cancel: [];
}>();
</script>

<template>
  <Teleport to="body">
    <div
      v-if="show"
      class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
      @click.self="emit('cancel')"
    >
      <div class="bg-white dark:bg-gray-800 rounded-lg p-6 max-w-md w-full mx-4 shadow-xl">
        <h3 class="text-lg font-semibold dark:text-gray-100 mb-2">{{ title }}</h3>
        <p class="text-gray-600 dark:text-gray-400 mb-6">{{ message }}</p>
        <div class="flex justify-end gap-3">
          <button
            @click="emit('cancel')"
            class="px-4 py-2 text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
          >
            {{ cancelText || "Cancel" }}
          </button>
          <button
            @click="emit('confirm')"
            class="px-4 py-2 rounded-lg transition-colors"
            :class="
              danger
                ? 'bg-red-600 text-white hover:bg-red-700'
                : 'bg-primary-600 text-white hover:bg-primary-700'
            "
          >
            {{ confirmText || "Confirm" }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
