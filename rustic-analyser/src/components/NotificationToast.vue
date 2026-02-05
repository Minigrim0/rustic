<template>
    <div class="pointer-events-none fixed inset-0 z-50 flex flex-col items-end gap-2 p-4">
        <TransitionGroup name="toast">
            <div v-for="item in items" :key="item.id"
                class="pointer-events-auto flex w-80 items-start gap-2 rounded-lg border px-3 py-2.5 text-sm shadow-lg"
                :class="typeClasses[item.type]">
                <span class="flex-1">{{ item.message }}</span>
                <button @click="dismiss(item.id)"
                    class="shrink-0 rounded p-0.5 opacity-60 transition-opacity hover:opacity-100">
                    <svg class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none"
                        stroke="currentColor" stroke-width="2">
                        <path stroke-linecap="round" stroke-linejoin="round"
                            d="M6 18 18 6M6 6l12 12" />
                    </svg>
                </button>
            </div>
        </TransitionGroup>
    </div>
</template>

<script lang="ts">
import { notifications } from "../stores/notifications";

const typeClasses: Record<string, string> = {
    error:
        "border-red-300/60 bg-red-50 text-red-800 dark:border-red-500/30 dark:bg-red-950 dark:text-red-200",
    warning:
        "border-yellow-300/60 bg-yellow-50 text-yellow-800 dark:border-yellow-500/30 dark:bg-yellow-950 dark:text-yellow-200",
    info:
        "border-blue-300/60 bg-blue-50 text-blue-800 dark:border-blue-500/30 dark:bg-blue-950 dark:text-blue-200",
};

export default {
    name: "NotificationToast",
    computed: {
        items() {
            return notifications.state.items;
        },
        typeClasses() {
            return typeClasses;
        },
    },
    methods: {
        dismiss(id: number) {
            notifications.dismiss(id);
        },
    },
};
</script>

<style scoped>
.toast-enter-active,
.toast-leave-active {
    transition: all 0.25s ease;
}
.toast-enter-from,
.toast-leave-to {
    opacity: 0;
    transform: translateX(1rem);
}
</style>