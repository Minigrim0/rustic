<template>
    <div class="flex h-full flex-col overflow-y-auto bg-gray-100 text-gray-900 dark:bg-gray-950 dark:text-gray-100">
        <!-- Header bar -->
        <div class="flex items-center justify-between border-b border-gray-200 px-4 py-2 dark:border-white/10">
            <div>
                <h1 class="text-sm font-semibold">Sample Analyser</h1>
                <p class="text-xs text-gray-500 dark:text-gray-400">Upload audio files for frequency analysis</p>
            </div>
        </div>

        <!-- Content area -->
        <div class="flex-1 space-y-4 p-4">
            <FileUpload @file-selected="handleFileSelected" v-if="!isProcessing && !analysisResult" />

            <!-- Loading -->
            <div v-if="isProcessing" class="flex flex-col items-center gap-3 py-16">
                <svg class="h-6 w-6 animate-spin text-indigo-400" fill="none" viewBox="0 0 24 24">
                    <circle class="opacity-25" cx="12" cy="12" r="10"
                        stroke="currentColor" stroke-width="4" />
                    <path class="opacity-75" fill="currentColor"
                        d="M4 12a8 8 0 0 1 8-8V0C5.373 0 0 5.373 0 12h4z" />
                </svg>
                <p class="text-xs text-gray-500 dark:text-gray-400">Processing audio file...</p>
            </div>

            <!-- Analysis results -->
            <div v-if="audioData && analysisResult" class="space-y-3">
                <!-- Toolbar -->
                <div class="flex items-center justify-between">
                    <h2 class="text-sm font-medium">{{ audioData.name }}</h2>
                    <div class="flex items-center gap-1">
                        <button @click="saveAnalysis"
                            class="rounded px-2 py-1 text-xs text-gray-600 transition-colors hover:bg-gray-200 dark:text-gray-400 dark:hover:bg-white/10">
                            Save
                        </button>
                        <button @click="clearData"
                            class="rounded px-2 py-1 text-xs text-gray-600 transition-colors hover:bg-gray-200 dark:text-gray-400 dark:hover:bg-white/10">
                            Clear
                        </button>
                    </div>
                </div>

                <!-- Info grid -->
                <div class="grid grid-cols-2 gap-px overflow-hidden rounded-lg border border-gray-200 bg-gray-200 sm:grid-cols-3 lg:grid-cols-4 dark:border-white/10 dark:bg-white/10">
                    <div class="flex flex-col gap-0.5 bg-white px-3 py-2 dark:bg-gray-900">
                        <span class="text-[10px] uppercase tracking-wider text-gray-400 dark:text-gray-500">Duration</span>
                        <span class="text-sm font-medium tabular-nums">{{ formatDuration(analysisResult.duration) }}</span>
                    </div>
                    <div class="flex flex-col gap-0.5 bg-white px-3 py-2 dark:bg-gray-900">
                        <span class="text-[10px] uppercase tracking-wider text-gray-400 dark:text-gray-500">Sample Rate</span>
                        <span class="text-sm font-medium tabular-nums">{{ analysisResult.sample_rate.toLocaleString() }} Hz</span>
                    </div>
                    <div class="flex flex-col gap-0.5 bg-white px-3 py-2 dark:bg-gray-900">
                        <span class="text-[10px] uppercase tracking-wider text-gray-400 dark:text-gray-500">Channels</span>
                        <span class="text-sm font-medium tabular-nums">{{ analysisResult.channels }}</span>
                    </div>
                    <div class="flex flex-col gap-0.5 bg-white px-3 py-2 dark:bg-gray-900">
                        <span class="text-[10px] uppercase tracking-wider text-gray-400 dark:text-gray-500">Peak Frequency</span>
                        <span class="text-sm font-medium tabular-nums">{{ formatFrequency(analysisResult.peak_frequency) }}</span>
                    </div>
                    <div class="flex flex-col gap-0.5 bg-white px-3 py-2 dark:bg-gray-900">
                        <span class="text-[10px] uppercase tracking-wider text-gray-400 dark:text-gray-500">RMS Level</span>
                        <span class="text-sm font-medium tabular-nums">{{ (analysisResult.rms_level * 100).toFixed(2) }}%</span>
                    </div>
                    <div v-if="analysisResult.pitch" class="flex flex-col gap-0.5 bg-white px-3 py-2 dark:bg-gray-900">
                        <span class="text-[10px] uppercase tracking-wider text-gray-400 dark:text-gray-500">Pitch</span>
                        <span class="text-sm font-medium tabular-nums">
                            {{ formatFrequency(analysisResult.pitch) }}
                            <span v-if="analysisResult.note" class="ml-1 text-xs font-normal text-indigo-500 dark:text-indigo-400">{{ analysisResult.note }}</span>
                        </span>
                    </div>
                </div>

                <!-- Visualizers -->
                <Visualizers v-if="analysisResult !== null" :audio_summary="analysisResult" />
            </div>
        </div>
    </div>
</template>

<script lang="ts">
import { writeFile, BaseDirectory } from "@tauri-apps/plugin-fs";

import FileUpload from "../components/FileUpload.vue";

import { analyzeAudioFile } from "../utils/tauri-api";
import { notifications } from "../stores/notifications";

import { type AudioSummary } from "../types";
import Visualizers from "../components/Visualizers.vue";

interface AudioFileInfo {
    name: string;
    size: number;
    type: string;
}

export default {
    name: "SampleAnalysis",
    components: {
        FileUpload,
        Visualizers
    },
    computed: {
        // STEP 25: Transform frequency data from backend format to component format
        transformedFrequencies() {
            if (!this.analysisResult || !this.analysisResult.frequencies) {
                console.log(
                    "📊 No frequency data available for transformation",
                );
                return [];
            }

            console.log(
                `📊 Transforming ${this.analysisResult.frequencies.length} frequency data points`,
            );

            // Transform from FrequencyData objects to [frequency, magnitude] pairs
            // Backend returns: { frequency: f32, magnitude: f32, phase: f32 }
            // Component expects: [frequency, magnitude] pairs (phase is ignored)
            const transformed = this.analysisResult.frequencies.map(
                (freqData, index) => {
                    // Handle both possible data structures from backend
                    if (Array.isArray(freqData)) {
                        // If already in array format [freq, mag]
                        return freqData;
                    } else if (
                        freqData.frequency !== undefined &&
                        freqData.magnitude !== undefined
                    ) {
                        // If in FrequencyData object format { frequency: x, magnitude: y, phase: z }
                        return [freqData.frequency, freqData.magnitude];
                    } else {
                        console.warn(
                            `⚠ Unexpected frequency data format at index ${index}:`,
                            freqData,
                        );
                        return [0, 0];
                    }
                },
            );

            // Sort by frequency to ensure proper display
            transformed.sort((a, b) => a[0] - b[0]);

            if (transformed.length > 0) {
                const freqRange = {
                    min: transformed[0][0],
                    max: transformed[transformed.length - 1][0],
                    count: transformed.length,
                };
                console.log(`✓ Frequency transformation complete:`, freqRange);
            }

            return transformed;
        },

        // STEP 26: Validate spectrogram data format
        validatedSpectrogram() {
            if (
                !this.spectrogram ||
                !Array.isArray(this.spectrogram) ||
                this.spectrogram.length === 0
            ) {
                console.log("📈 No spectrogram data available for validation");
                return null;
            }

            console.log(
                `📈 Validating spectrogram with ${this.spectrogram.length} time frames`,
            );

            // Ensure each frame is an array of numbers
            const validation = {
                isArray: Array.isArray(this.spectrogram),
                hasFrames: this.spectrogram.length > 0,
                allFramesAreArrays: this.spectrogram.every((frame) =>
                    Array.isArray(frame),
                ),
                allValuesAreNumbers: this.spectrogram.every(
                    (frame) =>
                        Array.isArray(frame) &&
                        frame.every(
                            (val) => typeof val === "number" && !isNaN(val),
                        ),
                ),
            };

            console.log("📈 Spectrogram validation:", validation);

            if (!validation.isArray || !validation.hasFrames) {
                console.warn("⚠ Spectrogram is not a valid array or is empty");
                return null;
            }

            if (!validation.allFramesAreArrays) {
                console.warn("⚠ Not all spectrogram frames are arrays");
                return null;
            }

            if (!validation.allValuesAreNumbers) {
                console.warn("⚠ Spectrogram contains non-numeric values");
                return null;
            }

            const spectrogramInfo = {
                timeFrames: this.spectrogram.length,
                frequencyBins: this.spectrogram[0].length,
                totalDataPoints:
                    this.spectrogram.length * this.spectrogram[0].length,
            };

            console.log(`✓ Spectrogram validation passed:`, spectrogramInfo);
            return this.spectrogram;
        },
    },

    data(): {
        audioData: AudioFileInfo | null;
        analysisResult: AudioSummary | null;
        errorMessage: string | null;
        isProcessing: boolean;
    } {
        return {
            audioData: null,
            analysisResult: null,
            errorMessage: null,
            isProcessing: false,
        };
    },
    methods: {
        setAudioFileInfo(file: File) {
            this.audioData = {
                name: file.name,
                size: file.size,
                type: file.type || "audio/*",
            };
        },

        async handleFileSelected(file: File) {
            console.log("=== FILE UPLOAD STARTED ===");
            console.log(
                `File selected: ${file.name} (${file.size} bytes, type: ${file.type})`,
            );
            notifications.info("File analysis started");

            this.isProcessing = true;
            this.errorMessage = null;

            try {
                // Process the uploaded file
                this.setAudioFileInfo(file);

                const filePath = await this.saveFileTemporarily(file);

                const analysisResult: AudioSummary = await analyzeAudioFile(filePath);
                this.analysisResult = analysisResult;
            } catch (err: any) {
                console.error("=== FILE UPLOAD FAILED ===");
                console.error("Error details:", err);
                this.handleError(err.message || err);
            } finally {
                this.isProcessing = false;
            }
        },

        async saveFileTemporarily(file: File) {
            const tmpFilePath = `/tmp/${file.name}`;

            try {
                const fileContent = await file.arrayBuffer();
                console.log(`Saving ${file.size} bytes to ${tmpFilePath}...`);

                await writeFile(tmpFilePath, new Uint8Array(fileContent), {
                    baseDir: BaseDirectory.AppConfig,
                });

                console.log("✓ File saved successfully");
                return tmpFilePath;
            } catch (error: any) {
                console.error("✗ Failed to save file temporarily:", error);
                throw new Error(`Failed to save file: ${error.message}`);
            }
        },

        handleError(message: string) {
            console.error("=== APPLICATION ERROR ===");
            console.error("Error message:", message);
            console.error("Timestamp:", new Date().toISOString());
            console.error("Current state:", {
                hasAudioData: !!this.audioData,
                hasAnalysisResult: !!this.analysisResult,
                isProcessing: this.isProcessing,
            });
            console.error("=== END ERROR LOG ===");

            notifications.error(`${message}`);
            this.isProcessing = false;
        },

        clearData() {
            this.audioData = null;
            this.analysisResult = null;
            this.errorMessage = null;
            this.isProcessing = false;
        },

        formatFrequency(freq: number) {
            if (freq < 1000) {
                return `${freq.toFixed(1)} Hz`;
            } else {
                return `${(freq / 1000).toFixed(1)} kHz`;
            }
        },

        formatDuration(seconds: number) {
            const minutes = Math.floor(seconds / 60);
            const remainingSeconds = seconds % 60;
            return `${minutes}:${remainingSeconds.toFixed(2).padStart(5, "0")}`;
        },
    },
}
</script>
