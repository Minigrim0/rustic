<template>
    <button
        class="play-button"
        :class="{ playing: modelValue }"
        @click="$emit('update:modelValue', !modelValue)"
    >
        {{ modelValue ? "Pause" : "Play" }}
    </button>
</template>

<script setup lang="ts">
defineProps<{
    modelValue: boolean;
}>();

defineEmits<{
    "update:modelValue": [value: boolean];
}>();
</script>

<style scoped>
.play-button {
    width: 100%;
    padding: 4px 12px;
    border: 1px solid #6366f1;
    border-radius: 4px;
    background: transparent;
    color: #6366f1;
    cursor: pointer;
    font-size: 0.85rem;
    transition: background 0.15s, color 0.15s;
}

.play-button:hover {
    background: #6366f1;
    color: #fff;
}

.play-button.playing {
    background: #6366f1;
    color: #fff;
}

.play-button.playing:hover {
    background: #4f46e5;
    border-color: #4f46e5;
}
</style>