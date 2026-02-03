<script setup lang="ts">
import { onMounted } from "vue";
import { RouterView } from "vue-router";
import AppLayout from "./components/AppLayout.vue";
import { useSettingsStore } from "./stores/settings";

const settingsStore = useSettingsStore();

onMounted(async () => {
  await settingsStore.fetchSettings();
  settingsStore.initTheme();
});
</script>

<template>
  <AppLayout>
    <RouterView />
  </AppLayout>
</template>
