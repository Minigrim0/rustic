<template>
    <div>
        <div v-if="loaded" class="flex h-64 flex-col">
            <!-- Chart area -->
            <div class="relative min-h-0 flex-1">
                <v-chart :option="chartOption" autoresize class="h-full w-full" />
                <div v-if="chartLoading"
                    class="absolute inset-0 flex items-center justify-center bg-white/60 dark:bg-gray-900/60">
                    <svg class="h-5 w-5 animate-spin text-indigo-400" fill="none" viewBox="0 0 24 24">
                        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                        <path class="opacity-75" fill="currentColor"
                            d="M4 12a8 8 0 0 1 8-8V0C5.373 0 0 5.373 0 12h4z" />
                    </svg>
                </div>
            </div>
            <!-- Bottom bar: info + settings cog -->
            <div
                class="flex shrink-0 items-center gap-4 border-t border-gray-100 px-3 py-1.5 text-[10px] text-gray-500 dark:border-white/5 dark:text-gray-400">
                <span v-if="data"><strong class="font-medium text-gray-700 dark:text-gray-300">{{
                    data.time_bins }}</strong>&times;<strong
                        class="font-medium text-gray-700 dark:text-gray-300">{{ data.freq_bins }}</strong>
                    bins</span>
                <span v-if="peakInfo">Peak: <strong class="font-medium text-gray-700 dark:text-gray-300">{{
                    formatFreq(peakInfo.freq) }}</strong> @ {{ peakInfo.time.toFixed(3) }}s</span>
                <div class="relative ml-auto shrink-0">
                    <button @click="settingsOpen = !settingsOpen"
                        class="rounded p-1 text-gray-300 transition-colors hover:text-gray-500 dark:text-gray-600 dark:hover:text-gray-400"
                        :class="{ 'text-gray-500 dark:text-gray-400': settingsOpen }" title="Spectrogram settings">
                        <svg class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor"
                            stroke-width="2">
                            <path stroke-linecap="round" stroke-linejoin="round"
                                d="M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.325.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 0 1 1.37.49l1.296 2.247a1.125 1.125 0 0 1-.26 1.431l-1.003.827c-.293.241-.438.613-.43.992a7 7 0 0 1 0 .255c-.007.378.138.75.43.991l1.004.827c.424.35.534.955.26 1.43l-1.298 2.247a1.125 1.125 0 0 1-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a7 7 0 0 1-.22.128c-.331.183-.581.495-.644.869l-.213 1.281c-.09.543-.56.94-1.11.94h-2.594c-.55 0-1.019-.398-1.11-.94l-.213-1.281c-.062-.374-.312-.686-.644-.87a7 7 0 0 1-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 0 1-1.369-.49l-1.297-2.247a1.125 1.125 0 0 1 .26-1.431l1.004-.827c.292-.24.437-.613.43-.991a7 7 0 0 1 0-.255c.007-.38-.138-.751-.43-.992l-1.004-.827a1.125 1.125 0 0 1-.26-1.43l1.297-2.247a1.125 1.125 0 0 1 1.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.086.22-.128.332-.183.582-.495.644-.869l.214-1.28Z" />
                            <path stroke-linecap="round" stroke-linejoin="round"
                                d="M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z" />
                        </svg>
                    </button>
                    <div v-if="settingsOpen"
                        class="absolute bottom-full right-0 z-50 mb-1 w-48 rounded-md border border-gray-200 bg-white p-2 shadow-lg dark:border-white/10 dark:bg-gray-800">
                        <div class="space-y-2">
                            <label class="block">
                                <span class="text-[10px] text-gray-500 dark:text-gray-400">Color scheme</span>
                                <select v-model="settings.colorScheme"
                                    class="mt-0.5 block w-full rounded border border-gray-200 bg-gray-50 px-1.5 py-0.5 text-[11px] text-gray-700 focus:border-indigo-400 focus:outline-none dark:border-white/10 dark:bg-gray-900 dark:text-gray-300">
                                    <option value="heat">Heat</option>
                                    <option value="plasma">Plasma</option>
                                    <option value="viridis">Viridis</option>
                                    <option value="grayscale">Grayscale</option>
                                </select>
                            </label>
                            <label class="flex items-center gap-1.5">
                                <input v-model="settings.enhancedContrast" type="checkbox"
                                    class="h-3 w-3 rounded border-gray-300 text-indigo-500 focus:ring-indigo-400 dark:border-white/20 dark:bg-gray-900" />
                                <span class="text-[10px] text-gray-500 dark:text-gray-400">Enhanced contrast</span>
                            </label>
                            <div class="flex items-center gap-1.5">
                                <span class="text-[10px] text-gray-500 dark:text-gray-400">Freq scale</span>
                                <button @click="settings.freqScale = settings.freqScale === 'linear' ? 'log' : 'linear'"
                                    class="rounded border px-1.5 py-0.5 text-[10px] font-medium transition-colors"
                                    :class="settings.freqScale === 'log'
                                        ? 'border-indigo-400 bg-indigo-50 text-indigo-600 dark:border-indigo-500 dark:bg-indigo-500/10 dark:text-indigo-400'
                                        : 'border-gray-200 text-gray-500 hover:border-gray-300 dark:border-white/10 dark:text-gray-400'">
                                    {{ settings.freqScale === 'log' ? 'Log' : 'Linear' }}
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
        <div v-else class="flex flex-col items-center gap-3 py-16">
            <svg class="h-6 w-6 animate-spin text-indigo-400" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 0 1 8-8V0C5.373 0 0 5.373 0 12h4z" />
            </svg>
            <p class="text-xs text-gray-500 dark:text-gray-400">Loading spectrogram...</p>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from "vue";
import VChart from "vue-echarts";
import "echarts";
import type { EChartsOption } from "echarts";

import { notifications } from "@/stores/notifications.ts";
import { type SpectrogramData } from "@/types";
import { getSpectrogram } from "@/utils/tauri-api.ts";
import { spectrogramSettings, type ColorScheme } from "@/composables/useSpectrogramSettings.ts";

const props = defineProps<{
    start: number;
    end: number;
}>();

const settings = spectrogramSettings;
const settingsOpen = ref(false);

// Close settings dropdown when clicking outside
function onClickOutside(e: MouseEvent) {
    if (settingsOpen.value && !(e.target as HTMLElement).closest(".relative")) {
        settingsOpen.value = false;
    }
}
onMounted(() => document.addEventListener("click", onClickOutside, true));
onUnmounted(() => document.removeEventListener("click", onClickOutside, true));

// --- Color schemes ---

const COLOR_SCHEMES: Record<ColorScheme, string[]> = {
    heat: ["#000000", "#8b0000", "#ff0000", "#ff8c00", "#ffff00", "#ffffff"],
    plasma: ["#0d0887", "#6a00a8", "#b12a90", "#e16462", "#fca636", "#f0f921"],
    viridis: ["#440154", "#443983", "#31688e", "#21918c", "#35b779", "#90d743", "#fde725"],
    grayscale: ["#000000", "#ffffff"],
};

// --- Data fetching ---

const loaded = ref(false);
const data = ref<SpectrogramData | null>(null);

async function loadData() {
    loaded.value = false;
    try {
        data.value = await getSpectrogram(props.start, props.end);
    } catch (err: any) {
        notifications.error(`Unable to load spectrogram: ${err.message || err}`);
    } finally {
        loaded.value = true;
    }
}

onMounted(loadData);
watch([() => props.start, () => props.end], loadData);

// --- Peak info ---

const peakInfo = computed(() => {
    const d = data.value;
    if (!d || d.data.length === 0) return null;

    const maxFreq = d.sample_rate / 2;
    const freqRes = maxFreq / d.freq_bins;
    const timeStep = (d.end_time - d.start_time) / d.time_bins;

    let peakMag = 0, peakT = 0, peakF = 0;
    for (let t = 0; t < d.data.length; t++) {
        const frame = d.data[t]!;
        for (let f = 0; f < frame.length; f++) {
            if (frame[f]! > peakMag) {
                peakMag = frame[f]!;
                peakT = t;
                peakF = f;
            }
        }
    }

    return {
        freq: peakF * freqRes,
        time: d.start_time + peakT * timeStep,
        magnitude: peakMag,
    };
});

// --- Helpers ---

function formatFreq(hz: number): string {
    if (hz < 1000) return Math.round(hz) + " Hz";
    return (hz / 1000).toFixed(1) + " kHz";
}

function formatTime(s: number): string {
    return s.toFixed(3) + "s";
}

// --- Log-scale resampling ---

function resampleToLog(
    frames: number[][],
    freqBins: number,
    maxFreq: number,
): { data: number[][]; freqLabels: number[] } {
    const minFreq = 20;
    const numBins = freqBins;
    const logFreqs: number[] = [];
    const freqRes = maxFreq / freqBins;

    for (let i = 0; i < numBins; i++) {
        logFreqs.push(minFreq * Math.pow(maxFreq / minFreq, i / (numBins - 1)));
    }

    const resampled = frames.map((frame) => {
        return logFreqs.map((freq) => {
            const idx = freq / freqRes;
            const lo = Math.floor(idx);
            const hi = Math.ceil(idx);
            const frac = idx - lo;
            if (hi >= frame.length) return frame[frame.length - 1]!;
            return frame[lo]! * (1 - frac) + frame[hi]! * frac;
        });
    });

    return { data: resampled, freqLabels: logFreqs };
}

// --- Chart option ---

const chartOption = ref<EChartsOption>({});
const chartLoading = ref(false);

watch(
    [data, () => settings.colorScheme, () => settings.enhancedContrast, () => settings.freqScale],
    async () => {
        const d = data.value;
        if (!d || d.data.length === 0) return;

        chartLoading.value = true;
        await nextTick();

        const maxFreq = d.sample_rate / 2;
        const freqRes = maxFreq / d.freq_bins;
        const timeStep = (d.end_time - d.start_time) / d.time_bins;
        const isLog = settings.freqScale === "log";

        // Determine source data and frequency labels
        let sourceFrames: number[][];
        let freqLabels: number[];

        if (isLog) {
            const resampled = resampleToLog(d.data, d.freq_bins, maxFreq);
            sourceFrames = resampled.data;
            freqLabels = resampled.freqLabels;
        } else {
            sourceFrames = d.data;
            freqLabels = Array.from({ length: d.freq_bins }, (_, i) => i * freqRes);
        }

        // Find max magnitude for normalization
        let maxMag = 0;
        for (const frame of sourceFrames) {
            for (const v of frame) {
                if (v > maxMag) maxMag = v;
            }
        }
        if (maxMag === 0) maxMag = 1;

        // Build heatmap data: [timeIdx, freqIdx, normalizedMagnitude]
        const gamma = settings.enhancedContrast ? 0.3 : 1.0;
        const heatmapData: [number, number, number][] = [];

        for (let t = 0; t < sourceFrames.length; t++) {
            const frame = sourceFrames[t]!;
            for (let f = 0; f < frame.length; f++) {
                const normalized = Math.pow(frame[f]! / maxMag, gamma);
                heatmapData.push([t, f, normalized]);
            }
        }

        // Time labels (show a subset to avoid clutter)
        const timeLabels = Array.from({ length: sourceFrames.length }, (_, i) =>
            formatTime(d.start_time + i * timeStep),
        );

        // Frequency labels
        const freqLabelStrings = freqLabels.map((f) => formatFreq(f));

        chartOption.value = {
            animation: false,
            grid: { top: 8, right: 60, bottom: 24, left: 48 },
            xAxis: {
                type: "category",
                data: timeLabels,
                axisLabel: { fontSize: 10 },
                axisTick: { show: false },
                axisLine: { lineStyle: { color: "#6b7280" } },
            },
            yAxis: {
                type: "category",
                data: freqLabelStrings,
                axisLabel: { fontSize: 10 },
                axisTick: { show: false },
                axisLine: { lineStyle: { color: "#6b7280" } },
            },
            visualMap: {
                min: 0,
                max: 1,
                calculable: false,
                orient: "vertical",
                right: 0,
                top: "center",
                inRange: {
                    color: COLOR_SCHEMES[settings.colorScheme],
                },
                textStyle: { fontSize: 9, color: "#6b7280" },
                itemWidth: 8,
                itemHeight: 80,
            },
            tooltip: {
                formatter: (params: any) => {
                    const [tIdx, fIdx, mag] = params.data as [number, number, number];
                    const time = d.start_time + tIdx * timeStep;
                    const freq = freqLabels[fIdx] ?? 0;
                    return `Time: ${formatTime(time)}<br/>Freq: ${formatFreq(freq)}<br/>Intensity: ${mag.toFixed(3)}`;
                },
            },
            series: [
                {
                    type: "heatmap",
                    data: heatmapData,
                    progressive: 5000,
                    emphasis: { itemStyle: { borderColor: "#333", borderWidth: 1 } },
                } as any,
            ],
        };

        await nextTick();
        chartLoading.value = false;
    },
    { immediate: true },
);
</script>