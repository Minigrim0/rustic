<template>
    <div>
        <div v-if="componentStatus.loaded" class="h-64">
            <v-chart :option="option" autoresize class="h-full w-full" />
        </div>
        <div v-if="componentStatus.loaded" class="flex items-center gap-4 border-t border-gray-100 px-3 py-1.5 text-[10px] text-gray-500 dark:border-white/5 dark:text-gray-400">
            <span><strong class="font-medium text-gray-700 dark:text-gray-300">{{ segmentSamples.toLocaleString() }}</strong> samples</span>
            <span>{{ segmentDuration.toFixed(3) }}s segment</span>
            <span v-if="componentStatus.data?.downsampled" class="text-amber-500 dark:text-amber-400">downsampled</span>
        </div>
        <div v-else class="flex flex-col items-center gap-3 py-16">
            <svg class="h-6 w-6 animate-spin text-indigo-400" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 0 1 8-8V0C5.373 0 0 5.373 0 12h4z" />
            </svg>
            <p class="text-xs text-gray-500 dark:text-gray-400">Loading waveform...</p>
        </div>
    </div>
</template>

<script setup lang="ts">
import { reactive, computed, onMounted, watch } from "vue";
import VChart from "vue-echarts";
import "echarts";
import type { EChartsOption } from "echarts";

import { notifications } from "../../stores/notifications";
import { type AudioSummary, type WaveformData } from "../../types";
import { getWaveform } from "../../utils/tauri-api";

const props = defineProps<{
    summary: AudioSummary;
    start: number;
    end: number;
}>();

type ComponentStatus = {
    loaded: boolean;
    error: string | null;
    data: WaveformData | null;
};

const componentStatus = reactive<ComponentStatus>({
    loaded: false,
    error: null,
    data: null,
});

const loadWaveformData = async () => {
    componentStatus.loaded = false;

    try {
        const data = await getWaveform(props.start, props.end, 1000);
        componentStatus.data = data;
    } catch (err: any) {
        notifications.error(`Unable to load waveform data: ${err.message || err}`);
        componentStatus.error = `${err.message || err}`;
    } finally {
        componentStatus.loaded = true;
    }
};

onMounted(async () => {
    await loadWaveformData();
});

watch([() => props.start, () => props.end], async () => {
    await loadWaveformData();
});

const segmentDuration = computed(() => props.end - props.start);
const segmentSamples = computed(() => Math.round(segmentDuration.value * props.summary.sample_rate));

/** Build time labels matching the sample count. */
const timeLabels = computed<number[]>(() => {
    const samples = componentStatus.data?.samples;
    if (!samples || samples.length === 0) return [];

    const count = samples.length;
    const labels: number[] = [];
    for (let i = 0; i < count; i++) {
        labels.push(props.start + (i / (count - 1)) * (props.end - props.start));
    }
    return labels;
});

const option = computed<EChartsOption>(() => ({
    grid: { top: 8, right: 8, bottom: 24, left: 48 },
    xAxis: {
        type: "category",
        data: timeLabels.value,
        axisLabel: {
            formatter: (v: string) => parseFloat(v).toFixed(3) + "s",
            fontSize: 10,
        },
        axisTick: { show: false },
        axisLine: { lineStyle: { color: "#6b7280" } },
    },
    yAxis: {
        type: "value",
        min: -1,
        max: 1,
        axisLabel: { fontSize: 10 },
        splitLine: { lineStyle: { color: "#e5e7eb", opacity: 0.3 } },
    },
    series: [{
        type: "line",
        data: componentStatus.data?.samples,
        showSymbol: false,
        lineStyle: { width: 1, color: "#6366f1" },
        large: true,
        largeThreshold: 2000,
    }],
}));
</script>