<script setup lang="ts">
import { onMounted, ref, computed } from "vue";
import { useRoute, useRouter } from "vue-router";
import { invoke } from "@tauri-apps/api/core";
import { useFamiliesStore } from "../stores/families";
import MemberCard from "../components/members/MemberCard.vue";
import ConfirmDialog from "../components/common/ConfirmDialog.vue";
import PhotoUpload from "../components/common/PhotoUpload.vue";
import PhotoCropper from "../components/common/PhotoCropper.vue";

const route = useRoute();
const router = useRouter();
const familiesStore = useFamiliesStore();

const showDeleteMemberDialog = ref(false);
const memberToDelete = ref<number | null>(null);

const showCropper = ref(false);
const cropTargetMemberId = ref<number | null>(null);
const familyPhotoBase64 = ref<string | null>(null);

const familyId = computed(() => Number(route.params.id));
const hasFamilyPhoto = computed(() => !!familiesStore.currentFamily?.photo_path);

onMounted(async () => {
  await familiesStore.fetchFamily(familyId.value);
});

async function handlePhotoSelect(path: string) {
  await familiesStore.saveFamilyPhoto(familyId.value, path);
  await loadFamilyPhotoForCropping();
}

async function handlePhotoRemove() {
  await familiesStore.removeFamilyPhoto(familyId.value);
  familyPhotoBase64.value = null;
}

async function loadFamilyPhotoForCropping() {
  if (!familiesStore.currentFamily?.photo_path) {
    familyPhotoBase64.value = null;
    return;
  }
  try {
    familyPhotoBase64.value = await invoke<string>("get_photo_base64", {
      photoType: "families",
      filename: familiesStore.currentFamily.photo_path,
    });
  } catch (e) {
    console.error("Failed to load family photo for cropping:", e);
    familyPhotoBase64.value = null;
  }
}

async function openCropperForMember(memberId: number) {
  cropTargetMemberId.value = memberId;
  if (!familyPhotoBase64.value) {
    await loadFamilyPhotoForCropping();
  }
  if (familyPhotoBase64.value) {
    showCropper.value = true;
  }
}

const cropLoading = ref(false);
const cropError = ref<string | null>(null);

async function handleCrop(cropData: { x: number; y: number; width: number; height: number }) {
  if (!cropTargetMemberId.value) {
    console.error("No target member ID");
    return;
  }

  console.log("handleCrop called with:", cropData);
  console.log("Target member ID:", cropTargetMemberId.value);
  console.log("Family ID:", familyId.value);

  cropLoading.value = true;
  cropError.value = null;

  try {
    await familiesStore.cropFamilyPhotoToMember(
      familyId.value,
      cropTargetMemberId.value,
      cropData
    );
    console.log("Crop successful");
    showCropper.value = false;
    cropTargetMemberId.value = null;
  } catch (e) {
    console.error("Failed to crop photo:", e);
    cropError.value = String(e);
    alert("Failed to crop photo: " + String(e));
  } finally {
    cropLoading.value = false;
  }
}

function closeCropper() {
  showCropper.value = false;
  cropTargetMemberId.value = null;
}

function goBack() {
  router.push("/");
}

function editFamily() {
  router.push(`/families/${familyId.value}/edit`);
}

function addMember() {
  router.push(`/families/${familyId.value}/members/new`);
}

function editMember(memberId: number) {
  router.push(`/members/${memberId}/edit`);
}

function confirmDeleteMember(memberId: number) {
  memberToDelete.value = memberId;
  showDeleteMemberDialog.value = true;
}

async function deleteMember() {
  if (memberToDelete.value) {
    await familiesStore.deleteMember(memberToDelete.value);
    showDeleteMemberDialog.value = false;
    memberToDelete.value = null;
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
      Back to Directory
    </button>

    <div v-if="familiesStore.loading" class="flex items-center justify-center py-12">
      <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600"></div>
    </div>

    <div v-else-if="familiesStore.error" class="bg-red-50 dark:bg-red-900/30 text-red-700 dark:text-red-400 p-4 rounded-lg">
      {{ familiesStore.error }}
    </div>

    <div v-else-if="familiesStore.currentFamily" class="space-y-6">
      <!-- Family Header -->
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow-sm p-6">
        <div class="flex items-start justify-between">
          <div class="flex items-start gap-6">
            <!-- Family Photo -->
            <PhotoUpload
              :current-photo="familiesStore.currentFamily.photo_path"
              photo-type="families"
              @select="handlePhotoSelect"
              @remove="handlePhotoRemove"
            />
            <div>
              <h1 class="text-2xl font-bold text-gray-800 dark:text-gray-100">
                {{ familiesStore.currentFamily.name }}
              </h1>
              <div class="mt-2 text-gray-600 dark:text-gray-400">
                <p v-if="familiesStore.currentFamily.mailing_name" class="font-medium">{{ familiesStore.currentFamily.mailing_name }}</p>
                <p v-if="familiesStore.currentFamily.address">{{ familiesStore.currentFamily.address }}</p>
                <p v-if="familiesStore.currentFamily.city || familiesStore.currentFamily.state || familiesStore.currentFamily.zip">
                  {{ [familiesStore.currentFamily.city, familiesStore.currentFamily.state].filter(Boolean).join(", ") }} {{ familiesStore.currentFamily.zip }}
                </p>
              </div>
              <div class="mt-3 space-y-1 text-gray-600 dark:text-gray-400">
                <p v-if="familiesStore.currentFamily.phone">
                  {{ familiesStore.currentFamily.phone }}
                </p>
                <p v-if="familiesStore.currentFamily.email" class="text-primary-600 dark:text-primary-400">
                  {{ familiesStore.currentFamily.email }}
                </p>
              </div>
            </div>
          </div>
          <button
            @click="editFamily"
            class="px-4 py-2 text-primary-600 dark:text-primary-400 hover:bg-primary-50 dark:hover:bg-primary-900/30 rounded-lg transition-colors"
          >
            Edit Family
          </button>
        </div>
        <p v-if="familiesStore.currentFamily.notes" class="mt-4 text-gray-600 dark:text-gray-400 border-t dark:border-gray-700 pt-4">
          {{ familiesStore.currentFamily.notes }}
        </p>
      </div>

      <!-- Members Section -->
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow-sm p-6">
        <div class="flex items-center justify-between mb-4">
          <h2 class="text-lg font-semibold text-gray-800 dark:text-gray-100">Family Members</h2>
          <button
            @click="addMember"
            class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors flex items-center gap-2"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
            Add Member
          </button>
        </div>

        <div v-if="familiesStore.currentFamily.members.length === 0" class="text-center py-8">
          <p class="text-gray-500 dark:text-gray-400">No members added yet</p>
        </div>

        <div v-else class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          <MemberCard
            v-for="member in familiesStore.currentFamily.members"
            :key="member.id"
            :member="member"
            :has-family-photo="hasFamilyPhoto"
            @edit="editMember"
            @delete="confirmDeleteMember"
            @crop-from-family="openCropperForMember"
          />
        </div>
      </div>
    </div>

    <ConfirmDialog
      :show="showDeleteMemberDialog"
      title="Delete Member"
      message="Are you sure you want to delete this member? This action cannot be undone."
      confirm-text="Delete"
      @confirm="deleteMember"
      @cancel="showDeleteMemberDialog = false"
    />

    <PhotoCropper
      v-if="showCropper && familyPhotoBase64"
      :image-src="familyPhotoBase64"
      :loading="cropLoading"
      @crop="handleCrop"
      @cancel="closeCropper"
    />
  </div>
</template>
