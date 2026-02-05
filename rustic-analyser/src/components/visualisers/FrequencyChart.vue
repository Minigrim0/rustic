<template>
    <div>
        <div v-if="componentStatus.loaded" class="flex h-64">
            <!-- Top frequencies list -->
            <div class="w-36 shrink-0 overflow-y-auto border-r border-gray-100 px-2 py-1.5 dark:border-white/5">
                <div class="flex items-center justify-between">
                    <span class="text-[10px] uppercase tracking-wider text-gray-400 dark:text-gray-500">Top
                        Frequencies</span>
                    <div class="flex items-center gap-0.5">
                        <button @click="filterTopByRange = !filterTopByRange" class="rounded p-0.5 transition-colors"
                            :class="filterTopByRange ? 'text-indigo-500 dark:text-indigo-400' : 'text-gray-300 hover:text-gray-400 dark:text-gray-600 dark:hover:text-gray-500'"
                            :title="filterTopByRange ? 'Showing peaks from visible range' : 'Showing global peaks'">
                            <svg class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                <path stroke-linecap="round" stroke-linejoin="round"
                                    d="M3 4a1 1 0 0 1 1-1h16a1 1 0 0 1 1 1v2.586a1 1 0 0 1-.293.707l-6.414 6.414a1 1 0 0 0-.293.707V17l-4 4v-6.586a1 1 0 0 0-.293-.707L3.293 7.293A1 1 0 0 1 3 6.586V4z" />
                            </svg>
                        </button>
                        <!-- Settings cog -->

                    </div>
                </div>
                <div v-if="topFreqsLoading" class="mt-2 flex justify-center">
                    <svg class="h-4 w-4 animate-spin text-indigo-400" fill="none" viewBox="0 0 24 24">
                        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                        <path class="opacity-75" fill="currentColor"
                            d="M4 12a8 8 0 0 1 8-8V0C5.373 0 0 5.373 0 12h4z" />
                    </svg>
                </div>
                <ol v-else class="mt-1 space-y-0.5">
                    <li v-for="(f, i) in topFrequencies" :key="i"
                        class="flex cursor-pointer items-baseline justify-between gap-1 rounded px-1 text-[11px] tabular-nums transition-colors hover:bg-gray-100 dark:hover:bg-white/5"
                        :class="selectedFreq !== null && Math.abs(selectedFreq - f.frequency) < 1 ? 'bg-indigo-50 dark:bg-indigo-500/10' : ''"
                        @click="selectedFreq = selectedFreq !== null && Math.abs(selectedFreq - f.frequency) < 1 ? null : f.frequency">
                        <span class="font-medium text-gray-700 dark:text-gray-300">{{ formatFreq(f.frequency) }}</span>
                        <span class="text-gray-400 dark:text-gray-500">{{ f.magnitude.toFixed(3) }}</span>
                    </li>
                </ol>
            </div>
            <!-- Chart + range selector -->
            <div class="flex h-full min-w-0 flex-1 flex-col">
                <div class="relative min-h-0 flex-1">
                    <v-chart :option="chartOption" autoresize class="h-full w-full" @click="onChartClick" />
                    <div v-if="chartLoading"
                        class="absolute inset-0 flex items-center justify-center bg-white/60 dark:bg-gray-900/60">
                        <svg class="h-5 w-5 animate-spin text-indigo-400" fill="none" viewBox="0 0 24 24">
                            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                            <path class="opacity-75" fill="currentColor"
                                d="M4 12a8 8 0 0 1 8-8V0C5.373 0 0 5.373 0 12h4z" />
                        </svg>
                    </div>
                </div>
                <div class="flex shrink-0 items-center gap-1 border-t border-gray-100 px-1 dark:border-white/5">
                    <RangeSelector v-model="freqRange" :min="0" :max="maxFreq" :step="1" :lazy="true"
                        :format-value="formatFreq" class="min-w-0 flex-1" />
                    <!-- Settings cog -->
                    <div class="relative shrink-0">
                        <button @click="settingsOpen = !settingsOpen"
                            class="rounded p-1 text-gray-300 transition-colors hover:text-gray-500 dark:text-gray-600 dark:hover:text-gray-400"
                            :class="{ 'text-gray-500 dark:text-gray-400': settingsOpen }" title="Spectrum settings">
                            <svg class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                <path stroke-linecap="round" stroke-linejoin="round"
                                    d="M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.325.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 0 1 1.37.49l1.296 2.247a1.125 1.125 0 0 1-.26 1.431l-1.003.827c-.293.241-.438.613-.43.992a7 7 0 0 1 0 .255c-.007.378.138.75.43.991l1.004.827c.424.35.534.955.26 1.43l-1.298 2.247a1.125 1.125 0 0 1-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a7 7 0 0 1-.22.128c-.331.183-.581.495-.644.869l-.213 1.281c-.09.543-.56.94-1.11.94h-2.594c-.55 0-1.019-.398-1.11-.94l-.213-1.281c-.062-.374-.312-.686-.644-.87a7 7 0 0 1-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 0 1-1.369-.49l-1.297-2.247a1.125 1.125 0 0 1 .26-1.431l1.004-.827c.292-.24.437-.613.43-.991a7 7 0 0 1 0-.255c.007-.38-.138-.751-.43-.992l-1.004-.827a1.125 1.125 0 0 1-.26-1.43l1.297-2.247a1.125 1.125 0 0 1 1.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.086.22-.128.332-.183.582-.495.644-.869l.214-1.28Z" />
                                <path stroke-linecap="round" stroke-linejoin="round"
                                    d="M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z" />
                            </svg>
                        </button>
                        <div v-if="settingsOpen"
                            class="absolute bottom-full right-0 z-50 mb-1 w-52 rounded-md border border-gray-200 bg-white p-2 shadow-lg dark:border-white/10 dark:bg-gray-800">
                            <div class="space-y-2">
                                <label class="block">
                                    <span class="text-[10px] text-gray-500 dark:text-gray-400">Min peak distance (Hz)</span>
                                    <input v-model.number="settings.minPeakDistance" type="number" min="1" max="1000" step="1"
                                        class="mt-0.5 block w-full rounded border border-gray-200 bg-gray-50 px-1.5 py-0.5 text-[11px] tabular-nums text-gray-700 focus:border-indigo-400 focus:outline-none dark:border-white/10 dark:bg-gray-900 dark:text-gray-300" />
                                </label>
                                <label class="block">
                                    <span class="text-[10px] text-gray-500 dark:text-gray-400">Top peaks shown</span>
                                    <input v-model.number="settings.topCount" type="number" min="1" max="50" step="1"
                                        class="mt-0.5 block w-full rounded border border-gray-200 bg-gray-50 px-1.5 py-0.5 text-[11px] tabular-nums text-gray-700 focus:border-indigo-400 focus:outline-none dark:border-white/10 dark:bg-gray-900 dark:text-gray-300" />
                                </label>
                                <label class="block">
                                    <span class="text-[10px] text-gray-500 dark:text-gray-400">Harmonic overtones</span>
                                    <input v-model.number="settings.numHarmonics" type="number" min="0" max="32" step="1"
                                        class="mt-0.5 block w-full rounded border border-gray-200 bg-gray-50 px-1.5 py-0.5 text-[11px] tabular-nums text-gray-700 focus:border-indigo-400 focus:outline-none dark:border-white/10 dark:bg-gray-900 dark:text-gray-300" />
                                </label>
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
            <p class="text-xs text-gray-500 dark:text-gray-400">Loading spectrum...</p>
        </div>
        <div v-if="componentStatus.loaded && componentStatus.data"
            class="flex items-center gap-4 border-t border-gray-100 px-3 py-1.5 text-[10px] text-gray-500 dark:border-white/5 dark:text-gray-400">
            <span><strong class="font-medium text-gray-700 dark:text-gray-300">{{
                componentStatus.data.frequencies.length.toLocaleString() }}</strong> frequency bins</span>
            <span v-if="peak">Peak: <strong class="font-medium text-gray-700 dark:text-gray-300">{{
                formatFreq(peak.frequency) }}</strong> ({{ peak.magnitude.toFixed(3) }})</span>
            <span v-if="selectedFreq !== null" class="ml-auto text-indigo-500 dark:text-indigo-400">
                Harmonics of <strong>{{ formatFreq(selectedFreq) }}</strong>
                <button @click="selectedFreq = null" class="ml-1 opacity-60 hover:opacity-100">&times;</button>
            </span>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted, watch, nextTick } from "vue";
import VChart from "vue-echarts";
import "echarts";
import type { EChartsOption } from "echarts";

import RangeSelector from "../RangeSelector.vue";
import { notifications } from "../../stores/notifications";
import { type SpectrumData, type FrequencyData } from "../../types";
import { getSpectrum, getTopFrequencies } from "../../utils/tauri-api";
import { spectrumSettings, appliedSettings } from "../../composables/useSpectrumSettings";

const props = defineProps<{
    start: number;
    end: number;
}>();

const settings = spectrumSettings;
const settingsOpen = ref(false);

// Close settings dropdown when clicking outside
function onClickOutside(e: MouseEvent) {
    if (settingsOpen.value && !(e.target as HTMLElement).closest(".relative")) {
        settingsOpen.value = false;
    }
}
onMounted(() => document.addEventListener("click", onClickOutside, true));
onUnmounted(() => document.removeEventListener("click", onClickOutside, true));

type ComponentStatus = {
    loaded: boolean;
    error: string | null;
    data: SpectrumData | null;
};

const componentStatus = reactive<ComponentStatus>({
    loaded: false,
    error: null,
    data: null,
});

const freqRange = ref<[number, number]>([0, 20000]);
const selectedFreq = ref<number | null>(null);
const filterTopByRange = ref(false);

const maxFreq = computed(() => {
    const freqs = componentStatus.data?.frequencies;
    if (!freqs || freqs.length === 0) return 20000;
    return freqs[freqs.length - 1]!.frequency;
});

const loadSpectrumData = async () => {
    componentStatus.loaded = false;

    try {
        const data = await getSpectrum(props.start, props.end, appliedSettings.topCount, appliedSettings.minPeakDistance);
        componentStatus.data = data;
        freqRange.value = [0, data.frequencies.length > 0 ? data.frequencies[data.frequencies.length - 1]!.frequency : 20000];
    } catch (err: any) {
        notifications.error(`Unable to load spectrum data: ${err.message || err}`);
        componentStatus.error = `${err.message || err}`;
    } finally {
        componentStatus.loaded = true;
    }
};

onMounted(async () => {
    await loadSpectrumData();
});

watch([() => props.start, () => props.end], async () => {
    await loadSpectrumData();
});

// Re-fetch when peak-picking settings change (debounced via appliedSettings)
watch([() => appliedSettings.topCount, () => appliedSettings.minPeakDistance], async () => {
    if (componentStatus.data) {
        await loadSpectrumData();
    }
});

// --- Visible frequencies (client-side range filter for chart) ---

const visibleFreqs = computed(() => {
    const freqs = componentStatus.data?.frequencies ?? [];
    const [lo, hi] = freqRange.value;
    return freqs.filter((f) => f.frequency >= lo && f.frequency <= hi);
});

// --- Top frequencies (computed in Rust) ---

const topFrequencies = ref<FrequencyData[]>([]);
const topFreqsLoading = ref(false);

watch(
    [() => componentStatus.data, filterTopByRange, freqRange, () => appliedSettings.topCount, () => appliedSettings.minPeakDistance],
    async () => {
        if (!componentStatus.data) return;

        if (!filterTopByRange.value) {
            topFrequencies.value = componentStatus.data.top_frequencies;
            return;
        }

        topFreqsLoading.value = true;
        try {
            const [lo, hi] = freqRange.value;
            const result = await getTopFrequencies(props.start, props.end, lo, hi, appliedSettings.topCount, appliedSettings.minPeakDistance);
            topFrequencies.value = result;
        } catch (err: any) {
            notifications.error(`Unable to load top frequencies: ${err.message || err}`);
        } finally {
            topFreqsLoading.value = false;
        }
    },
    { immediate: true },
);

// --- Peak ---

const peak = computed(() => {
    const top = componentStatus.data?.top_frequencies;
    if (!top || top.length === 0) return null;
    return top[0];
});

// --- Helpers ---

const formatFreq = (hz: number): string => {
    if (hz < 1000) return Math.round(hz) + " Hz";
    return (hz / 1000).toFixed(1) + " kHz";
};

function onChartClick(params: any) {
    if (params.componentType !== "series") {
        selectedFreq.value = null;
        return;
    }
    const clickedFreq = visibleFreqs.value[params.dataIndex]?.frequency;
    if (!clickedFreq) return;

    if (selectedFreq.value !== null && Math.abs(selectedFreq.value - clickedFreq) < 1) {
        selectedFreq.value = null;
    } else {
        selectedFreq.value = clickedFreq;
    }
}

/** Find the closest visible frequency bin value for a target Hz. Returns as string for category axis. */
function findClosestBin(targetHz: number, freqs: { frequency: number }[]): string | null {
    if (freqs.length === 0) return null;
    const maxVisible = freqs[freqs.length - 1]!.frequency;
    const minVisible = freqs[0]!.frequency;
    if (targetHz > maxVisible || targetHz < minVisible) return null;

    let closest = freqs[0]!;
    for (const f of freqs) {
        if (Math.abs(f.frequency - targetHz) < Math.abs(closest.frequency - targetHz)) {
            closest = f;
        }
    }
    return String(closest.frequency);
}

function buildHarmonicMarkLines(freqs: { frequency: number }[]): any[] {
    if (selectedFreq.value === null || freqs.length === 0) return [];

    const fundamental = selectedFreq.value;
    const lines: any[] = [];

    const fundamentalBin = findClosestBin(fundamental, freqs);
    if (fundamentalBin !== null) {
        lines.push({
            xAxis: fundamentalBin,
            lineStyle: { color: "#ef4444", width: 2, type: "solid" },
            label: { formatter: `F₀ ${formatFreq(fundamental)}`, fontSize: 9, color: "#ef4444" },
        });
    }

    for (let n = 2; n <= appliedSettings.numHarmonics + 1; n++) {
        const harmonicHz = fundamental * n;
        const bin = findClosestBin(harmonicHz, freqs);
        if (bin === null) continue;
        lines.push({
            xAxis: bin,
            lineStyle: { color: "#f97316", width: 1, type: "dashed", opacity: 1 - (n - 2) * 0.08 },
            label: { formatter: `${n}×`, fontSize: 9, color: "#f97316" },
        });
    }

    return lines;
}

// --- Chart option (deferred to avoid freezing) ---

const chartOption = ref<EChartsOption>({});
const chartLoading = ref(false);

watch(
    [visibleFreqs, selectedFreq, () => appliedSettings.numHarmonics],
    async () => {
        chartLoading.value = true;
        await nextTick();

        const freqs = visibleFreqs.value;
        const markLines = buildHarmonicMarkLines(freqs);

        chartOption.value = {
            grid: { top: 8, right: 8, bottom: 24, left: 48 },
            xAxis: {
                type: "category",
                data: freqs.map((f) => f.frequency),
                axisLabel: {
                    formatter: (v: string) => formatFreq(parseFloat(v)),
                    fontSize: 10,
                },
                axisTick: { show: false },
                axisLine: { lineStyle: { color: "#6b7280" } },
            },
            yAxis: {
                type: "value",
                name: "Magnitude",
                nameTextStyle: { fontSize: 10, color: "#6b7280" },
                axisLabel: { fontSize: 10 },
                splitLine: { lineStyle: { color: "#e5e7eb", opacity: 0.3 } },
            },
            series: [{
                type: "line",
                data: freqs.map((f) => f.magnitude),
                showSymbol: false,
                lineStyle: { width: 1, color: "#6366f1" },
                areaStyle: { color: "#6366f1", opacity: 0.08 },
                large: true,
                largeThreshold: 2000,
                markLine: markLines.length > 0 ? {
                    symbol: "none",
                    silent: true,
                    data: markLines,
                } : undefined,
            } as any],
        };

        await nextTick();
        chartLoading.value = false;
    },
    { immediate: true },
);
</script>
