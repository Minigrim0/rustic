<template>
    <div class="flex h-full flex-col overflow-hidden bg-gray-100 text-gray-900 dark:bg-gray-950 dark:text-gray-100">
        <!-- Header bar -->
        <div class="flex items-center justify-between border-b border-gray-200 px-4 py-2 dark:border-white/10">
            <div>
                <h1 class="text-sm font-semibold">Graph Playground</h1>
                <p class="text-xs text-gray-500 dark:text-gray-400">Build and preview audio signal graphs</p>
            </div>
            <div class="flex items-center gap-2">
                <button
                    class="rounded border border-indigo-500 px-3 py-1 text-xs text-indigo-500 transition hover:bg-indigo-500 hover:text-white"
                    @click="playSelected"
                >
                    ▶ Play Selected
                </button>
                <button
                    class="rounded border border-gray-400 px-3 py-1 text-xs text-gray-400 transition hover:bg-gray-400 hover:text-white dark:border-gray-500 dark:text-gray-500"
                    @click="stopSelected"
                >
                    ■ Stop Selected
                </button>
                <span
                    v-if="isDirty"
                    class="text-xs text-amber-400 animate-pulse"
                    title="Graph topology changed — compile to apply"
                >● unsync'd</span>
                <button
                    :class="isDirty
                        ? 'rounded border border-amber-500 px-3 py-1 text-xs text-amber-500 transition hover:bg-amber-500 hover:text-white animate-pulse'
                        : 'rounded border border-emerald-500 px-3 py-1 text-xs text-emerald-500 transition hover:bg-emerald-500 hover:text-white'"
                    @click="compile"
                >
                    ⚙ Compile &amp; Send
                </button>
            </div>
        </div>

        <!-- Main area: graph canvas + optional side panel -->
        <div class="flex flex-1 overflow-hidden">
            <!-- Graph canvas -->
            <BaklavaEditor :view-model="baklava" class="flex-1 overflow-hidden" />

            <!-- Envelope editor side panel -->
            <transition name="slide">
                <div
                    v-if="envelopeNode"
                    class="envelope-panel flex flex-col gap-3 border-l border-white/10 bg-gray-900 p-4"
                >
                    <div class="flex items-center justify-between">
                        <span class="text-xs font-semibold text-gray-300">
                            Envelope — {{ envelopeNode.title }}
                        </span>
                        <button
                            class="text-gray-500 hover:text-gray-200 text-lg leading-none"
                            @click="envelopeNode = null"
                            title="Close"
                        >✕</button>
                    </div>
                    <EnvelopeEditor
                        :attack="envAttack"
                        :decay="envDecay"
                        :sustain="envSustain"
                        :release="envRelease"
                        :attack-curve="envAttackCurve"
                        :decay-curve="envDecayCurve"
                        :release-curve="envReleaseCurve"
                        :attack-cp-t="envAttackCpT"
                        :decay-cp-t="envDecayCpT"
                        :release-cp-t="envReleaseCpT"
                        @update:attack="setEnvParam('attack', $event)"
                        @update:decay="setEnvParam('decay', $event)"
                        @update:sustain="setEnvParam('sustain', $event)"
                        @update:release="setEnvParam('release', $event)"
                        @update:attackCurve="setEnvParam('attack_curve', $event)"
                        @update:decayCurve="setEnvParam('decay_curve', $event)"
                        @update:releaseCurve="setEnvParam('release_curve', $event)"
                        @update:attackCpT="setEnvParam('attack_cp_t', $event)"
                        @update:decayCpT="setEnvParam('decay_cp_t', $event)"
                        @update:releaseCpT="setEnvParam('release_cp_t', $event)"
                    />
                </div>
            </transition>
        </div>
    </div>
</template>

<script setup lang="ts">
import { onMounted, computed, watch } from "vue";
import { BaklavaEditor, useBaklava } from "@baklavajs/renderer-vue";
import "@baklavajs/themes/dist/syrup-dark.css";
import {
    getGraphMetadata,
    graphCompile,
} from "@/utils/tauri-api";
import { registerNodesFromMetadata, getNodeKind } from "@/graph/nodes";
import { useGraphBridge } from "@/graph/useGraphBridge";
import { notifications } from "@/stores/notifications";
import type { AbstractNode } from "@baklavajs/core";
import EnvelopeEditor from "@/components/graph/EnvelopeEditor.vue";

const baklava = useBaklava();
const { isDirty, envelopeNode } = useGraphBridge(baklava);

// ─── Envelope panel state ────────────────────────────────────────────────────

/** Read a numeric node interface value, falling back to `def` when the interface is absent. */
function nodeParam(paramName: string, def: number): number {
    return Number((envelopeNode.value?.inputs as any)?.[paramName]?.value ?? def);
}

const envAttack       = computed(() => nodeParam("attack",        0.01));
const envDecay        = computed(() => nodeParam("decay",         0.1 ));
const envSustain      = computed(() => nodeParam("sustain",       0.8 ));
const envRelease      = computed(() => nodeParam("release",       0.3 ));
const envAttackCurve  = computed(() => nodeParam("attack_curve",  0.0 ));
const envDecayCurve   = computed(() => nodeParam("decay_curve",   0.0 ));
const envReleaseCurve = computed(() => nodeParam("release_curve", 0.0 ));
const envAttackCpT    = computed(() => nodeParam("attack_cp_t",   0.5 ));
const envDecayCpT     = computed(() => nodeParam("decay_cp_t",    0.5 ));
const envReleaseCpT   = computed(() => nodeParam("release_cp_t",  0.5 ));

/**
 * Write a value into a node interface by name.
 * The existing `setValue` subscription in useGraphBridge will pick it up and forward
 * it to the backend via `graphSetParameter`, so no extra IPC call is needed here.
 */
function setEnvParam(paramName: string, value: number) {
    if (!envelopeNode.value) return;
    (envelopeNode.value.inputs as any)[paramName].value = value;
}

// ─── Playback ────────────────────────────────────────────────────────────────

/** Compile the current graph and push it to the render thread. */
async function compile() {
    try {
        await graphCompile();
        isDirty.value = false;
        notifications.info("Graph compiled and sent to audio engine.");
    } catch (e) {
        notifications.error(`Compile failed: ${e}`);
    }
}

/** Start all selected generator / Trigger nodes. */
async function playSelected() {
    for (const node of baklava.displayedGraph.selectedNodes) {
        if (getNodeKind(node) !== "Generator" && node.title !== "Trigger") continue;
        (node.inputs as any).playing.value = "playing";
    }
}

/** Stop (graceful release) all selected generator / Trigger nodes. */
async function stopSelected() {
    for (const node of baklava.displayedGraph.selectedNodes) {
        if (getNodeKind(node) !== "Generator" && node.title !== "Trigger") continue;
        (node.inputs as any).playing.value = "releasing";
    }
}

onMounted(async () => {
    const metadata = await getGraphMetadata();
    registerNodesFromMetadata(baklava.editor, metadata);
});

// Watch selectedNodes from the renderer (reactive via baklava's view model)
watch(
    () => (baklava.displayedGraph as any).selectedNodes as AbstractNode[],
    (selected) => {
        const firstEnvNode =
            selected?.find(
                (n) => getNodeKind(n) === "Generator" || n.title === "Trigger"
            ) ?? null;
        // Keep panel open if the same node is still selected
        if (firstEnvNode) {
            envelopeNode.value = firstEnvNode;
        } else if (!selected?.some((n) => n.id === envelopeNode.value?.id)) {
            // Previously displayed node was deselected
            envelopeNode.value = null;
        }
    },
    { deep: true }
);
</script>

<style scoped>
.envelope-panel {
    width: 340px;
    min-width: 340px;
    overflow-y: auto;
}

.slide-enter-active,
.slide-leave-active {
    transition: width 0.2s ease, opacity 0.2s ease;
    overflow: hidden;
}
.slide-enter-from,
.slide-leave-to {
    width: 0;
    opacity: 0;
}
</style>
