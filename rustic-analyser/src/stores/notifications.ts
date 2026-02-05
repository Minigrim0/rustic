import { reactive } from "vue";

export interface Notification {
    id: number;
    message: string;
    type: "error" | "warning" | "info";
}

let nextId = 0;

const state = reactive<{ items: Notification[] }>({
    items: [],
});

function push(message: string, type: Notification["type"] = "error", durationMs = 6000) {
    const id = nextId++;
    state.items.push({ id, message, type });
    if (durationMs > 0) {
        setTimeout(() => dismiss(id), durationMs);
    }
}

function dismiss(id: number) {
    const idx = state.items.findIndex((n) => n.id === id);
    if (idx !== -1) state.items.splice(idx, 1);
}

export const notifications = {
    state,
    dismiss,
    error: (msg: string, duration?: number) => push(msg, "error", duration),
    warn: (msg: string, duration?: number) => push(msg, "warning", duration),
    info: (msg: string, duration?: number) => push(msg, "info", duration),
};