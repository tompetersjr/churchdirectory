<script setup lang="ts">
import { ref, watch } from "vue";

const props = defineProps<{
  value: string;
  placeholder?: string;
}>();

const emit = defineEmits<{
  search: [query: string];
}>();

const localValue = ref(props.value);

watch(
  () => props.value,
  (newVal) => {
    localValue.value = newVal;
  }
);

function handleInput(event: Event) {
  const target = event.target as HTMLInputElement;
  localValue.value = target.value;
  emit("search", target.value);
}

function clear() {
  localValue.value = "";
  emit("search", "");
}
</script>

<template>
  <div class="relative">
    <svg
      class="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-400"
      fill="none"
      stroke="currentColor"
      viewBox="0 0 24 24"
    >
      <path
        stroke-linecap="round"
        stroke-linejoin="round"
        stroke-width="2"
        d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
      />
    </svg>
    <input
      :value="localValue"
      @input="handleInput"
      type="text"
      :placeholder="placeholder || 'Search...'"
      class="w-full pl-10 pr-10 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500 bg-white dark:bg-gray-700 dark:text-gray-100 dark:placeholder-gray-400"
    />
    <button
      v-if="localValue"
      @click="clear"
      class="absolute right-3 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
    >
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
      </svg>
    </button>
  </div>
</template>
