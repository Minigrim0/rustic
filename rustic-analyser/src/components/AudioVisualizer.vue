<template>
    <div class="visualizer-container">
        <!-- Canvas for waveform display -->
        <canvas ref="canvas" width="800" height="200" class="waveform-canvas"
            :title="`Waveform: ${samples.length} samples at ${sampleRate}Hz`"></canvas>

        <!-- Waveform info display -->
        <div class="waveform-info">
            <div class="info-row">
                <span class="info-label">Samples:</span>
                <span class="info-value">{{
                    formatNumber(samples.length)
                    }}</span>
            </div>

            <div class="info-row">

                <span class="info-label">Duration:</span>
                <span class="info-value">{{ formatDuration(duration) }}</span>
            </div>

            <div class="info-row">
                <span class="info-label">Peak:</span>
                <span class="info-value">{{
                    formatAmplitude(peakAmplitude)
                    }}</span>
            </div>
        </div>
    </div>
</template>

<script lang="ts">
/**
 * AudioVisualizer Component
 *
 * Renders audio waveform visualization from sample data.
 * Features:
 * - Adaptive sampling for performance with large datasets
 * - Theme-aware rendering with CSS custom properties
 * - Real-time peak amplitude detection
 * - Responsive canvas sizing
 * - Error handling for invalid data
 */
export default {
    name: "AudioVisualizer",

    props: {
        /**
         * Array of audio samples (floating point values, typically -1.0 to 1.0)
         */
        samples: {
            type: Array,
            required: true,
            default: () => [],
        },

        /**
         * Sample rate of the audio in Hz
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
            // Peak amplitude for display
            peakAmplitude: 0,
            // Resize observer for responsive canvas
            resizeObserver: null,
            // Animation frame ID for cleanup
            animationFrameId: null,
        };
    },

    computed: {
        /**
         * Calculate audio duration in seconds
         */
        duration() {
            if (!this.samples.length || !this.sampleRate) return 0;
            return this.samples.length / this.sampleRate;
        },

        /**
         * Check if we have valid data to render
         */
        hasValidData() {
            return (
                Array.isArray(this.samples) &&
                this.samples.length > 0 &&
                this.sampleRate > 0
            );
        },
    },

    mounted() {
        console.log("🎵 AudioVisualizer mounted");
        this.initializeCanvas();
        this.setupResizeObserver();
        this.renderWaveform();
    },

    beforeUnmount() {
        console.log("🎵 AudioVisualizer cleanup");
        this.cleanup();
    },

    watch: {
        // Re-render when samples change
        samples: {
            handler() {
                console.log(
                    `🎵 Samples updated: ${this.samples.length} samples`,
                );
                this.renderWaveform();
            },
            immediate: false,
        },

        // Re-render when sample rate changes
        sampleRate() {
            console.log(`🎵 Sample rate updated: ${this.sampleRate}Hz`);
            this.renderWaveform();
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
                // Debounce resize events
                if (this.animationFrameId) {
                    cancelAnimationFrame(this.animationFrameId);
                }

                this.animationFrameId = requestAnimationFrame(() => {
                    this.initializeCanvas();
                    this.renderWaveform();
                });
            });

            this.resizeObserver.observe(this.$refs.canvas);
        },

        /**
         * Main waveform rendering function
         */
        renderWaveform() {
            if (!this.ctx || !this.hasValidData) {
                this.renderEmptyState();
                return;
            }

            console.log(
                `🎨 Rendering waveform: ${this.samples.length} samples`,
            );

            const canvas = this.$refs.canvas;
            const width = canvas.clientWidth;
            const height = canvas.clientHeight;

            // Clear canvas
            this.ctx.clearRect(0, 0, width, height);

            // Calculate peak amplitude for info display
            this.calculatePeakAmplitude();

            // Prepare sample data for rendering
            const normalizedSamples = this.prepareSampleData();

            // Draw waveform
            this.drawWaveform(normalizedSamples, width, height);

            // Draw center line and scale indicators
            this.drawWaveformGuides(width, height);

            console.log("✅ Waveform rendered successfully");
        },

        /**
         * Calculate peak amplitude from samples
         */
        calculatePeakAmplitude() {
            this.peakAmplitude = Math.max(...this.samples.map(Math.abs));
        },

        /**
         * Prepare sample data for efficient rendering
         * Uses adaptive sampling for large datasets
         */
        prepareSampleData() {
            const canvas = this.$refs.canvas;
            const canvasWidth = canvas.clientWidth;

            // If we have fewer samples than canvas width, use all samples
            if (this.samples.length <= canvasWidth) {
                return this.normalizeSamples(this.samples);
            }

            // For large datasets, use RMS downsampling for better visual representation
            const samplesPerPixel = Math.ceil(
                this.samples.length / canvasWidth,
            );
            const downsampledData = [];

            for (let i = 0; i < canvasWidth; i++) {
                const startIdx = i * samplesPerPixel;
                const endIdx = Math.min(
                    startIdx + samplesPerPixel,
                    this.samples.length,
                );

                // Calculate RMS value for this pixel
                let sum = 0;
                for (let j = startIdx; j < endIdx; j++) {
                    sum += this.samples[j] * this.samples[j];
                }

                const rms = Math.sqrt(sum / (endIdx - startIdx));
                // Preserve sign by using the average of the segment
                const avg =
                    this.samples
                        .slice(startIdx, endIdx)
                        .reduce((a, b) => a + b, 0) /
                    (endIdx - startIdx);
                downsampledData.push(avg >= 0 ? rms : -rms);
            }

            return this.normalizeSamples(downsampledData);
        },

        /**
         * Normalize samples to -1 to 1 range
         */
        normalizeSamples(samples) {
            if (!samples.length) return [];

            const maxAmplitude = Math.max(...samples.map(Math.abs));
            if (maxAmplitude === 0) return samples;

            return samples.map((sample) => sample / maxAmplitude);
        },

        /**
         * Draw the actual waveform
         */
        drawWaveform(samples, width, height) {
            if (!samples.length) return;

            const midY = height / 2;
            const amplitude = (height / 2) * 0.9; // Leave 10% margin

            // Get theme-aware colors from CSS custom properties
            const primaryColor =
                getComputedStyle(document.documentElement)
                    .getPropertyValue("--accent-primary")
                    .trim() || "#2196F3";

            // Draw waveform path
            this.ctx.strokeStyle = primaryColor;
            this.ctx.lineWidth = 1.5;
            this.ctx.lineCap = "round";
            this.ctx.lineJoin = "round";

            this.ctx.beginPath();

            for (let i = 0; i < samples.length; i++) {
                const x = (i / (samples.length - 1)) * width;
                const y = midY - samples[i] * amplitude;

                if (i === 0) {
                    this.ctx.moveTo(x, y);
                } else {
                    this.ctx.lineTo(x, y);
                }
            }

            this.ctx.stroke();

            // Add subtle fill for better visual appeal
            this.ctx.globalAlpha = 0.1;
            this.ctx.fillStyle = primaryColor;

            // Close the path to baseline for fill
            this.ctx.lineTo(width, midY);
            this.ctx.lineTo(0, midY);
            this.ctx.closePath();
            this.ctx.fill();

            this.ctx.globalAlpha = 1.0;
        },

        /**
         * Draw waveform guides (center line, scale)
         */
        drawWaveformGuides(width, height) {
            const midY = height / 2;

            // Get theme-aware color
            const borderColor =
                getComputedStyle(document.documentElement)
                    .getPropertyValue("--border-medium")
                    .trim() || "#dddddd";

            this.ctx.strokeStyle = borderColor;
            this.ctx.lineWidth = 1;
            this.ctx.setLineDash([5, 5]);

            // Center line (zero amplitude)
            this.ctx.beginPath();
            this.ctx.moveTo(0, midY);
            this.ctx.lineTo(width, midY);
            this.ctx.stroke();

            // Reset line dash
            this.ctx.setLineDash([]);
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

            // Get theme-aware colors
            const textColor =
                getComputedStyle(document.documentElement)
                    .getPropertyValue("--text-tertiary")
                    .trim() || "#888888";

            // Draw empty state message
            this.ctx.fillStyle = textColor;
            this.ctx.font =
                "14px -apple-system, BlinkMacSystemFont, sans-serif";
            this.ctx.textAlign = "center";
            this.ctx.textBaseline = "middle";

            const message =
                this.samples.length === 0
                    ? "No audio data available"
                    : "Invalid audio data";

            this.ctx.fillText(message, width / 2, height / 2);

            console.log("📝 Empty state rendered:", message);
        },

        /**
         * Format large numbers with commas
         */
        formatNumber(num) {
            return num.toLocaleString();
        },

        /**
         * Format duration in seconds to MM:SS format
         */
        formatDuration(seconds) {
            if (!seconds || seconds < 0) return "0:00";

            const minutes = Math.floor(seconds / 60);
            const remainingSeconds = seconds % 60;

            return `${minutes}:${remainingSeconds.toFixed(2).padStart(5, "0")}`;
        },

        /**
         * Format amplitude with appropriate precision
         */
        formatAmplitude(amplitude) {
            if (amplitude === 0) return "0.00";
            if (amplitude < 0.01) return amplitude.toExponential(2);
            return amplitude.toFixed(3);
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
