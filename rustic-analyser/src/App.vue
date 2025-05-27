<script>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { writeFile, BaseDirectory } from "@tauri-apps/plugin-fs";
import FileUpload from "./components/FileUpload.vue";
import AudioVisualizer from "./components/AudioVisualizer.vue";
import FrequencyChart from "./components/FrequencyChart.vue";
import SpectrumDisplay from "./components/SpectrumDisplay.vue";

export default {
    name: "App",
    components: {
        FileUpload,
        AudioVisualizer,
        FrequencyChart,
        SpectrumDisplay,
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

        // STEP 22: Get theme display info
        getThemeIcon() {
            return this.currentTheme === "light" ? "🌙" : "☀️";
        },

        getThemeLabel() {
            return this.currentTheme === "light" ? "Dark Mode" : "Light Mode";
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
    data() {
        return {
            audioData: null,
            analysisResult: null,
            errorMessage: null,
            isProcessing: false,
            spectrogram: null, // Add spectrogram data storage
            currentTheme: "light", // Theme state
        };
    },
    methods: {
        // STEP 1: Handle file selection and processing
        async handleFileSelected(file) {
            console.log("=== FILE UPLOAD STARTED ===");
            console.log(
                `File selected: ${file.name} (${file.size} bytes, type: ${file.type})`,
            );

            this.isProcessing = true;
            this.errorMessage = null;

            try {
                // Process the uploaded file
                console.log("Step 1: Saving file temporarily...");
                const filePath = await this.saveFileTemporarily(file);
                console.log(`✓ File saved to: ${filePath}`);

                // STEP 2: Call Tauri backend to analyze the audio file
                console.log("Step 2: Starting main audio analysis...");
                const analysisResult = await this.analyzeAudioFile(filePath);
                console.log("✓ Main analysis completed:", {
                    duration: analysisResult.duration,
                    sampleRate: analysisResult.sample_rate,
                    channels: analysisResult.channels,
                    frequencyDataPoints:
                        analysisResult.frequencies?.length || 0,
                });

                // STEP 2b: If analysis doesn't include frequency data, get it separately
                if (
                    !analysisResult.frequencies ||
                    analysisResult.frequencies.length === 0
                ) {
                    console.log(
                        "⚠ No frequency data in main analysis, fetching separately...",
                    );
                    try {
                        const frequencies = await this.getFrequencyList();
                        if (frequencies && frequencies.length > 0) {
                            analysisResult.frequencies = frequencies;
                            console.log(
                                `✓ Retrieved ${frequencies.length} frequency data points separately`,
                            );
                        } else {
                            console.log(
                                "⚠ No frequency data available from separate call either",
                            );
                        }
                    } catch (error) {
                        console.warn(
                            "⚠ Could not fetch frequency data separately:",
                            error.message,
                        );
                    }
                }

                // STEP 3: Get additional data from backend
                console.log("Step 3: Getting audio samples...");
                const samples = await this.getSamples();
                console.log(
                    `✓ Retrieved ${samples?.length || 0} audio samples`,
                );

                console.log("Step 4: Getting spectrogram data...");
                const spectrogram = await this.getSpectrogram();
                console.log(
                    spectrogram
                        ? `✓ Retrieved spectrogram: ${spectrogram.length} time frames, ${spectrogram[0]?.length || 0} freq bins`
                        : "⚠ No spectrogram data available",
                );

                // STEP 4: Structure the data for components
                console.log("Step 5: Structuring data for UI components...");
                this.audioData = {
                    name: file.name,
                    size: file.size,
                    samples: samples || [], // Ensure we have an array even if empty
                    sample_rate: analysisResult.sample_rate,
                    duration: analysisResult.duration,
                    channels: analysisResult.channels,
                };

                this.analysisResult = analysisResult;
                this.spectrogram = spectrogram;

                // STEP 4b: Log data status for debugging
                this.logDataStatus();
                console.log("✓ Analysis workflow completed successfully");
                console.log("=== FILE UPLOAD COMPLETED ===");
            } catch (err) {
                console.error("=== FILE UPLOAD FAILED ===");
                console.error("Error details:", err);
                this.handleError(err.message || err);
            } finally {
                this.isProcessing = false;
            }
        },

        // STEP 21: Theme Management
        initializeTheme() {
            console.log("🎨 Initializing theme system...");

            // Check for saved theme in localStorage
            const savedTheme = localStorage.getItem("rustic-theme");

            // Check system preference if no saved theme
            const systemPrefersDark = window.matchMedia(
                "(prefers-color-scheme: dark)",
            ).matches;

            // Determine initial theme
            let initialTheme = "light";
            if (savedTheme) {
                initialTheme = savedTheme;
                console.log(`✓ Loaded saved theme: ${savedTheme}`);
            } else if (systemPrefersDark) {
                initialTheme = "dark";
                console.log("✓ Using system dark theme preference");
            } else {
                console.log("✓ Using default light theme");
            }

            this.setTheme(initialTheme);

            // Listen for system theme changes
            window
                .matchMedia("(prefers-color-scheme: dark)")
                .addEventListener("change", (e) => {
                    if (!localStorage.getItem("rustic-theme")) {
                        // Only auto-switch if user hasn't manually set a theme
                        const newTheme = e.matches ? "dark" : "light";
                        console.log(`🎨 System theme changed to: ${newTheme}`);
                        this.setTheme(newTheme);
                    }
                });
        },

        setTheme(theme) {
            console.log(`🎨 Setting theme to: ${theme}`);

            this.currentTheme = theme;

            // Set data attribute on document element for CSS targeting
            document.documentElement.setAttribute("data-theme", theme);

            // Save theme preference
            localStorage.setItem("rustic-theme", theme);

            console.log(`✓ Theme applied: ${theme}`);
        },

        toggleTheme() {
            const newTheme = this.currentTheme === "light" ? "dark" : "light";
            console.log(
                `🎨 Toggling theme from ${this.currentTheme} to ${newTheme}`,
            );
            this.setTheme(newTheme);
        },

        // STEP 5: Save file temporarily for backend access
        async saveFileTemporarily(file) {
            const tmpFilePath = `/tmp/${file.name}`;

            try {
                const fileContent = await file.arrayBuffer();
                console.log(`Saving ${file.size} bytes to ${tmpFilePath}...`);

                await writeFile(tmpFilePath, new Uint8Array(fileContent), {
                    baseDir: BaseDirectory.AppConfig,
                });

                console.log("✓ File saved successfully");
                return tmpFilePath;
            } catch (error) {
                console.error("✗ Failed to save file temporarily:", error);
                throw new Error(`Failed to save file: ${error.message}`);
            }
        },

        // STEP 6: Call Tauri command to analyze audio file
        async analyzeAudioFile(filePath) {
            console.log("=== BACKEND ANALYSIS START ===");
            console.log(`Calling analyze_audio_file with path: ${filePath}`);

            try {
                const startTime = Date.now();
                const result = await invoke("analyze_audio_file", {
                    path: filePath,
                });
                const duration = Date.now() - startTime;

                console.log(`✓ Analysis completed in ${duration}ms`);
                console.log("Analysis result structure:", {
                    hasSampleRate: !!result.sample_rate,
                    hasDuration: !!result.duration,
                    hasChannels: !!result.channels,
                    hasPeakFrequency: !!result.peak_frequency,
                    hasRmsLevel: !!result.rms_level,
                    hasPitch: !!result.pitch,
                    hasNote: !!result.note,
                    frequenciesType: Array.isArray(result.frequencies)
                        ? "array"
                        : typeof result.frequencies,
                    frequenciesLength: result.frequencies?.length || 0,
                });

                // Validate the result structure
                if (!result.frequencies || !Array.isArray(result.frequencies)) {
                    console.warn(
                        "⚠ Invalid frequency data received from backend",
                    );
                    result.frequencies = [];
                } else if (result.frequencies.length > 0) {
                    console.log(
                        "✓ Frequency data sample:",
                        result.frequencies[0],
                    );
                }

                console.log("=== BACKEND ANALYSIS END ===");
                return result;
            } catch (error) {
                console.error("=== BACKEND ANALYSIS FAILED ===");
                console.error("Error type:", error.constructor.name);
                console.error("Error message:", error.message);
                console.error("Full error:", error);
                throw new Error(`Analysis failed: ${error.message || error}`);
            }
        },

        // STEP 7: Get audio samples from backend
        async getSamples() {
            console.log("Fetching audio samples from backend...");

            try {
                const startTime = Date.now();
                const samples = await invoke("get_samples");
                const duration = Date.now() - startTime;

                if (!Array.isArray(samples)) {
                    console.warn(
                        "⚠ Expected array of samples, got:",
                        typeof samples,
                    );
                    return [];
                }

                console.log(
                    `✓ Retrieved ${samples.length} audio samples in ${duration}ms`,
                );

                if (samples.length > 0) {
                    const sampleStats = {
                        min: Math.min(...samples),
                        max: Math.max(...samples),
                        first: samples[0],
                        last: samples[samples.length - 1],
                    };
                    console.log("Sample statistics:", sampleStats);
                }

                return samples;
            } catch (error) {
                console.error("✗ Error getting samples:", error.message);
                console.warn(
                    "⚠ Continuing without sample data - waveform won't be available",
                );
                return [];
            }
        },

        // STEP 8: Get spectrogram data from backend
        async getSpectrogram() {
            try {
                const spectrogram = await invoke("compute_spectrum_command");

                if (!Array.isArray(spectrogram)) {
                    console.warn(
                        "Expected 2D array for spectrogram, got:",
                        typeof spectrogram,
                    );
                    return null;
                }

                if (spectrogram.length === 0) {
                    console.warn("Received empty spectrogram data");
                    return null;
                }

                // Validate that it's a 2D array
                const isValid2D = spectrogram.every((frame) =>
                    Array.isArray(frame),
                );
                if (!isValid2D) {
                    console.warn("Spectrogram data is not a proper 2D array");
                    return null;
                }

                console.log(
                    `Retrieved spectrogram: ${spectrogram.length} time frames, ${spectrogram[0]?.length || 0} frequency bins`,
                );
                return spectrogram;
            } catch (error) {
                console.error("Error getting spectrogram:", error);
                // Return null if spectrogram fails - it's not critical
                return null;
            }
        },

        // STEP 9: Get FFT data (alternative method)
        async getFFTData() {
            try {
                const fftData = await invoke("compute_fft_command");
                console.log(`Retrieved ${fftData.length} FFT data points`);
                return fftData;
            } catch (error) {
                console.error("Error getting FFT data:", error);
                throw new Error(`Failed to get FFT data: ${error}`);
            }
        },

        // STEP 9b: Get frequency list (alternative to FFT)
        async getFrequencyList() {
            try {
                const frequencies = await invoke("list_frequencies");
                console.log(
                    `Retrieved frequency list with ${frequencies.length} entries`,
                );
                return frequencies;
            } catch (error) {
                console.error("Error getting frequency list:", error);
                return null;
            }
        },

        // STEP 10: Estimate pitch using backend
        async estimatePitch() {
            try {
                const pitch = await invoke("estimate_pitch_command");
                console.log("Pitch estimation:", pitch);
                return pitch;
            } catch (error) {
                console.error("Error estimating pitch:", error);
                return null;
            }
        },

        // STEP 11: Convert frequency to note name
        async frequencyToNote(frequency) {
            try {
                const note = await invoke("frequency_to_note_command", {
                    frequency: frequency,
                });
                return note;
            } catch (error) {
                console.error("Error converting frequency to note:", error);
                return null;
            }
        },

        // STEP 12: Save analysis results
        async saveAnalysis() {
            if (!this.analysisResult) {
                this.handleError("No analysis results to save");
                return;
            }

            try {
                const fileName = `analysis_${Date.now()}.json`;
                const filePath = `/tmp/${fileName}`;

                await invoke("save_analysis", {
                    path: filePath,
                    result: this.analysisResult,
                });

                console.log(`Analysis saved to ${filePath}`);

                // You might want to show a success message to the user
                alert(`Analysis saved successfully to ${fileName}`);
            } catch (error) {
                console.error("Error saving analysis:", error);
                this.handleError(`Failed to save analysis: ${error}`);
            }
        },

        // STEP 13: Error handling
        handleError(message) {
            console.error("=== APPLICATION ERROR ===");
            console.error("Error message:", message);
            console.error("Timestamp:", new Date().toISOString());
            console.error("Current state:", {
                hasAudioData: !!this.audioData,
                hasAnalysisResult: !!this.analysisResult,
                hasSpectrogram: !!this.spectrogram,
                isProcessing: this.isProcessing,
            });
            console.error("=== END ERROR LOG ===");

            this.errorMessage = `${message}`;
            this.isProcessing = false;

            // Auto-dismiss error after 10 seconds
            setTimeout(() => {
                if (this.errorMessage === message) {
                    this.errorMessage = null;
                }
            }, 10000);
        },

        // STEP 14: Clear all data
        clearData() {
            this.audioData = null;
            this.analysisResult = null;
            this.spectrogram = null;
            this.errorMessage = null;
            this.isProcessing = false;
        },

        // STEP 15: Format frequency for display
        formatFrequency(freq) {
            if (freq < 1000) {
                return `${freq.toFixed(1)} Hz`;
            } else {
                return `${(freq / 1000).toFixed(1)} kHz`;
            }
        },

        // STEP 16: Format duration for display
        formatDuration(seconds) {
            const minutes = Math.floor(seconds / 60);
            const remainingSeconds = seconds % 60;
            return `${minutes}:${remainingSeconds.toFixed(2).padStart(5, "0")}`;
        },

        // STEP 17: Refresh frequency data
        async refreshFrequencyData() {
            if (!this.analysisResult) {
                console.warn(
                    "⚠ Cannot refresh frequency data - no analysis result available",
                );
                return;
            }

            console.log("🔄 Refreshing frequency data...");
            this.isProcessing = true;

            try {
                const startTime = Date.now();
                const frequencies = await this.getFrequencyList();
                const duration = Date.now() - startTime;

                if (frequencies && frequencies.length > 0) {
                    this.analysisResult.frequencies = frequencies;
                    console.log(
                        `✅ Refresh successful: Updated with ${frequencies.length} frequency data points in ${duration}ms`,
                    );
                } else {
                    console.log(
                        "⚠ Refresh completed but no frequency data received",
                    );
                }
            } catch (error) {
                console.error("✗ Failed to refresh frequency data:", error);
                this.handleError(
                    `Failed to refresh frequency data: ${error.message}`,
                );
            } finally {
                this.isProcessing = false;
            }
        },

        // STEP 19: Validate application state
        validateApplicationState() {
            console.log("🔍 Validating application state...");

            const checks = {
                hasFileData: !!this.audioData,
                hasAnalysisResult: !!this.analysisResult,
                hasSamples: !!(this.audioData?.samples?.length > 0),
                hasFrequencies: !!(
                    this.analysisResult?.frequencies?.length > 0
                ),
                hasSpectrogram: !!this.spectrogram,
                isProcessing: this.isProcessing,
                hasErrors: !!this.errorMessage,
            };

            const issues = [];

            if (checks.hasFileData && !checks.hasSamples) {
                issues.push(
                    "Audio samples missing - waveform visualization unavailable",
                );
            }

            if (checks.hasAnalysisResult && !checks.hasFrequencies) {
                issues.push(
                    "Frequency data missing - spectrum visualization unavailable",
                );
            }

            if (checks.hasAnalysisResult && !checks.hasSpectrogram) {
                issues.push(
                    "Spectrogram data missing - time-frequency visualization unavailable",
                );
            }

            console.log("State validation results:", checks);

            if (issues.length > 0) {
                console.warn("⚠ Validation issues found:", issues);
            } else if (checks.hasFileData) {
                console.log(
                    "✅ Application state is healthy - all data available",
                );
            }

            return { checks, issues };
        },

        // STEP 20: Export debug information
        exportDebugInfo() {
            const debugInfo = {
                timestamp: new Date().toISOString(),
                userAgent: navigator.userAgent,
                applicationState: this.validateApplicationState(),
                audioData: this.audioData
                    ? {
                          name: this.audioData.name,
                          size: this.audioData.size,
                          sampleRate: this.audioData.sample_rate,
                          duration: this.audioData.duration,
                          channels: this.audioData.channels,
                          samplesLength: this.audioData.samples?.length || 0,
                      }
                    : null,
                analysisResult: this.analysisResult
                    ? {
                          sampleRate: this.analysisResult.sample_rate,
                          duration: this.analysisResult.duration,
                          channels: this.analysisResult.channels,
                          peakFrequency: this.analysisResult.peak_frequency,
                          rmsLevel: this.analysisResult.rms_level,
                          pitch: this.analysisResult.pitch,
                          note: this.analysisResult.note,
                          frequenciesLength:
                              this.analysisResult.frequencies?.length || 0,
                      }
                    : null,
                spectrogram: this.spectrogram
                    ? {
                          timeFrames: this.spectrogram.length,
                          frequencyBins: this.spectrogram[0]?.length || 0,
                      }
                    : null,
                uiState: {
                    isProcessing: this.isProcessing,
                    hasError: !!this.errorMessage,
                    errorMessage: this.errorMessage,
                },
            };

            console.log("🔧 Debug information exported:", debugInfo);

            // Create downloadable debug file
            const debugJson = JSON.stringify(debugInfo, null, 2);
            const blob = new Blob([debugJson], { type: "application/json" });
            const url = URL.createObjectURL(blob);
            const a = document.createElement("a");
            a.href = url;
            a.download = `rustic-debug-${Date.now()}.json`;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(url);

            console.log("📥 Debug file downloaded");
            return debugInfo;
        },

        // STEP 18: Debug data availability
        logDataStatus() {
            console.log("=== COMPREHENSIVE DATA STATUS ===");
            console.log("Timestamp:", new Date().toISOString());

            console.log("📁 Audio File Data:", {
                available: !!this.audioData,
                name: this.audioData?.name,
                size: this.audioData?.size,
                sampleRate: this.audioData?.sample_rate,
                duration: this.audioData?.duration,
                channels: this.audioData?.channels,
                samplesCount: this.audioData?.samples?.length || 0,
                samplesType: Array.isArray(this.audioData?.samples)
                    ? "array"
                    : typeof this.audioData?.samples,
            });

            console.log("📊 Analysis Results:", {
                available: !!this.analysisResult,
                peakFrequency: this.analysisResult?.peak_frequency,
                rmsLevel: this.analysisResult?.rms_level,
                pitch: this.analysisResult?.pitch,
                note: this.analysisResult?.note,
                frequenciesCount: this.analysisResult?.frequencies?.length || 0,
                frequenciesType: Array.isArray(this.analysisResult?.frequencies)
                    ? "array"
                    : typeof this.analysisResult?.frequencies,
            });

            if (this.analysisResult?.frequencies?.length > 0) {
                const firstFreq = this.analysisResult.frequencies[0];
                const lastFreq =
                    this.analysisResult.frequencies[
                        this.analysisResult.frequencies.length - 1
                    ];
                console.log("🎵 Frequency Range:", {
                    first: firstFreq,
                    last: lastFreq,
                    firstType: typeof firstFreq,
                    hasFrequencyProperty: firstFreq?.frequency !== undefined,
                    hasMagnitudeProperty: firstFreq?.magnitude !== undefined,
                    hasPhaseProperty: firstFreq?.phase !== undefined,
                });
            }

            console.log("📈 Spectrogram Data:", {
                available: !!this.spectrogram,
                isArray: Array.isArray(this.spectrogram),
                timeFrames: this.spectrogram?.length || 0,
                frequencyBins: this.spectrogram?.[0]?.length || 0,
                is2DArray: this.spectrogram
                    ? this.spectrogram.every((frame) => Array.isArray(frame))
                    : false,
            });

            console.log("🎨 UI Component Data:", {
                transformedFrequencies:
                    this.transformedFrequencies?.length || 0,
                validatedSpectrogram: !!this.validatedSpectrogram,
                isProcessing: this.isProcessing,
                hasError: !!this.errorMessage,
            });

            console.log("=== END COMPREHENSIVE STATUS ===");
        },
    },
};
</script>

<template>
    <div class="app-container">
        <header>
            <div class="header-content">
                <div class="title-section">
                    <h1 class="main-title">Rustic Sample Analyser</h1>
                    <p>
                        Upload audio files for comprehensive frequency analysis
                    </p>
                </div>
                <div class="header-controls">
                    <button
                        @click="toggleTheme"
                        class="theme-toggle"
                        :title="getThemeLabel"
                    >
                        {{ getThemeIcon }} {{ getThemeLabel }}
                    </button>
                </div>
            </div>
        </header>

        <main>
            <!-- STEP 17: File upload component -->
            <FileUpload @file-selected="handleFileSelected" />

            <!-- STEP 18: Error display -->
            <div v-if="errorMessage" class="error-message">
                <p>Error: {{ errorMessage }}</p>
                <button @click="errorMessage = null" class="secondary-button">
                    Dismiss
                </button>
            </div>

            <!-- STEP 19: Loading indicator -->
            <div v-if="isProcessing" class="loading">
                <div class="spinner"></div>
                <p>Processing audio file...</p>
            </div>

            <!-- STEP 20: Analysis results display -->
            <div v-if="audioData && analysisResult" class="analysis-container">
                <div class="analysis-header">
                    <h2>Analysis Results: {{ audioData.name }}</h2>
                    <div class="header-buttons">
                        <button
                            @click="refreshFrequencyData"
                            class="secondary-button"
                            :disabled="isProcessing"
                        >
                            🔄 Refresh Frequency Data
                        </button>
                        <button @click="logDataStatus" class="secondary-button">
                            📊 Debug Info
                        </button>
                        <button
                            @click="exportDebugInfo"
                            class="secondary-button"
                        >
                            🔧 Export Debug
                        </button>
                        <button @click="saveAnalysis" class="secondary-button">
                            💾 Save Analysis
                        </button>
                    </div>
                </div>

                <!-- STEP 21: Basic audio information -->
                <div class="analysis-info">
                    <div class="info-grid">
                        <div class="info-item">
                            <label>Duration:</label>
                            <span>{{
                                formatDuration(analysisResult.duration)
                            }}</span>
                        </div>
                        <div class="info-item">
                            <label>Sample Rate:</label>
                            <span
                                >{{
                                    analysisResult.sample_rate.toLocaleString()
                                }}
                                Hz</span
                            >
                        </div>
                        <div class="info-item">
                            <label>Channels:</label>
                            <span>{{ analysisResult.channels }}</span>
                        </div>
                        <div class="info-item">
                            <label>Peak Frequency:</label>
                            <span>{{
                                formatFrequency(analysisResult.peak_frequency)
                            }}</span>
                        </div>
                        <div class="info-item">
                            <label>RMS Level:</label>
                            <span
                                >{{
                                    (analysisResult.rms_level * 100).toFixed(2)
                                }}%</span
                            >
                        </div>
                        <div v-if="analysisResult.pitch" class="info-item">
                            <label>Estimated Pitch:</label>
                            <span>
                                {{ formatFrequency(analysisResult.pitch) }}
                                <span
                                    v-if="analysisResult.note"
                                    class="note-name"
                                >
                                    ({{ analysisResult.note }})
                                </span>
                            </span>
                        </div>
                        <div class="info-item">
                            <label>Frequency Data Points:</label>
                            <span>{{
                                analysisResult.frequencies
                                    ? analysisResult.frequencies.length
                                    : 0
                            }}</span>
                        </div>
                    </div>
                </div>

                <!-- STEP 22: Visualizations -->
                <div class="visualizations">
                    <!-- Waveform visualization -->
                    <div
                        v-if="audioData.samples && audioData.samples.length > 0"
                        class="visualization-card"
                    >
                        <h3>Waveform</h3>
                        <AudioVisualizer
                            :samples="audioData.samples"
                            :sampleRate="audioData.sample_rate"
                        />
                    </div>

                    <!-- Show message if no samples data -->
                    <div v-else class="visualization-card">
                        <h3>Waveform</h3>
                        <div class="no-data-message">
                            <p>No waveform data available</p>
                            <p class="help-text">
                                Audio samples could not be loaded
                            </p>
                        </div>
                    </div>

                    <!-- Frequency spectrum visualization -->
                    <div
                        v-if="
                            transformedFrequencies &&
                            transformedFrequencies.length > 0
                        "
                        class="visualization-card"
                    >
                        <h3>Frequency Spectrum</h3>
                        <FrequencyChart
                            :frequencies="transformedFrequencies"
                            :sampleRate="audioData.sample_rate"
                        />
                    </div>

                    <!-- Show message if no frequency data -->
                    <div v-else class="visualization-card">
                        <h3>Frequency Spectrum</h3>
                        <div class="no-data-message">
                            <p>No frequency data available</p>
                            <button
                                @click="refreshFrequencyData"
                                class="secondary-button"
                                :disabled="isProcessing"
                            >
                                Load Frequency Data
                            </button>
                        </div>
                    </div>

                    <!-- Spectrogram visualization (if available) -->
                    <div v-if="validatedSpectrogram" class="visualization-card">
                        <h3>Spectrogram</h3>
                        <SpectrumDisplay
                            :spectrogram="validatedSpectrogram"
                            :minFrequency="20.0"
                            :maxFrequency="
                                Math.min(20000.0, audioData.sample_rate / 2)
                            "
                            :sampleRate="audioData.sample_rate"
                        />
                    </div>

                    <!-- Show message if no spectrogram data -->
                    <div v-else class="visualization-card">
                        <h3>Spectrogram</h3>
                        <div class="no-data-message">
                            <p>Spectrogram analysis not available</p>
                            <p class="help-text">
                                Time-frequency analysis could not be computed
                            </p>
                        </div>
                    </div>
                </div>

                <!-- STEP 23: Action buttons -->
                <div class="action-buttons">
                    <button @click="clearData" class="primary-button">
                        Clear Data
                    </button>
                </div>
            </div>
        </main>

        <footer>
            <p>© 2024 Rustic Sample Analyser - Powered by Tauri & Rust</p>
            <p class="help-text">
                💡 Having issues? Use the Debug Info button to check data
                status, or Export Debug to save diagnostic information.
            </p>
        </footer>
    </div>
</template>
