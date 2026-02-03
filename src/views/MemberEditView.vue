<script setup lang="ts">
import { onMounted, ref, computed } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useFamiliesStore } from "../stores/families";
import MemberForm from "../components/members/MemberForm.vue";
import type { Member } from "../types";

const route = useRoute();
const router = useRouter();
const familiesStore = useFamiliesStore();

const isNew = computed(() => route.name === "member-new");
const memberId = computed(() => (isNew.value ? null : Number(route.params.id)));
const familyId = computed(() =>
  isNew.value ? Number(route.params.familyId) : null
);
const loading = ref(false);
const memberLoading = ref(false);

const initialData = ref<Partial<Member>>({
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

const currentFamilyId = ref<number | null>(null);

onMounted(async () => {
  if (!isNew.value && memberId.value) {
    memberLoading.value = true;
    try {
      const member = await familiesStore.getMember(memberId.value);
      initialData.value = { ...member };
      currentFamilyId.value = member.family_id;
    } finally {
      memberLoading.value = false;
    }
  } else if (isNew.value && familyId.value) {
    currentFamilyId.value = familyId.value;
    initialData.value.family_id = familyId.value;
  }
});

function goBack() {
  const backFamilyId = currentFamilyId.value || familyId.value;
  if (backFamilyId) {
    router.push(`/families/${backFamilyId}`);
  } else {
    router.push("/");
  }
}

async function handleSubmit(data: Partial<Member>) {
  loading.value = true;
  try {
    if (isNew.value && familyId.value) {
      await familiesStore.createMember({
        ...data,
        family_id: familyId.value,
        sort_order: data.sort_order || 0,
      } as any);
      router.push(`/families/${familyId.value}`);
    } else if (memberId.value) {
      await familiesStore.updateMember(memberId.value, data);
      goBack();
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
      Back to Family
    </button>

    <div class="bg-white dark:bg-gray-800 rounded-lg shadow-sm p-6">
      <h1 class="text-2xl font-bold text-gray-800 dark:text-gray-100 mb-6">
        {{ isNew ? "Add New Member" : "Edit Member" }}
      </h1>

      <div v-if="memberLoading" class="flex items-center justify-center py-12">
        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600"></div>
      </div>

      <MemberForm
        v-else
        :initial-data="initialData"
        :loading="loading"
        @submit="handleSubmit"
        @cancel="goBack"
      />
    </div>
  </div>
</template>
