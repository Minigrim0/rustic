<template>
    <div class="node-controls">
        <button
            class="ctrl-btn play-btn"
            :class="{ active: modelValue === 'idle' || modelValue === 'releasing' || modelValue === 'killed' }"
            :disabled="modelValue === 'playing'"
            title="Play"
            @click="$emit('update:modelValue', 'playing')"
        >▶</button>
        <button
            class="ctrl-btn stop-btn"
            :class="{ active: modelValue === 'playing' }"
            :disabled="modelValue !== 'playing'"
            title="Stop (release)"
            @click="$emit('update:modelValue', 'releasing')"
        >⏸</button>
        <button
            class="ctrl-btn kill-btn"
            :class="{ active: modelValue === 'playing' }"
            :disabled="modelValue !== 'playing'"
            title="Kill (immediate)"
            @click="$emit('update:modelValue', 'killed')"
        >✕</button>
    </div>
</template>

<script setup lang="ts">
defineProps<{
    modelValue: 'idle' | 'playing' | 'releasing' | 'killed';
}>();

defineEmits<{
    'update:modelValue': [value: 'idle' | 'playing' | 'releasing' | 'killed'];
}>();
</script>

<style scoped>
.node-controls {
    display: flex;
    gap: 4px;
    width: 100%;
}

.ctrl-btn {
    flex: 1;
    padding: 3px 6px;
    border-radius: 4px;
    border: 1px solid;
    background: transparent;
    cursor: pointer;
    font-size: 0.8rem;
    transition: background 0.12s, color 0.12s, opacity 0.12s;
}

.ctrl-btn:disabled {
    opacity: 0.35;
    cursor: default;
}

.play-btn {
    border-color: #22c55e;
    color: #22c55e;
}
.play-btn.active {
    background: #22c55e;
    color: #fff;
}
.play-btn:not(:disabled):hover {
    background: #22c55e;
    color: #fff;
}

.stop-btn {
    border-color: #f59e0b;
    color: #f59e0b;
}
.stop-btn.active {
    background: #f59e0b;
    color: #fff;
}
.stop-btn:not(:disabled):hover {
    background: #f59e0b;
    color: #fff;
}

.kill-btn {
    border-color: #ef4444;
    color: #ef4444;
}
.kill-btn.active {
    background: transparent;
    color: #ef4444;
}
.kill-btn:not(:disabled):hover {
    background: #ef4444;
    color: #fff;
}
</style>
