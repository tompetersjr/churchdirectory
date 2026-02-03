import { createRouter, createWebHistory } from "vue-router";
import HomeView from "./views/HomeView.vue";

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: "/",
      name: "home",
      component: HomeView,
    },
    {
      path: "/families/new",
      name: "family-new",
      component: () => import("./views/FamilyEditView.vue"),
    },
    {
      path: "/families/:id",
      name: "family-detail",
      component: () => import("./views/FamilyDetailView.vue"),
    },
    {
      path: "/families/:id/edit",
      name: "family-edit",
      component: () => import("./views/FamilyEditView.vue"),
    },
    {
      path: "/families/:familyId/members/new",
      name: "member-new",
      component: () => import("./views/MemberEditView.vue"),
    },
    {
      path: "/members/:id/edit",
      name: "member-edit",
      component: () => import("./views/MemberEditView.vue"),
    },
    {
      path: "/import",
      name: "import",
      component: () => import("./views/ImportView.vue"),
    },
    {
      path: "/generate",
      name: "generate",
      component: () => import("./views/GenerateView.vue"),
    },
    {
      path: "/backup",
      name: "backup",
      component: () => import("./views/BackupView.vue"),
    },
    {
      path: "/settings",
      name: "settings",
      component: () => import("./views/SettingsView.vue"),
    },
    {
      path: "/about",
      name: "about",
      component: () => import("./views/AboutView.vue"),
    },
  ],
});

export default router;
