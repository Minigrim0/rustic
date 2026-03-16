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

<script setup lang="ts">
import { onMounted } from "vue";
import { listen } from "@tauri-apps/api/event";
import Navbar from "./components/Navbar.vue";
import NotificationToast from "./components/NotificationToast.vue";
import { notifications } from "./stores/notifications";

onMounted(async () => {
    await listen("rustic-event", (event) => {
        const payload = event.payload as any;

        if ("Error" in payload) {
            const err = payload.Error;
            if ("ThreadPanic" in err) {
                notifications.error(
                    `Audio thread crashed: ${err.ThreadPanic.message}`,
                    0, // persistent — user must dismiss
                );
            } else if ("GraphError" in err) {
                notifications.error(`Graph error: ${err.GraphError.description}`);
            } else if ("CommandFailed" in err) {
                notifications.error(`Command failed: ${err.CommandFailed.message}`);
            }
        } else if ("Status" in payload) {
            const status = payload.Status;
            if ("AudioStarted" in status) {
                notifications.info(`Audio engine started (${status.AudioStarted.sample_rate} Hz)`);
            } else if ("AudioStopped" in status) {
                notifications.info("Audio engine stopped.");
            }
        }
    });
});
</script>
