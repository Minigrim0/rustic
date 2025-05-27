<template>
    <div class="spectrum-display">
        <!-- Canvas for spectrogram visualization -->
        <canvas
            ref="canvas"
            width="800"
            height="400"
            class="spectrogram-canvas"
            :title="`Spectrogram: ${timeFrames} time frames × ${frequencyBins} frequency bins`"
        ></canvas>

        <!-- Spectrogram controls -->
        <div class="spectrum-controls">
            <div class="control-group">
                <label class="control-label">Color Map:</label>
                <select v-model="colorMapType" class="color-map-select">
                    <option value="heat">Heat (Red-Yellow-White)</option>
                    <option value="plasma">Plasma (Purple-Pink-Yellow)</option>
                    <option value="viridis">Viridis (Purple-Blue-Green)</option>
                    <option value="grayscale">Grayscale</option>
                </select>
            </div>

            <div class="control-group">
                <label class="control-label">Scale:</label>
                <button
                    @click="toggleFreqScale"
                    class="scale-toggle"
                    :class="{ active: isLogFreqScale }"
                >
                    {{ isLogFreqScale ? "Log" : "Linear" }}
                </button>

                <button
                    @click="toggleIntensity"
                    class="intensity-toggle"
                    :class="{ active: enhanceContrast }"
                >
                    {{ enhanceContrast ? "Enhanced" : "Normal" }}
                </button>
            </div>
        </div>

        <!-- Spectrogram info display -->
        <div class="spectrum-info">
            <div class="info-row">
                <span class="info-label">Time Range:</span>
                <span class="info-value"
                    >{{ formatTime(0) }} - {{ formatTime(totalDuration) }}</span
                >

                <span class="info-label">Freq Range:</span>
                <span class="info-value"
                    >{{ formatFrequency(actualMinFreq) }} -
                    {{ formatFrequency(actualMaxFreq) }}</span
                >

                <span class="info-label">Resolution:</span>
                <span class="info-value"
                    >{{ timeFrames }} × {{ frequencyBins }}</span
                >
            </div>

            <div class="info-row">
                <span class="info-label">Time Res:</span>
                <span class="info-value"
                    >{{ formatTime(timeResolution) }}/frame</span
                >

                <span class="info-label">Freq Res:</span>
                <span class="info-value"
                    >{{ formatFrequency(frequencyResolution) }}/bin</span
                >

                <span class="info-label">Peak:</span>
                <span class="info-value"
                    >{{ formatFrequency(peakFrequency) }} @
                    {{ formatTime(peakTime) }}</span
                >
            </div>
        </div>

        <!-- Color scale legend -->
        <div class="color-legend">
            <div
                class="legend-gradient"
                :style="{ background: colorGradient }"
            ></div>
            <div class="legend-labels">
                <span>Low</span>
                <span>Intensity</span>
                <span>High</span>
            </div>
        </div>
    </div>
</template>

<script>
/**
 * SpectrumDisplay Component
 *
 * Renders spectrogram (time-frequency analysis) with dynamic scaling.
 * Features:
 * - Dynamic frequency range based on actual data and sample rate
 * - Multiple color mapping schemes with theme awareness
 * - Linear and logarithmic frequency scaling
 * - Contrast enhancement options
 * - Real-time peak detection across time-frequency domain
 * - Responsive rendering with performance optimization
 * - Detailed analysis information display
 */
export default {
    name: "SpectrumDisplay",

    props: {
        /**
         * 2D array of spectrogram data [timeFrame][frequencyBin]
         * Each inner array represents frequency magnitudes for a time frame
         */
        spectrogram: {
            type: Array,
            required: true,
            default: () => [],
        },

        /**
         * Minimum frequency to consider (Hz)
         */
        minFrequency: {
            type: Number,
            default: 20.0,
        },

        /**
         * Maximum frequency to consider (Hz)
         */
        maxFrequency: {
            type: Number,
            default: 20000.0,
        },

        /**
         * Sample rate of the audio (Hz)
         */
        sampleRate: {
            type: Number,
            required: true,
            default: 44100,
        },
    },

    data() {
        return {
            // Canvas rendering context
            ctx: null,
            // Visualization options
            colorMapType: "heat",
            isLogFreqScale: false,
            enhanceContrast: true,
            // Calculated dimensions
            timeFrames: 0,
            frequencyBins: 0,
            // Calculated ranges
            actualMinFreq: 20,
            actualMaxFreq: 20000,
            totalDuration: 0,
            timeResolution: 0,
            frequencyResolution: 0,
            // Peak analysis
            peakFrequency: 0,
            peakTime: 0,
            peakMagnitude: 0,
            // Performance
            resizeObserver: null,
            animationFrameId: null,
            // Processed data cache
            processedData: null,
            maxMagnitude: 0,
        };
    },

    computed: {
        /**
         * Check if we have valid spectrogram data
         */
        hasValidData() {
            return (
                Array.isArray(this.spectrogram) &&
                this.spectrogram.length > 0 &&
                this.spectrogram.every(
                    (frame) =>
                        Array.isArray(frame) &&
                        frame.length > 0 &&
                        frame.every(
                            (val) => typeof val === "number" && !isNaN(val),
                        ),
                )
            );
        },

        /**
         * Generate CSS gradient for color legend
         */
        colorGradient() {
            const colors = this.getColorMapColors();
            return `linear-gradient(to right, ${colors.join(", ")})`;
        },

        /**
         * Calculate effective frequency range for display
         */
        displayFreqRange() {
            const nyquist = this.sampleRate / 2;
            const minFreq = Math.max(this.minFrequency, 20);
            const maxFreq = Math.min(this.maxFrequency, nyquist);

            return { min: minFreq, max: maxFreq };
        },
    },

    mounted() {
        console.log("📈 SpectrumDisplay mounted");
        this.initializeCanvas();
        this.setupResizeObserver();
        this.analyzeSpectrogramData();
        this.renderSpectrogram();
    },

    beforeUnmount() {
        console.log("📈 SpectrumDisplay cleanup");
        this.cleanup();
    },

    watch: {
        // Re-render when spectrogram data changes
        spectrogram: {
            handler() {
                console.log(
                    `📈 Spectrogram updated: ${this.spectrogram.length} time frames`,
                );
                this.analyzeSpectrogramData();
                this.renderSpectrogram();
            },
            immediate: false,
        },

        // Re-render when sample rate changes
        sampleRate() {
            console.log(`📈 Sample rate updated: ${this.sampleRate}Hz`);
            this.analyzeSpectrogramData();
            this.renderSpectrogram();
        },

        // Re-render when frequency range changes
        minFrequency() {
            this.updateFrequencyRange();
        },
        maxFrequency() {
            this.updateFrequencyRange();
        },

        // Re-render when visualization options change
        colorMapType() {
            console.log(`📈 Color map: ${this.colorMapType}`);
            this.renderSpectrogram();
        },

        isLogFreqScale() {
            console.log(
                `📈 Frequency scale: ${this.isLogFreqScale ? "logarithmic" : "linear"}`,
            );
            this.renderSpectrogram();
        },

        enhanceContrast() {
            console.log(
                `📈 Contrast enhancement: ${this.enhanceContrast ? "enabled" : "disabled"}`,
            );
            this.renderSpectrogram();
        },
    },

    methods: {
        /**
         * Initialize canvas context and setup
         */
        initializeCanvas() {
            const canvas = this.$refs.canvas;
            if (!canvas) {
                console.error("❌ Canvas ref not found");
                return;
            }

            this.ctx = canvas.getContext("2d");
            if (!this.ctx) {
                console.error("❌ Could not get canvas 2D context");
                return;
            }

            // Set canvas resolution for high-DPI displays
            const dpr = window.devicePixelRatio || 1;
            const rect = canvas.getBoundingClientRect();

            canvas.width = rect.width * dpr;
            canvas.height = rect.height * dpr;

            this.ctx.scale(dpr, dpr);
            canvas.style.width = rect.width + "px";
            canvas.style.height = rect.height + "px";

            console.log("✅ Canvas initialized:", {
                width: canvas.width,
                height: canvas.height,
                dpr: dpr,
            });
        },

        /**
         * Setup resize observer for responsive canvas
         */
        setupResizeObserver() {
            if (!window.ResizeObserver) return;

            this.resizeObserver = new ResizeObserver(() => {
                if (this.animationFrameId) {
                    cancelAnimationFrame(this.animationFrameId);
                }

                this.animationFrameId = requestAnimationFrame(() => {
                    this.initializeCanvas();
                    this.renderSpectrogram();
                });
            });

            this.resizeObserver.observe(this.$refs.canvas);
        },

        /**
         * Analyze spectrogram data to extract dimensions and statistics
         */
        analyzeSpectrogramData() {
            if (!this.hasValidData) {
                this.resetAnalysis();
                return;
            }

            // Calculate dimensions
            this.timeFrames = this.spectrogram.length;
            this.frequencyBins = this.spectrogram[0].length;

            // Calculate time parameters
            // Assume hop size is half the window size for typical STFT
            const hopSizeSeconds =
                (this.frequencyBins * 2) / (this.sampleRate * 2);
            this.totalDuration = this.timeFrames * hopSizeSeconds;
            this.timeResolution = hopSizeSeconds;

            // Calculate frequency parameters
            this.frequencyResolution =
                this.sampleRate / (2 * this.frequencyBins);
            this.actualMinFreq = Math.max(
                this.minFrequency,
                this.frequencyResolution,
            );
            this.actualMaxFreq = Math.min(
                this.maxFrequency,
                this.sampleRate / 2,
            );

            // Find overall maximum magnitude for normalization
            this.maxMagnitude = 0;
            let peakMag = 0;
            let peakTimeIdx = 0;
            let peakFreqIdx = 0;

            for (let timeIdx = 0; timeIdx < this.timeFrames; timeIdx++) {
                const frame = this.spectrogram[timeIdx];
                for (let freqIdx = 0; freqIdx < this.frequencyBins; freqIdx++) {
                    const magnitude = Math.abs(frame[freqIdx]);
                    this.maxMagnitude = Math.max(this.maxMagnitude, magnitude);

                    // Track peak for display
                    if (magnitude > peakMag) {
                        peakMag = magnitude;
                        peakTimeIdx = timeIdx;
                        peakFreqIdx = freqIdx;
                    }
                }
            }

            // Calculate peak location
            this.peakTime = peakTimeIdx * this.timeResolution;
            this.peakFrequency = peakFreqIdx * this.frequencyResolution;
            this.peakMagnitude = peakMag;

            // Cache processed data for rendering
            this.processedData = this.preprocessSpectrogramData();

            console.log("📈 Spectrogram analysis:", {
                dimensions: `${this.timeFrames} × ${this.frequencyBins}`,
                duration: `${this.totalDuration.toFixed(2)}s`,
                freqRange: `${this.actualMinFreq.toFixed(1)}Hz - ${this.actualMaxFreq.toFixed(1)}Hz`,
                peak: `${this.peakFrequency.toFixed(1)}Hz @ ${this.peakTime.toFixed(2)}s`,
                maxMagnitude: this.maxMagnitude.toFixed(3),
            });
        },

        /**
         * Reset analysis values
         */
        resetAnalysis() {
            this.timeFrames = 0;
            this.frequencyBins = 0;
            this.totalDuration = 0;
            this.timeResolution = 0;
            this.frequencyResolution = 0;
            this.actualMinFreq = this.minFrequency;
            this.actualMaxFreq = this.maxFrequency;
            this.peakFrequency = 0;
            this.peakTime = 0;
            this.peakMagnitude = 0;
            this.maxMagnitude = 0;
            this.processedData = null;
        },

        /**
         * Update frequency range and re-render
         */
        updateFrequencyRange() {
            this.analyzeSpectrogramData();
            this.renderSpectrogram();
        },

        /**
         * Preprocess spectrogram data for efficient rendering
         */
        preprocessSpectrogramData() {
            if (!this.hasValidData || this.maxMagnitude === 0) return null;

            const processed = [];

            for (let timeIdx = 0; timeIdx < this.timeFrames; timeIdx++) {
                const frame = this.spectrogram[timeIdx];
                const processedFrame = [];

                for (let freqIdx = 0; freqIdx < this.frequencyBins; freqIdx++) {
                    let magnitude =
                        Math.abs(frame[freqIdx]) / this.maxMagnitude;

                    // Apply contrast enhancement if enabled
                    if (this.enhanceContrast) {
                        // Gamma correction for better visual contrast
                        magnitude = Math.pow(magnitude, 0.3);
                    }

                    processedFrame.push(Math.max(0, Math.min(1, magnitude)));
                }

                processed.push(processedFrame);
            }

            return processed;
        },

        /**
         * Main spectrogram rendering function
         */
        renderSpectrogram() {
            if (!this.ctx || !this.hasValidData || !this.processedData) {
                this.renderEmptyState();
                return;
            }

            console.log(
                `🎨 Rendering spectrogram: ${this.timeFrames} × ${this.frequencyBins}`,
            );

            const canvas = this.$refs.canvas;
            const width = canvas.clientWidth;
            const height = canvas.clientHeight;

            // Clear canvas
            this.ctx.clearRect(0, 0, width, height);

            // Set up drawing area with margins for axes
            const margin = { top: 20, right: 40, bottom: 60, left: 80 };
            const drawWidth = width - margin.left - margin.right;
            const drawHeight = height - margin.top - margin.bottom;

            // Render spectrogram data
            this.drawSpectrogramData(
                margin.left,
                margin.top,
                drawWidth,
                drawHeight,
            );

            // Draw axes and labels
            this.drawAxesAndLabels(
                margin.left,
                margin.top,
                drawWidth,
                drawHeight,
            );

            console.log("✅ Spectrogram rendered successfully");
        },

        /**
         * Draw the spectrogram data as colored pixels
         */
        drawSpectrogramData(x, y, width, height) {
            // Create ImageData for efficient pixel manipulation
            const imageData = this.ctx.createImageData(width, height);
            const data = imageData.data;

            // Calculate frequency bin range to display
            const minBinIdx = Math.floor(
                this.actualMinFreq / this.frequencyResolution,
            );
            const maxBinIdx = Math.min(
                Math.ceil(this.actualMaxFreq / this.frequencyResolution),
                this.frequencyBins - 1,
            );

            const displayBins = maxBinIdx - minBinIdx + 1;

            // Render each pixel
            for (let canvasY = 0; canvasY < height; canvasY++) {
                for (let canvasX = 0; canvasX < width; canvasX++) {
                    // Map canvas coordinates to data coordinates
                    const timeIdx = Math.floor(
                        (canvasX / width) * this.timeFrames,
                    );

                    let freqBinIdx;
                    if (this.isLogFreqScale) {
                        freqBinIdx = this.logFreqToIdx(
                            canvasY,
                            height,
                            minBinIdx,
                            maxBinIdx,
                        );
                    } else {
                        const freqRatio = 1 - canvasY / height; // Flip Y axis
                        freqBinIdx = Math.floor(
                            minBinIdx + freqRatio * displayBins,
                        );
                    }

                    // Ensure indices are within bounds
                    if (
                        timeIdx >= 0 &&
                        timeIdx < this.timeFrames &&
                        freqBinIdx >= minBinIdx &&
                        freqBinIdx <= maxBinIdx
                    ) {
                        const magnitude =
                            this.processedData[timeIdx][freqBinIdx];
                        const color = this.getColorFromMagnitude(magnitude);

                        // Set pixel color in ImageData
                        const pixelIdx = (canvasY * width + canvasX) * 4;
                        data[pixelIdx] = color.r; // Red
                        data[pixelIdx + 1] = color.g; // Green
                        data[pixelIdx + 2] = color.b; // Blue
                        data[pixelIdx + 3] = 255; // Alpha
                    }
                }
            }

            // Draw the ImageData to canvas
            this.ctx.putImageData(imageData, x, y);
        },

        /**
         * Convert logarithmic frequency scale to bin index
         */
        logFreqToIdx(canvasY, height, minBinIdx, maxBinIdx) {
            const minFreq = minBinIdx * this.frequencyResolution;
            const maxFreq = maxBinIdx * this.frequencyResolution;

            if (minFreq <= 0) return minBinIdx;

            const logMin = Math.log10(minFreq);
            const logMax = Math.log10(maxFreq);
            const freqRatio = 1 - canvasY / height; // Flip Y axis
            const logFreq = logMin + freqRatio * (logMax - logMin);
            const freq = Math.pow(10, logFreq);

            return Math.floor(freq / this.frequencyResolution);
        },

        /**
         * Get color from magnitude using selected color map
         */
        getColorFromMagnitude(magnitude) {
            const clampedMag = Math.max(0, Math.min(1, magnitude));

            switch (this.colorMapType) {
                case "heat":
                    return this.heatColorMap(clampedMag);
                case "plasma":
                    return this.plasmaColorMap(clampedMag);
                case "viridis":
                    return this.viridisColorMap(clampedMag);
                case "grayscale":
                    return this.grayscaleColorMap(clampedMag);
                default:
                    return this.heatColorMap(clampedMag);
            }
        },

        /**
         * Heat color map (black -> red -> yellow -> white)
         */
        heatColorMap(t) {
            if (t < 0.33) {
                const v = t * 3;
                return { r: Math.floor(v * 255), g: 0, b: 0 };
            } else if (t < 0.66) {
                const v = (t - 0.33) * 3;
                return { r: 255, g: Math.floor(v * 255), b: 0 };
            } else {
                const v = (t - 0.66) * 3;
                return { r: 255, g: 255, b: Math.floor(v * 255) };
            }
        },

        /**
         * Plasma color map (purple -> pink -> yellow)
         */
        plasmaColorMap(t) {
            const r = Math.floor(
                255 * Math.max(0, Math.min(1, 0.13 + 0.87 * t)),
            );
            const g = Math.floor(
                255 * Math.max(0, Math.min(1, -0.05 + 0.8 * t)),
            );
            const b = Math.floor(
                255 * Math.max(0, Math.min(1, 0.54 - 0.6 * t + 0.2 * t * t)),
            );
            return { r, g, b };
        },

        /**
         * Viridis color map (purple -> blue -> green -> yellow)
         */
        viridisColorMap(t) {
            const r = Math.floor(
                255 * Math.max(0, Math.min(1, 0.27 + 0.9 * t - 0.4 * t * t)),
            );
            const g = Math.floor(
                255 * Math.max(0, Math.min(1, 0.0 + 0.95 * t)),
            );
            const b = Math.floor(
                255 * Math.max(0, Math.min(1, 0.33 + 0.6 * t - 0.8 * t * t)),
            );
            return { r, g, b };
        },

        /**
         * Grayscale color map
         */
        grayscaleColorMap(t) {
            const value = Math.floor(t * 255);
            return { r: value, g: value, b: value };
        },

        /**
         * Get color map colors for legend gradient
         */
        getColorMapColors() {
            const samples = [0, 0.25, 0.5, 0.75, 1.0];
            return samples.map((t) => {
                const { r, g, b } = this.getColorFromMagnitude(t);
                return `rgb(${r}, ${g}, ${b})`;
            });
        },

        /**
         * Draw axes and labels
         */
        drawAxesAndLabels(x, y, width, height) {
            const textColor =
                getComputedStyle(document.documentElement)
                    .getPropertyValue("--text-secondary")
                    .trim() || "#666666";

            this.ctx.fillStyle = textColor;
            this.ctx.font =
                "11px -apple-system, BlinkMacSystemFont, sans-serif";

            // Time axis (X-axis)
            this.ctx.textAlign = "center";
            this.ctx.textBaseline = "top";

            const timeSteps = 6;
            for (let i = 0; i <= timeSteps; i++) {
                const time = (i * this.totalDuration) / timeSteps;
                const labelX = x + (i * width) / timeSteps;
                this.ctx.fillText(
                    this.formatTime(time),
                    labelX,
                    y + height + 5,
                );
            }

            // Frequency axis (Y-axis)
            this.ctx.textAlign = "right";
            this.ctx.textBaseline = "middle";

            const freqSteps = 8;
            for (let i = 0; i <= freqSteps; i++) {
                let freq;
                if (this.isLogFreqScale) {
                    const logMin = Math.log10(this.actualMinFreq);
                    const logMax = Math.log10(this.actualMaxFreq);
                    const logFreq =
                        logMax - (i * (logMax - logMin)) / freqSteps;
                    freq = Math.pow(10, logFreq);
                } else {
                    freq =
                        this.actualMaxFreq -
                        (i * (this.actualMaxFreq - this.actualMinFreq)) /
                            freqSteps;
                }

                const labelY = y + (i * height) / freqSteps;
                this.ctx.fillText(this.formatFrequency(freq), x - 10, labelY);
            }

            // Axis titles
            this.ctx.font =
                "12px -apple-system, BlinkMacSystemFont, sans-serif";

            // X-axis title
            this.ctx.textAlign = "center";
            this.ctx.textBaseline = "top";
            this.ctx.fillText("Time (s)", x + width / 2, y + height + 35);

            // Y-axis title
            this.ctx.save();
            this.ctx.translate(x - 50, y + height / 2);
            this.ctx.rotate(-Math.PI / 2);
            this.ctx.textAlign = "center";
            this.ctx.textBaseline = "middle";
            this.ctx.fillText("Frequency (Hz)", 0, 0);
            this.ctx.restore();
        },

        /**
         * Render empty state when no valid data
         */
        renderEmptyState() {
            if (!this.ctx) return;

            const canvas = this.$refs.canvas;
            const width = canvas.clientWidth;
            const height = canvas.clientHeight;

            // Clear canvas
            this.ctx.clearRect(0, 0, width, height);

            // Draw empty state message
            const textColor =
                getComputedStyle(document.documentElement)
                    .getPropertyValue("--text-tertiary")
                    .trim() || "#888888";

            this.ctx.fillStyle = textColor;
            this.ctx.font =
                "14px -apple-system, BlinkMacSystemFont, sans-serif";
            this.ctx.textAlign = "center";
            this.ctx.textBaseline = "middle";

            const message =
                this.spectrogram.length === 0
                    ? "No spectrogram data available"
                    : "Invalid spectrogram data format";

            this.ctx.fillText(message, width / 2, height / 2);

            console.log("📝 Empty state rendered:", message);
        },

        /**
         * Toggle frequency scale between linear and logarithmic
         */
        toggleFreqScale() {
            this.isLogFreqScale = !this.isLogFreqScale;
        },

        /**
         * Toggle contrast enhancement
         */
        toggleIntensity() {
            this.enhanceContrast = !this.enhanceContrast;
            // Need to reprocess data when contrast changes
            this.processedData = this.preprocessSpectrogramData();
        },

        /**
         * Format time for display
         */
        formatTime(seconds) {
            if (seconds < 1) {
                return `${(seconds * 1000).toFixed(0)}ms`;
            } else if (seconds < 60) {
                return `${seconds.toFixed(1)}s`;
            } else {
                const minutes = Math.floor(seconds / 60);
                const remainingSeconds = seconds % 60;
                return `${minutes}:${remainingSeconds.toFixed(1).padStart(4, "0")}`;
            }
        },

        /**
         * Format frequency for display
         */
        formatFrequency(freq) {
            if (freq < 1000) {
                return `${Math.round(freq)}`;
            } else if (freq < 10000) {
                return `${(freq / 1000).toFixed(1)}k`;
            } else {
                return `${Math.round(freq / 1000)}k`;
            }
        },

        /**
         * Cleanup resources
         */
        cleanup() {
            if (this.resizeObserver) {
                this.resizeObserver.disconnect();
                this.resizeObserver = null;
            }

            if (this.animationFrameId) {
                cancelAnimationFrame(this.animationFrameId);
                this.animationFrameId = null;
            }
        },
    },
};
</script>
