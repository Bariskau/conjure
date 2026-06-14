import { createRouter, createWebHistory } from "vue-router";

import EditorPage from "@/pages/EditorPage.vue";
import LogsPage from "@/pages/LogsPage.vue";
import SettingsPage from "@/pages/SettingsPage.vue";
import TestPage from "@/pages/TestPage.vue";
import ToolsPage from "@/pages/ToolsPage.vue";

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: "/", redirect: "/test" },
    { path: "/tools", name: "tools", component: ToolsPage },
    { path: "/tools/new", name: "tool-create", component: EditorPage },
    { path: "/tools/:toolId/edit", name: "tool-edit", component: EditorPage, props: true },
    { path: "/test/:toolId?", name: "test", component: TestPage, props: true },
    { path: "/logs", name: "logs", component: LogsPage },
    { path: "/settings", name: "settings", component: SettingsPage },
  ],
});

export default router;
