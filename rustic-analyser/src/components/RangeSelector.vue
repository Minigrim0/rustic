<template>
    <div class="range-selector px-2 py-3">
        <Slider
            v-model="range"
            :min="min"
            :max="max"
            :step="step"
            :tooltips="true"
            :lazy="lazy"
            :format="formatValue"
        />
    </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import Slider from "@vueform/slider";

const props = withDefaults(defineProps<{
    modelValue: [number, number];
    min?: number;
    max?: number;
    step?: number;
    /** Only emit on handle release instead of during drag. */
    lazy?: boolean;
    /** Tooltip label formatter. */
    formatValue?: (v: number) => string;
}>(), {
    min: 0,
    max: 1,
    step: -1,
    lazy: true,
    formatValue: (v: number) => v.toFixed(4),
});

const emit = defineEmits<{
    "update:modelValue": [value: [number, number]];
}>();

const range = computed({
    get: () => props.modelValue,
    set: (val: [number, number]) => emit("update:modelValue", val),
});
</script>

<style>
@import "@vueform/slider/themes/default.css";

.range-selector {
    --slider-bg: #d1d5db;
    --slider-connect-bg: #6366f1;
    --slider-handle-bg: #fff;
    --slider-handle-border: 2px solid #6366f1;
    --slider-handle-width: 16px;
    --slider-handle-height: 16px;
    --slider-handle-shadow: 0 1px 3px rgba(0, 0, 0, 0.15);
    --slider-height: 4px;
    --slider-tooltip-bg: #1f2937;
    --slider-tooltip-color: #f3f4f6;
    --slider-tooltip-font-size: 0.625rem;
    --slider-tooltip-line-height: 1.25;
    --slider-tooltip-py: 2px;
    --slider-tooltip-px: 6px;
}

:where(.dark, .dark *) .range-selector {
    --slider-bg: #374151;
    --slider-connect-bg: #818cf8;
    --slider-handle-bg: #1f2937;
    --slider-handle-border: 2px solid #818cf8;
    --slider-tooltip-bg: #111827;
    --slider-tooltip-color: #e5e7eb;
}
</style>