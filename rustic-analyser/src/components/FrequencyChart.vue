<template>
    <div class="frequency-chart-container">
        <!-- Canvas for frequency spectrum display -->
        <canvas
            ref="canvas"
            width="800"
            height="300"
            class="frequency-canvas"
            :title="`Frequency Spectrum: ${frequencies.length} data points`"
        ></canvas>

        <!-- Chart controls -->
        <div class="chart-controls">
            <div class="control-group">
                <button
                    @click="toggleScale"
                    class="scale-toggle"
                    :class="{ active: isLogScale }"
                >
                    {{ isLogScale ? "Log" : "Linear" }} Scale
                </button>

                <button
                    @click="toggleSmoothing"
                    class="smoothing-toggle"
                    :class="{ active: enableSmoothing }"
                >
                    {{ enableSmoothing ? "Smooth" : "Raw" }}
                </button>
            </div>
        </div>

        <!-- Frequency info display -->
        <div class="frequency-info">
            <div class="info-row">
                <span class="info-label">Range:</span>
                <span class="info-value"
                    >{{ formatFrequency(minFreq) }} -
                    {{ formatFrequency(maxFreq) }}</span
                >

                <span class="info-label">Peak:</span>
                <span class="info-value"
                    >{{ formatFrequency(peakFrequency) }} ({{
                        peakMagnitude.toFixed(3)
                    }})</span
                >

                <span class="info-label">Points:</span>
                <span class="info-value">{{ frequencies.length }}</span>
            </div>
        </div>

        <!-- Chart legend -->
        <div class="chart-legend">
            <div class="legend-item">
                <span class="legend-color frequency-magnitude"></span>
                <span class="legend-label">Frequency Magnitude</span>
            </div>
            <div v-if="enableSmoothing" class="legend-item">
                <span class="legend-color smoothed-line"></span>
                <span class="legend-label">Smoothed Curve</span>
            </div>
        </div>
    </div>
</template>

<script>
/**
 * FrequencyChart Component
 *
 * Renders frequency spectrum analysis with adaptive scaling.
 * Features:
 * - Dynamic frequency range based on actual data
 * - Linear and logarithmic scale options
 * - Smoothing option for cleaner visualization
 * - Theme-aware rendering
 * - Performance optimized for large datasets
 * - Interactive controls and detailed info display
 */
export default {
    name: "FrequencyChart",

    props: {
        /**
         * Array of frequency data as [frequency, magnitude] pairs
         * Expected format: [[freq1, mag1], [freq2, mag2], ...]
         */
        frequencies: {
            type: Array,
            required: true,
            default: () => [],
        },

        /**
         * Sample rate of the audio (in Hz)
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
            // Scale mode (linear or logarithmic)
            isLogScale: false,
            // Smoothing option
            enableSmoothing: true,
            // Calculated frequency range
            minFreq: 0,
            maxFreq: 20000,
            // Peak detection
            peakFrequency: 0,
            peakMagnitude: 0,
            // Resize observer for responsive canvas
            resizeObserver: null,
            // Animation frame ID for cleanup
            animationFrameId: null,
        };
    },

    computed: {
        /**
         * Check if we have valid frequency data
         */
        hasValidData() {
            return (
                Array.isArray(this.frequencies) &&
                this.frequencies.length > 0 &&
                this.frequencies.every(
                    (point) =>
                        Array.isArray(point) &&
                        point.length >= 2 &&
                        typeof point[0] === "number" &&
                        typeof point[1] === "number",
                )
            );
        },

        /**
         * Processed frequency data for rendering
         */
        processedData() {
            if (!this.hasValidData) return [];

            // Sort by frequency to ensure proper order
            const sortedData = [...this.frequencies].sort(
                (a, b) => a[0] - b[0],
            );

            // Filter out invalid frequencies and normalize magnitudes
            const validData = sortedData.filter(
                ([freq, mag]) =>
                    freq >= 0 &&
                    freq <= this.sampleRate / 2 &&
                    !isNaN(freq) &&
                    !isNaN(mag) &&
                    mag >= 0,
            );

            // Normalize magnitudes to 0-1 range
            const maxMagnitude = Math.max(...validData.map(([, mag]) => mag));
            if (maxMagnitude === 0) return validData;

            return validData.map(([freq, mag]) => [freq, mag / maxMagnitude]);
        },
    },

    mounted() {
        console.log("📊 FrequencyChart mounted");
        this.initializeCanvas();
        this.setupResizeObserver();
        this.analyzeFrequencyData();
        this.renderChart();
    },

    beforeUnmount() {
        console.log("📊 FrequencyChart cleanup");
        this.cleanup();
    },

    watch: {
        // Re-render when frequency data changes
        frequencies: {
            handler() {
                console.log(
                    `📊 Frequencies updated: ${this.frequencies.length} data points`,
                );
                this.analyzeFrequencyData();
                this.renderChart();
            },
            immediate: false,
        },

        // Re-render when sample rate changes
        sampleRate() {
            console.log(`📊 Sample rate updated: ${this.sampleRate}Hz`);
            this.analyzeFrequencyData();
            this.renderChart();
        },

        // Re-render when scale mode changes
        isLogScale() {
            console.log(
                `📊 Scale mode: ${this.isLogScale ? "logarithmic" : "linear"}`,
            );
            this.renderChart();
        },

        // Re-render when smoothing changes
        enableSmoothing() {
            console.log(
                `📊 Smoothing: ${this.enableSmoothing ? "enabled" : "disabled"}`,
            );
            this.renderChart();
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
                    this.renderChart();
                });
            });

            this.resizeObserver.observe(this.$refs.canvas);
        },

        /**
         * Analyze frequency data to determine range and peak
         */
        analyzeFrequencyData() {
            if (!this.hasValidData) {
                this.resetAnalysis();
                return;
            }

            // Calculate actual frequency range from data
            const frequencies = this.processedData.map(([freq]) => freq);
            this.minFreq = Math.max(Math.min(...frequencies), 20); // Start from 20Hz minimum
            this.maxFreq = Math.min(
                Math.max(...frequencies),
                this.sampleRate / 2,
            );

            // Find peak frequency and magnitude
            let maxMag = 0;
            let peakFreq = 0;

            this.processedData.forEach(([freq, mag]) => {
                if (mag > maxMag) {
                    maxMag = mag;
                    peakFreq = freq;
                }
            });

            this.peakFrequency = peakFreq;
            this.peakMagnitude = maxMag;

            console.log("📊 Frequency analysis:", {
                range: `${this.minFreq.toFixed(1)}Hz - ${this.maxFreq.toFixed(1)}Hz`,
                peak: `${this.peakFrequency.toFixed(1)}Hz (${this.peakMagnitude.toFixed(3)})`,
                points: this.processedData.length,
            });
        },

        /**
         * Reset analysis values
         */
        resetAnalysis() {
            this.minFreq = 0;
            this.maxFreq = this.sampleRate / 2;
            this.peakFrequency = 0;
            this.peakMagnitude = 0;
        },

        /**
         * Main chart rendering function
         */
        renderChart() {
            if (!this.ctx || !this.hasValidData) {
                this.renderEmptyState();
                return;
            }

            console.log(
                `🎨 Rendering frequency chart: ${this.processedData.length} points`,
            );

            const canvas = this.$refs.canvas;
            const width = canvas.clientWidth;
            const height = canvas.clientHeight;

            // Clear canvas
            this.ctx.clearRect(0, 0, width, height);

            // Set up drawing area with margins
            const margin = { top: 20, right: 20, bottom: 60, left: 60 };
            const drawWidth = width - margin.left - margin.right;
            const drawHeight = height - margin.top - margin.bottom;

            // Draw background and grid
            this.drawBackground(margin.left, margin.top, drawWidth, drawHeight);
            this.drawGrid(margin.left, margin.top, drawWidth, drawHeight);

            // Draw frequency spectrum
            this.drawSpectrum(margin.left, margin.top, drawWidth, drawHeight);

            // Draw axes and labels
            this.drawAxes(margin.left, margin.top, drawWidth, drawHeight);
            this.drawLabels(margin.left, margin.top, drawWidth, drawHeight);

            console.log("✅ Frequency chart rendered successfully");
        },

        /**
         * Draw background
         */
        drawBackground(x, y, width, height) {
            // Get theme-aware background color
            const bgColor =
                getComputedStyle(document.documentElement)
                    .getPropertyValue("--canvas-bg")
                    .trim() || "#f5f5f5";

            this.ctx.fillStyle = bgColor;
            this.ctx.fillRect(x, y, width, height);
        },

        /**
         * Draw grid lines
         */
        drawGrid(x, y, width, height) {
            const gridColor =
                getComputedStyle(document.documentElement)
                    .getPropertyValue("--grid-color")
                    .trim() || "#dddddd";

            this.ctx.strokeStyle = gridColor;
            this.ctx.lineWidth = 0.5;
            this.ctx.setLineDash([2, 2]);

            // Horizontal grid lines (magnitude)
            const magSteps = 5;
            for (let i = 0; i <= magSteps; i++) {
                const gridY = y + (i * height) / magSteps;
                this.ctx.beginPath();
                this.ctx.moveTo(x, gridY);
                this.ctx.lineTo(x + width, gridY);
                this.ctx.stroke();
            }

            // Vertical grid lines (frequency)
            const freqSteps = this.isLogScale ? this.getLogFreqSteps() : 10;
            if (this.isLogScale) {
                // Logarithmic frequency grid
                freqSteps.forEach((freq) => {
                    if (freq >= this.minFreq && freq <= this.maxFreq) {
                        const gridX = x + this.freqToX(freq, width);
                        this.ctx.beginPath();
                        this.ctx.moveTo(gridX, y);
                        this.ctx.lineTo(gridX, y + height);
                        this.ctx.stroke();
                    }
                });
            } else {
                // Linear frequency grid
                for (let i = 0; i <= freqSteps; i++) {
                    const gridX = x + (i * width) / freqSteps;
                    this.ctx.beginPath();
                    this.ctx.moveTo(gridX, y);
                    this.ctx.lineTo(gridX, y + height);
                    this.ctx.stroke();
                }
            }

            this.ctx.setLineDash([]);
        },

        /**
         * Draw frequency spectrum
         */
        drawSpectrum(x, y, width, height) {
            if (!this.processedData.length) return;

            // Get theme-aware color
            const primaryColor =
                getComputedStyle(document.documentElement)
                    .getPropertyValue("--accent-primary")
                    .trim() || "#2196F3";

            if (this.enableSmoothing) {
                this.drawSmoothSpectrum(x, y, width, height, primaryColor);
            } else {
                this.drawBarSpectrum(x, y, width, height, primaryColor);
            }
        },

        /**
         * Draw smooth curve spectrum
         */
        drawSmoothSpectrum(x, y, width, height, color) {
            this.ctx.strokeStyle = color;
            this.ctx.lineWidth = 2;
            this.ctx.lineCap = "round";
            this.ctx.lineJoin = "round";

            this.ctx.beginPath();

            this.processedData.forEach(([freq, mag], index) => {
                const plotX = x + this.freqToX(freq, width);
                const plotY = y + height - mag * height;

                if (index === 0) {
                    this.ctx.moveTo(plotX, plotY);
                } else {
                    this.ctx.lineTo(plotX, plotY);
                }
            });

            this.ctx.stroke();

            // Add fill area
            this.ctx.globalAlpha = 0.1;
            this.ctx.fillStyle = color;

            // Close path for fill
            const lastPoint = this.processedData[this.processedData.length - 1];
            if (lastPoint) {
                this.ctx.lineTo(
                    x + this.freqToX(lastPoint[0], width),
                    y + height,
                );
                this.ctx.lineTo(
                    x + this.freqToX(this.processedData[0][0], width),
                    y + height,
                );
                this.ctx.closePath();
                this.ctx.fill();
            }

            this.ctx.globalAlpha = 1.0;
        },

        /**
         * Draw bar spectrum
         */
        drawBarSpectrum(x, y, width, height, color) {
            this.ctx.fillStyle = color;

            const barWidth = Math.max(1, width / this.processedData.length);

            this.processedData.forEach(([freq, mag]) => {
                const barX = x + this.freqToX(freq, width) - barWidth / 2;
                const barHeight = mag * height;
                const barY = y + height - barHeight;

                this.ctx.fillRect(barX, barY, barWidth, barHeight);
            });
        },

        /**
         * Convert frequency to X coordinate
         */
        freqToX(freq, width) {
            if (this.isLogScale) {
                if (freq <= 0) return 0;
                const logMin = Math.log10(this.minFreq);
                const logMax = Math.log10(this.maxFreq);
                const logFreq = Math.log10(freq);
                return ((logFreq - logMin) / (logMax - logMin)) * width;
            } else {
                return (
                    ((freq - this.minFreq) / (this.maxFreq - this.minFreq)) *
                    width
                );
            }
        },

        /**
         * Get logarithmic frequency steps for grid
         */
        getLogFreqSteps() {
            const steps = [];
            const decades = [1, 2, 5];

            for (let power = 1; power <= 5; power++) {
                decades.forEach((base) => {
                    const freq = base * Math.pow(10, power);
                    if (freq >= 10 && freq <= 100000) {
                        steps.push(freq);
                    }
                });
            }

            return steps;
        },

        /**
         * Draw axes
         */
        drawAxes(x, y, width, height) {
            const axisColor =
                getComputedStyle(document.documentElement)
                    .getPropertyValue("--text-primary")
                    .trim() || "#333333";

            this.ctx.strokeStyle = axisColor;
            this.ctx.lineWidth = 1;

            // X-axis
            this.ctx.beginPath();
            this.ctx.moveTo(x, y + height);
            this.ctx.lineTo(x + width, y + height);
            this.ctx.stroke();

            // Y-axis
            this.ctx.beginPath();
            this.ctx.moveTo(x, y);
            this.ctx.lineTo(x, y + height);
            this.ctx.stroke();
        },

        /**
         * Draw axis labels
         */
        drawLabels(x, y, width, height) {
            const textColor =
                getComputedStyle(document.documentElement)
                    .getPropertyValue("--text-secondary")
                    .trim() || "#666666";

            this.ctx.fillStyle = textColor;
            this.ctx.font =
                "11px -apple-system, BlinkMacSystemFont, sans-serif";

            // Frequency labels (X-axis)
            this.ctx.textAlign = "center";
            this.ctx.textBaseline = "top";

            if (this.isLogScale) {
                const freqSteps = this.getLogFreqSteps();
                freqSteps.forEach((freq) => {
                    if (freq >= this.minFreq && freq <= this.maxFreq) {
                        const labelX = x + this.freqToX(freq, width);
                        const label = this.formatFrequency(freq);
                        this.ctx.fillText(label, labelX, y + height + 5);
                    }
                });
            } else {
                const freqSteps = 8;
                for (let i = 0; i <= freqSteps; i++) {
                    const freq =
                        this.minFreq +
                        (i * (this.maxFreq - this.minFreq)) / freqSteps;
                    const labelX = x + (i * width) / freqSteps;
                    const label = this.formatFrequency(freq);
                    this.ctx.fillText(label, labelX, y + height + 5);
                }
            }

            // Magnitude labels (Y-axis)
            this.ctx.textAlign = "right";
            this.ctx.textBaseline = "middle";

            const magSteps = 5;
            for (let i = 0; i <= magSteps; i++) {
                const magnitude = i / magSteps;
                const labelY = y + height - (i * height) / magSteps;
                const label = magnitude.toFixed(1);
                this.ctx.fillText(label, x - 10, labelY);
            }

            // Axis titles
            this.ctx.textAlign = "center";
            this.ctx.textBaseline = "top";
            this.ctx.font =
                "12px -apple-system, BlinkMacSystemFont, sans-serif";

            // X-axis title
            this.ctx.fillText("Frequency (Hz)", x + width / 2, y + height + 35);

            // Y-axis title
            this.ctx.save();
            this.ctx.translate(x - 40, y + height / 2);
            this.ctx.rotate(-Math.PI / 2);
            this.ctx.textAlign = "center";
            this.ctx.textBaseline = "middle";
            this.ctx.fillText("Magnitude", 0, 0);
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
                this.frequencies.length === 0
                    ? "No frequency data available"
                    : "Invalid frequency data format";

            this.ctx.fillText(message, width / 2, height / 2);

            console.log("📝 Empty state rendered:", message);
        },

        /**
         * Toggle between linear and logarithmic scale
         */
        toggleScale() {
            this.isLogScale = !this.isLogScale;
        },

        /**
         * Toggle smoothing on/off
         */
        toggleSmoothing() {
            this.enableSmoothing = !this.enableSmoothing;
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
