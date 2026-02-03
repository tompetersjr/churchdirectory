<script setup lang="ts">
import { ref } from "vue";
import { RouterLink, useRoute } from "vue-router";
import { exit } from "@tauri-apps/plugin-process";

const route = useRoute();
const collapsed = ref(false);

async function exitApp() {
  await exit(0);
}

function toggleSidebar() {
  collapsed.value = !collapsed.value;
}

const navItems = [
  { path: "/", label: "Directory", icon: "home" },
  { path: "/import", label: "Import", icon: "upload" },
  { path: "/generate", label: "Generate PDF", icon: "document" },
  { path: "/backup", label: "Backup", icon: "archive" },
  { path: "/settings", label: "Settings", icon: "cog" },
  { path: "/about", label: "About", icon: "info" },
];

const isActive = (path: string) => {
  if (path === "/") return route.path === "/";
  return route.path.startsWith(path);
};
</script>

<template>
  <div class="flex h-screen bg-gray-100 dark:bg-gray-900">
    <!-- Sidebar -->
    <aside
      class="bg-white dark:bg-gray-800 shadow-md flex flex-col transition-all duration-300"
      :class="collapsed ? 'w-16' : 'w-64'"
    >
      <div class="p-4 border-b dark:border-gray-700 flex items-center justify-between min-h-[60px]">
        <h1
          v-if="!collapsed"
          class="text-xl font-bold text-primary-700 dark:text-primary-400 truncate"
        >
          Church Directory
        </h1>
        <button
          @click="toggleSidebar"
          class="p-1 rounded hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-600 dark:text-gray-400 transition-colors"
          :class="collapsed ? 'mx-auto' : ''"
          :title="collapsed ? 'Expand sidebar' : 'Collapse sidebar'"
        >
          <svg
            class="w-5 h-5 transition-transform duration-300"
            :class="collapsed ? 'rotate-180' : ''"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M11 19l-7-7 7-7m8 14l-7-7 7-7"
            />
          </svg>
        </button>
      </div>
      <nav class="flex-1 p-4" :class="collapsed ? 'px-2' : ''">
        <ul class="space-y-2">
          <li v-for="item in navItems" :key="item.path">
            <RouterLink
              :to="item.path"
              class="flex items-center rounded-lg transition-colors"
              :class="[
                isActive(item.path)
                  ? 'bg-primary-100 dark:bg-primary-900/50 text-primary-700 dark:text-primary-400 font-medium'
                  : 'text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700',
                collapsed ? 'justify-center p-2' : 'gap-3 px-4 py-2',
              ]"
              :title="collapsed ? item.label : ''"
            >
              <!-- Icons -->
              <svg
                v-if="item.icon === 'home'"
                class="w-5 h-5"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6"
                />
              </svg>
              <svg
                v-if="item.icon === 'upload'"
                class="w-5 h-5"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12"
                />
              </svg>
              <svg
                v-if="item.icon === 'document'"
                class="w-5 h-5"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
                />
              </svg>
              <svg
                v-if="item.icon === 'archive'"
                class="w-5 h-5"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M5 8h14M5 8a2 2 0 110-4h14a2 2 0 110 4M5 8v10a2 2 0 002 2h10a2 2 0 002-2V8m-9 4h4"
                />
              </svg>
              <svg
                v-if="item.icon === 'cog'"
                class="w-5 h-5"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
                />
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                />
              </svg>
              <svg
                v-if="item.icon === 'info'"
                class="w-5 h-5"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                />
              </svg>
              <span v-if="!collapsed">{{ item.label }}</span>
            </RouterLink>
          </li>
        </ul>
      </nav>
      <div class="p-4 border-t dark:border-gray-700" :class="collapsed ? 'px-2' : ''">
        <button
          @click="exitApp"
          class="flex items-center w-full rounded-lg text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
          :class="collapsed ? 'justify-center p-2' : 'gap-3 px-4 py-2'"
          :title="collapsed ? 'Exit' : ''"
        >
          <svg
            class="w-5 h-5"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"
            />
          </svg>
          <span v-if="!collapsed">Exit</span>
        </button>
      </div>
    </aside>

    <!-- Main content -->
    <main class="flex-1 overflow-hidden">
      <slot />
    </main>
  </div>
</template>
