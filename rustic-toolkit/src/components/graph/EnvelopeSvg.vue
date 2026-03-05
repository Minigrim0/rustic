<template>
    <div class="envelope-svg-wrapper" @mousemove="onMouseMove" @mouseup="onMouseUp" @mouseleave="onMouseUp">
        <svg
            ref="svgEl"
            :width="W + PAD * 2"
            :height="H + PAD * 2 + LABEL_H"
            class="envelope-svg"
        >
            <!-- Grid lines at 0%, 25%, 50%, 75%, 100% amplitude -->
            <line
                v-for="frac in [0, 0.25, 0.5, 0.75, 1.0]"
                :key="frac"
                :x1="PAD" :y1="PAD + (1 - frac) * H"
                :x2="PAD + W" :y2="PAD + (1 - frac) * H"
                stroke="#ffffff0d" stroke-width="1"
            />

            <!-- Filled area under the curve -->
            <path :d="fillD" fill="#6366f115" stroke="none" />

            <!-- Main ADSR curve -->
            <path :d="curveD" fill="none" stroke="#6366f1" stroke-width="2" stroke-linejoin="round" />

            <!-- Dashed control arms: connect each diamond to its two adjacent endpoints -->
            <line
                v-for="arm in controlArms"
                :key="arm.key"
                :x1="arm.x1" :y1="arm.y1" :x2="arm.x2" :y2="arm.y2"
                stroke="#6366f140" stroke-width="1" stroke-dasharray="3,3"
            />

            <!-- Phase labels -->
            <text
                v-for="lbl in phaseLabels"
                :key="lbl.text"
                :x="lbl.x" :y="PAD + H + LABEL_H"
                text-anchor="middle" font-size="9" fill="#6b7280"
            >{{ lbl.text }}</text>

            <!-- Static endpoint indicator at attack peak (amplitude always = 1) -->
            <circle :cx="xPeak" :cy="yTop" r="4" fill="#4f46e5" stroke="#c7d2fe" stroke-width="1" />

            <!-- Static endpoint indicator at release start (inherits sustain level) -->
            <circle :cx="xSusEnd" :cy="ySustain" r="4" fill="#4f46e5" stroke="#c7d2fe" stroke-width="1" />

            <!-- Sustain handle (circle) — drag Y only to control sustain level -->
            <circle
                :cx="xKnee"
                :cy="ySustain"
                r="5.5"
                :fill="dragging === 'sus' ? '#818cf8' : '#6366f1'"
                stroke="#c7d2fe" stroke-width="1.5"
                class="handle"
                @mousedown.prevent="startDrag('sus', $event)"
            />

            <!-- Curve handles (diamonds) — drag X to move control point, drag Y to change curve shape -->
            <rect
                v-for="h in curveHandles"
                :key="h.id"
                :x="h.x - DIAMOND" :y="h.y - DIAMOND"
                :width="DIAMOND * 2" :height="DIAMOND * 2"
                :transform="`rotate(45, ${h.x}, ${h.y})`"
                :fill="dragging === h.id ? '#fb923c' : '#f97316'"
                stroke="#fed7aa" stroke-width="1.5"
                class="handle"
                @mousedown.prevent="startDrag(h.id, $event)"
            />

            <!-- Value readout during active drag -->
            <text
                v-if="readout"
                :x="readout.x" :y="readout.y"
                text-anchor="middle" font-size="10" fill="#e0e7ff"
            >{{ readout.text }}</text>
        </svg>
    </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";

// ─── Props & emits ────────────────────────────────────────────────────────────

const props = defineProps<{
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

const emit = defineEmits<{
    "update:sustain":       [v: number];
    "update:attackCurve":   [v: number];
    "update:decayCurve":    [v: number];
    "update:releaseCurve":  [v: number];
    "update:attackCpT":     [v: number];
    "update:decayCpT":      [v: number];
    "update:releaseCpT":    [v: number];
}>();

// ─── Layout constants ─────────────────────────────────────────────────────────

const W       = 300;   // drawable width (px)
const H       = 110;   // drawable height (amplitude axis, px)
const PAD     = 18;    // padding around canvas
const LABEL_H = 16;    // extra height below curve for phase labels
const DIAMOND = 5;     // half-size of diamond handles

// Fixed visual duration for the sustain plateau.
const SUSTAIN_DUR = 0.3;

const svgEl = ref<SVGSVGElement | null>(null);

// ─── Coordinate helpers ───────────────────────────────────────────────────────

const totalTime = computed(() => props.attack + props.decay + SUSTAIN_DUR + props.release);

function timeToX(t: number): number {
    return PAD + (t / totalTime.value) * W;
}

function ampToY(amp: number): number {
    return PAD + (1 - amp) * H;
}

/**
 * Mirrors Rust's `control_y(from, to, curve)`.
 * Maps a curve scalar [-1, 1] to a bezier control-point amplitude.
 * curve = 0 → midpoint (linear); ±1 → maximum curvature toward one endpoint.
 */
function curveToAmp(from: number, to: number, curve: number): number {
    const c = Math.max(-1, Math.min(1, curve));
    return from + ((c + 1) / 2) * (to - from);
}

/**
 * Inverse of curveToAmp: given an amplitude and the segment endpoints,
 * returns the corresponding curve scalar.  Returns 0 when from == to.
 */
function ampToCurve(from: number, to: number, amp: number): number {
    if (Math.abs(to - from) < 1e-6) return 0;
    return Math.max(-1, Math.min(1, 2 * (amp - from) / (to - from) - 1));
}

// ─── Derived SVG positions ────────────────────────────────────────────────────

const xStart  = computed(() => PAD);
const xPeak   = computed(() => timeToX(props.attack));
const xKnee   = computed(() => timeToX(props.attack + props.decay));
const xSusEnd = computed(() => timeToX(props.attack + props.decay + SUSTAIN_DUR));
const xEnd    = computed(() => PAD + W);

const yBottom  = computed(() => ampToY(0));
const yTop     = computed(() => ampToY(1.0));
const ySustain = computed(() => ampToY(props.sustain));

// Bezier control point positions.
// X = segment_start + cp_t * segment_width; Y = ampToY(curveToAmp(from, to, curve)).
const cpA = computed(() => ({
    x: xStart.value  + props.attackCpT  * (xPeak.value   - xStart.value),
    y: ampToY(curveToAmp(0.0,          1.0,          props.attackCurve)),
}));
const cpD = computed(() => ({
    x: xPeak.value   + props.decayCpT   * (xKnee.value   - xPeak.value),
    y: ampToY(curveToAmp(1.0,          props.sustain, props.decayCurve)),
}));
const cpR = computed(() => ({
    x: xSusEnd.value + props.releaseCpT * (xEnd.value    - xSusEnd.value),
    y: ampToY(curveToAmp(props.sustain, 0.0,          props.releaseCurve)),
}));

// ─── SVG paths ────────────────────────────────────────────────────────────────

const curveD = computed(() =>
    `M${xStart.value},${yBottom.value} ` +
    `Q${cpA.value.x},${cpA.value.y} ${xPeak.value},${yTop.value} ` +
    `Q${cpD.value.x},${cpD.value.y} ${xKnee.value},${ySustain.value} ` +
    `L${xSusEnd.value},${ySustain.value} ` +
    `Q${cpR.value.x},${cpR.value.y} ${xEnd.value},${yBottom.value}`
);

const fillD = computed(() =>
    `${curveD.value} ` +
    `L${xEnd.value},${PAD + H} L${xStart.value},${PAD + H} Z`
);

// ─── Handles ─────────────────────────────────────────────────────────────────

const curveHandles = computed(() => [
    { id: "cpA", x: cpA.value.x, y: cpA.value.y },
    { id: "cpD", x: cpD.value.x, y: cpD.value.y },
    { id: "cpR", x: cpR.value.x, y: cpR.value.y },
]);

const controlArms = computed(() => [
    { key: "cpA-l", x1: xStart.value,  y1: yBottom.value,  x2: cpA.value.x, y2: cpA.value.y },
    { key: "cpA-r", x1: xPeak.value,   y1: yTop.value,     x2: cpA.value.x, y2: cpA.value.y },
    { key: "cpD-l", x1: xPeak.value,   y1: yTop.value,     x2: cpD.value.x, y2: cpD.value.y },
    { key: "cpD-r", x1: xKnee.value,   y1: ySustain.value, x2: cpD.value.x, y2: cpD.value.y },
    { key: "cpR-l", x1: xSusEnd.value, y1: ySustain.value, x2: cpR.value.x, y2: cpR.value.y },
    { key: "cpR-r", x1: xEnd.value,    y1: yBottom.value,  x2: cpR.value.x, y2: cpR.value.y },
]);

const phaseLabels = computed(() => [
    { text: "A", x: (xStart.value  + xPeak.value)   / 2 },
    { text: "D", x: (xPeak.value   + xKnee.value)   / 2 },
    { text: "S", x: (xKnee.value   + xSusEnd.value) / 2 },
    { text: "R", x: (xSusEnd.value + xEnd.value)    / 2 },
]);

// ─── Drag state ───────────────────────────────────────────────────────────────

/** ID of the handle currently being dragged, or null. */
const dragging = ref<string | null>(null);

function svgPos(e: MouseEvent): { x: number; y: number } {
    const rect = svgEl.value!.getBoundingClientRect();
    return { x: e.clientX - rect.left, y: e.clientY - rect.top };
}

function startDrag(id: string, e: MouseEvent) {
    dragging.value = id;
    // Prevent text selection while dragging
    e.preventDefault();
}

/**
 * During drag, compute new values from absolute mouse position.
 *
 * Sustain handle ("sus"):
 *   Y only → new sustain level = 1 − (mouseY − PAD) / H
 *
 * Curve handles ("cpA", "cpD", "cpR"):
 *   X → new cp_t = (mouseX − segStart) / segWidth  (clamped to [0.01, 0.99])
 *   Y → new curve = ampToCurve(from, to, amp) where amp = 1 − (mouseY − PAD) / H
 *       Using absolute position avoids the inversion issue: for phases where
 *       amplitude decreases (decay, release), the amplitude formula still maps
 *       dragging down → lower amp → correct curve direction.
 */
function onMouseMove(e: MouseEvent) {
    if (!dragging.value) return;
    const { x, y } = svgPos(e);
    const amp = clamp(1 - (y - PAD) / H, 0, 1);

    if (dragging.value === "sus") {
        emit("update:sustain", amp);
        return;
    }

    if (dragging.value === "cpA") {
        const cpT = clamp((x - xStart.value) / (xPeak.value - xStart.value), 0.01, 0.99);
        emit("update:attackCpT",  cpT);
        emit("update:attackCurve", ampToCurve(0.0, 1.0, amp));
    } else if (dragging.value === "cpD") {
        const cpT = clamp((x - xPeak.value) / (xKnee.value - xPeak.value), 0.01, 0.99);
        emit("update:decayCpT",  cpT);
        emit("update:decayCurve", ampToCurve(1.0, props.sustain, amp));
    } else if (dragging.value === "cpR") {
        const segW = xEnd.value - xSusEnd.value;
        const cpT = segW > 0 ? clamp((x - xSusEnd.value) / segW, 0.01, 0.99) : 0.5;
        emit("update:releaseCpT",  cpT);
        emit("update:releaseCurve", ampToCurve(props.sustain, 0.0, amp));
    }
}

function onMouseUp() {
    dragging.value = null;
}

function clamp(v: number, lo: number, hi: number): number {
    return Math.max(lo, Math.min(hi, v));
}

// ─── Value readout during drag ────────────────────────────────────────────────

const readout = computed<{ x: number; y: number; text: string } | null>(() => {
    const id = dragging.value;
    if (!id) return null;
    let x = 0, y = 0, text = "";
    if (id === "sus") {
        x = xKnee.value;  y = ySustain.value - 12;
        text = `S: ${props.sustain.toFixed(2)}`;
    } else if (id === "cpA") {
        x = cpA.value.x;  y = cpA.value.y - 12;
        text = `curve ${props.attackCurve.toFixed(2)}`;
    } else if (id === "cpD") {
        x = cpD.value.x;  y = cpD.value.y - 12;
        text = `curve ${props.decayCurve.toFixed(2)}`;
    } else if (id === "cpR") {
        x = cpR.value.x;  y = cpR.value.y - 12;
        text = `curve ${props.releaseCurve.toFixed(2)}`;
    }
    return text ? { x, y, text } : null;
});
</script>

<style scoped>
.envelope-svg-wrapper {
    user-select: none;
}

.envelope-svg {
    background: #0f172a;
    border-radius: 6px;
    border: 1px solid rgba(255, 255, 255, 0.07);
    display: block;
}

.handle {
    cursor: grab;
}
.handle:active {
    cursor: grabbing;
}
</style>
