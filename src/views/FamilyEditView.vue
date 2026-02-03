<script setup lang="ts">
import { onMounted, ref, computed } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useFamiliesStore } from "../stores/families";
import FamilyForm from "../components/families/FamilyForm.vue";
import type { Family } from "../types";

const route = useRoute();
const router = useRouter();
const familiesStore = useFamiliesStore();

const isNew = computed(() => route.name === "family-new");
const familyId = computed(() => (isNew.value ? null : Number(route.params.id)));
const loading = ref(false);

const initialData = ref<Partial<Family>>({
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

onMounted(async () => {
  if (!isNew.value && familyId.value) {
    await familiesStore.fetchFamily(familyId.value);
    if (familiesStore.currentFamily) {
      initialData.value = { ...familiesStore.currentFamily };
    }
  }
});

function goBack() {
  if (isNew.value) {
    router.push("/");
  } else {
    router.push(`/families/${familyId.value}`);
  }
}

async function handleSubmit(data: Partial<Family>) {
  loading.value = true;
  try {
    if (isNew.value) {
      const id = await familiesStore.createFamily(data as any);
      router.push(`/families/${id}`);
    } else if (familyId.value) {
      await familiesStore.updateFamily(familyId.value, data);
      router.push(`/families/${familyId.value}`);
    }
  } finally {
    loading.value = false;
  }
}
</script>

<template>
  <div class="p-6 h-full overflow-y-auto">
    <button
      @click="goBack"
      class="flex items-center gap-2 text-gray-600 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-200 mb-6"
    >
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
      </svg>
      {{ isNew ? "Back to Directory" : "Back to Family" }}
    </button>

    <div class="bg-white dark:bg-gray-800 rounded-lg shadow-sm p-6">
      <h1 class="text-2xl font-bold text-gray-800 dark:text-gray-100 mb-6">
        {{ isNew ? "Add New Family" : "Edit Family" }}
      </h1>

      <div v-if="familiesStore.loading && !isNew" class="flex items-center justify-center py-12">
        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600"></div>
      </div>

      <FamilyForm
        v-else
        :initial-data="initialData"
        :loading="loading"
        @submit="handleSubmit"
        @cancel="goBack"
      />
    </div>
  </div>
</template>
