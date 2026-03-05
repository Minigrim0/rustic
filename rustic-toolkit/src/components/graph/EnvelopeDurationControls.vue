<template>
    <div class="duration-controls">
        <div v-for="ctrl in controls" :key="ctrl.key" class="duration-field">
            <label class="duration-label">{{ ctrl.label }}</label>
            <input
                type="range"
                :min="ctrl.min"
                :max="ctrl.max"
                :step="ctrl.step"
                :value="ctrl.value"
                class="duration-slider"
                @input="onInput(ctrl.key, $event)"
            />
            <span class="duration-value">{{ ctrl.value.toFixed(3) }}s</span>
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
    attack: number;
    decay: number;
    release: number;
}>();

const emit = defineEmits<{
    "update:attack":  [v: number];
    "update:decay":   [v: number];
    "update:release": [v: number];
}>();

const controls = computed(() => [
    { key: "attack",  label: "A", min: 0.001, max: 5, step: 0.001, value: props.attack  },
    { key: "decay",   label: "D", min: 0.001, max: 5, step: 0.001, value: props.decay   },
    { key: "release", label: "R", min: 0.001, max: 5, step: 0.001, value: props.release },
]);

function onInput(key: string, e: Event) {
    const val = Number((e.target as HTMLInputElement).value);
    if      (key === "attack")  emit("update:attack",  val);
    else if (key === "decay")   emit("update:decay",   val);
    else if (key === "release") emit("update:release", val);
}
</script>

<style scoped>
.duration-controls {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 6px 4px 2px;
}

.duration-field {
    display: flex;
    align-items: center;
    gap: 6px;
}

.duration-label {
    width: 12px;
    font-size: 10px;
    color: #6b7280;
    text-align: right;
    flex-shrink: 0;
}

.duration-slider {
    flex: 1;
    accent-color: #6366f1;
    height: 4px;
    cursor: pointer;
}

.duration-value {
    width: 44px;
    font-size: 10px;
    color: #9ca3af;
    text-align: right;
    font-variant-numeric: tabular-nums;
}
</style>
