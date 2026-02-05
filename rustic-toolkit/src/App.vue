<template>
    <NotificationToast />
    <div class="flex h-full">
        <Navbar />
        <router-view v-slot="{ Component }">
            <keep-alive>
                <component :is="Component" class="flex-1 overflow-hidden" />
            </keep-alive>
        </router-view>
    </div>
</template>

<script lang="ts">
import Navbar from "./components/Navbar.vue";
import NotificationToast from "./components/NotificationToast.vue";
import SampleAnalysis from "./pages/SampleAnalysis.vue";

export default {
    name: "App",
    components: {
        Navbar,
        SampleAnalysis,
        NotificationToast,
    },
};
</script>
