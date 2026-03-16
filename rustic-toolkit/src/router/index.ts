import { createRouter, createWebHistory } from "vue-router";
import SampleAnalysis from "../pages/SampleAnalysis.vue";
import Graph from "../pages/Graph.vue";
import Settings from "../pages/Settings.vue";

const router = createRouter({
    history: createWebHistory(),
    routes: [
        {
            path: "/",
            name: "analyzer",
            component: SampleAnalysis,
        },
        {
            path: "/graph",
            name: "graph",
            component: Graph,
        },
        {
            path: "/settings",
            name: "settings",
            component: Settings,
        },
    ],
});

export default router;