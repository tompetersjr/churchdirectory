<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { useFamiliesStore } from "../stores/families";
import SearchBar from "../components/common/SearchBar.vue";
import FamilyList from "../components/families/FamilyList.vue";

const router = useRouter();
const familiesStore = useFamiliesStore();
const showDeleteConfirm = ref(false);
const familyToDelete = ref<number | null>(null);

onMounted(() => {
  familiesStore.fetchFamilies();
});

function handleSearch(query: string) {
  familiesStore.searchQuery = query;
}

function handleSort(field: "name" | "updated_at") {
  if (familiesStore.sortBy === field) {
    familiesStore.sortOrder = familiesStore.sortOrder === "asc" ? "desc" : "asc";
  } else {
    familiesStore.sortBy = field;
    familiesStore.sortOrder = "asc";
  }
}

function createFamily() {
  router.push("/families/new");
}

function viewFamily(id: number) {
  router.push(`/families/${id}`);
}

function editFamily(id: number) {
  router.push(`/families/${id}/edit`);
}

function confirmDelete(id: number) {
  familyToDelete.value = id;
  showDeleteConfirm.value = true;
}

async function deleteFamily() {
  if (familyToDelete.value) {
    await familiesStore.deleteFamily(familyToDelete.value);
    showDeleteConfirm.value = false;
    familyToDelete.value = null;
  }
}
</script>

<template>
  <div class="p-6 h-full flex flex-col">
    <div class="flex items-center gap-4 mb-6">
      <div class="flex-shrink-0">
        <h1 class="text-2xl font-bold text-gray-800 dark:text-gray-100">Family Directory</h1>
        <p class="text-sm text-gray-500 dark:text-gray-400">
          <span v-if="familiesStore.searchQuery && familiesStore.filteredFamilies.length !== familiesStore.families.length">
            Showing {{ familiesStore.filteredFamilies.length }} of {{ familiesStore.families.length }} families
          </span>
          <span v-else>
            {{ familiesStore.families.length }} {{ familiesStore.families.length === 1 ? 'family' : 'families' }}
          </span>
        </p>
      </div>
      <div class="flex-1">
        <SearchBar
          :value="familiesStore.searchQuery"
          placeholder="Search families..."
          @search="handleSearch"
        />
      </div>
      <button
        @click="createFamily"
        class="flex-shrink-0 ml-auto px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors flex items-center gap-2"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
        Add Family
      </button>
    </div>

    <div v-if="familiesStore.loading" class="flex items-center justify-center py-12">
      <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600"></div>
    </div>

    <div v-else-if="familiesStore.error" class="bg-red-50 dark:bg-red-900/30 text-red-700 dark:text-red-400 p-4 rounded-lg">
      {{ familiesStore.error }}
    </div>

    <div v-else-if="familiesStore.filteredFamilies.length === 0" class="text-center py-12">
      <svg class="w-16 h-16 mx-auto text-gray-300 dark:text-gray-600 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
      </svg>
      <p class="text-gray-500 dark:text-gray-400">No families found</p>
      <button
        @click="createFamily"
        class="mt-4 px-4 py-2 text-primary-600 hover:text-primary-700"
      >
        Add your first family
      </button>
    </div>

    <div v-else class="flex-1 min-h-0">
      <FamilyList
        :families="familiesStore.filteredFamilies"
        :sort-by="familiesStore.sortBy"
        :sort-order="familiesStore.sortOrder"
        @sort="handleSort"
        @view="viewFamily"
        @edit="editFamily"
        @delete="confirmDelete"
      />
    </div>

    <!-- Delete Confirmation Modal -->
    <div
      v-if="showDeleteConfirm"
      class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
      @click.self="showDeleteConfirm = false"
    >
      <div class="bg-white dark:bg-gray-800 rounded-lg p-6 max-w-md w-full mx-4">
        <h3 class="text-lg font-semibold dark:text-gray-100 mb-2">Delete Family</h3>
        <p class="text-gray-600 dark:text-gray-400 mb-4">
          Are you sure you want to delete this family? This will also delete all members. This action cannot be undone.
        </p>
        <div class="flex justify-end gap-3">
          <button
            @click="showDeleteConfirm = false"
            class="px-4 py-2 text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg"
          >
            Cancel
          </button>
          <button
            @click="deleteFamily"
            class="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700"
          >
            Delete
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
