<template>
    <div class="space-y-2">
        <!-- Time range selector -->
        <div class="rounded-lg border border-gray-200 bg-white px-3 py-2 dark:border-white/10 dark:bg-gray-900">
            <span class="text-[10px] uppercase tracking-wider text-gray-400 dark:text-gray-500">Time Range</span>
            <RangeSelector
                v-model="timeRange"
                :min="0"
                :max="audio_summary.duration"
                :step="0.001"
                :format-value="(v: number) => v.toFixed(3) + 's'"
            />
        </div>

        <!-- Waveform -->
        <section class="overflow-hidden rounded-lg border border-gray-200 bg-white dark:border-white/10 dark:bg-gray-900">
            <button @click="waveformOpen = !waveformOpen"
                class="flex w-full items-center gap-2 px-3 py-1.5 text-left text-xs font-medium text-gray-600 transition-colors hover:bg-gray-50 dark:text-gray-400 dark:hover:bg-white/5">
                <svg class="h-3 w-3 transition-transform" :class="{ 'rotate-90': waveformOpen }"
                    fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                    <path stroke-linecap="round" stroke-linejoin="round" d="m9 5 7 7-7 7" />
                </svg>
                Waveform
            </button>
            <div v-show="waveformOpen" class="border-t border-gray-100 dark:border-white/5">
                <AudioVisualizer :summary="audio_summary" :start="timeRange[0]" :end="timeRange[1]" />
            </div>
        </section>

        <!-- Frequency Spectrum -->
        <section class="overflow-hidden rounded-lg border border-gray-200 bg-white dark:border-white/10 dark:bg-gray-900">
            <button @click="spectrumOpen = !spectrumOpen"
                class="flex w-full items-center gap-2 px-3 py-1.5 text-left text-xs font-medium text-gray-600 transition-colors hover:bg-gray-50 dark:text-gray-400 dark:hover:bg-white/5">
                <svg class="h-3 w-3 transition-transform" :class="{ 'rotate-90': spectrumOpen }"
                    fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                    <path stroke-linecap="round" stroke-linejoin="round" d="m9 5 7 7-7 7" />
                </svg>
                Frequency Spectrum
            </button>
            <div v-show="spectrumOpen" class="border-t border-gray-100 dark:border-white/5">
                <FrequencyChart :start="timeRange[0]" :end="timeRange[1]" />
            </div>
        </section>

        <!-- Spectrogram -->
        <section class="overflow-hidden rounded-lg border border-gray-200 bg-white dark:border-white/10 dark:bg-gray-900">
            <button @click="spectrogramOpen = !spectrogramOpen"
                class="flex w-full items-center gap-2 px-3 py-1.5 text-left text-xs font-medium text-gray-600 transition-colors hover:bg-gray-50 dark:text-gray-400 dark:hover:bg-white/5">
                <svg class="h-3 w-3 transition-transform" :class="{ 'rotate-90': spectrogramOpen }"
                    fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                    <path stroke-linecap="round" stroke-linejoin="round" d="m9 5 7 7-7 7" />
                </svg>
                Spectrogram
            </button>
            <div v-show="spectrogramOpen" class="border-t border-gray-100 dark:border-white/5">
                <SpectrumDisplay />
            </div>
        </section>
    </div>
</template>

<script setup lang="ts">
import { ref } from "vue";

import RangeSelector from "./RangeSelector.vue";
import AudioVisualizer from "./visualisers/AudioVisualizer.vue";
import FrequencyChart from "./visualisers/FrequencyChart.vue";
import SpectrumDisplay from "./visualisers/SpectrumDisplay.vue";

import { type AudioSummary } from "../types";

const waveformOpen = ref(true);
const spectrumOpen = ref(true);
const spectrogramOpen = ref(true);

const props = defineProps<{
    audio_summary: AudioSummary;
}>();

const timeRange = ref<[number, number]>([0, props.audio_summary.duration]);
</script>