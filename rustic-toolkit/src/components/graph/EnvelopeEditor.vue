<template>
    <div class="envelope-editor">
        <EnvelopeSvg
            :attack="attack"
            :decay="decay"
            :sustain="sustain"
            :release="release"
            :attack-curve="attackCurve"
            :decay-curve="decayCurve"
            :release-curve="releaseCurve"
            :attack-cp-t="attackCpT"
            :decay-cp-t="decayCpT"
            :release-cp-t="releaseCpT"
            @update:sustain="$emit('update:sustain', $event)"
            @update:attackCurve="$emit('update:attackCurve', $event)"
            @update:decayCurve="$emit('update:decayCurve', $event)"
            @update:releaseCurve="$emit('update:releaseCurve', $event)"
            @update:attackCpT="$emit('update:attackCpT', $event)"
            @update:decayCpT="$emit('update:decayCpT', $event)"
            @update:releaseCpT="$emit('update:releaseCpT', $event)"
        />
        <EnvelopeDurationControls
            :attack="attack"
            :decay="decay"
            :release="release"
            @update:attack="$emit('update:attack', $event)"
            @update:decay="$emit('update:decay', $event)"
            @update:release="$emit('update:release', $event)"
        />
    </div>
</template>

<script setup lang="ts">
import EnvelopeSvg from "./EnvelopeSvg.vue";
import EnvelopeDurationControls from "./EnvelopeDurationControls.vue";

defineProps<{
    attack: number;
    decay: number;
    sustain: number;
    release: number;
    attackCurve: number;
    decayCurve: number;
    releaseCurve: number;
    attackCpT: number;
    decayCpT: number;
    releaseCpT: number;
}>();

defineEmits<{
    "update:attack":        [v: number];
    "update:decay":         [v: number];
    "update:sustain":       [v: number];
    "update:release":       [v: number];
    "update:attackCurve":   [v: number];
    "update:decayCurve":    [v: number];
    "update:releaseCurve":  [v: number];
    "update:attackCpT":     [v: number];
    "update:decayCpT":      [v: number];
    "update:releaseCpT":    [v: number];
}>();
</script>

<style scoped>
.envelope-editor {
    display: flex;
    flex-direction: column;
    gap: 2px;
}
</style>
