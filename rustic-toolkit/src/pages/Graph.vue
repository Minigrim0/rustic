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
import { onMounted, ref, computed, watch } from "vue";
import { BaklavaEditor, useBaklava } from "@baklavajs/renderer-vue";
import "@baklavajs/themes/dist/syrup-dark.css";
import {
    getGraphMetadata,
    graphAddNode,
    graphRemoveNode,
    graphConnect,
    graphDisconnect,
    graphStartNode,
    graphStopNode,
    graphSetParameter,
    graphCompile,
    graphModulateParameter,
    graphDemodulateParameter,
} from "@/utils/tauri-api";
import { registerNodesFromMetadata, getNodeKind, getPortIndex } from "@/graph/nodes";
import { notifications } from "@/stores/notifications";
import type { AbstractNode, NodeInterface } from "@baklavajs/core";
import EnvelopeEditor from "@/components/graph/EnvelopeEditor.vue";

const baklava = useBaklava();
const isDirty = ref(false);

// ─── Envelope panel state ────────────────────────────────────────────────────

/** The generator node currently shown in the envelope panel (null = hidden). */
const envelopeNode = ref<AbstractNode | null>(null);

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
 * The existing `setValue` subscription in onMounted will pick it up and forward
 * it to the backend via `graphSetParameter`, so no extra IPC call is needed here.
 */
function setEnvParam(paramName: string, value: number) {
    if (!envelopeNode.value) return;
    (envelopeNode.value.inputs as any)[paramName].value = value;
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/** Find the interface key (e.g. "out_0", "in_1") for a given NodeInterface within its owning node. */
function findInterfaceKey(node: AbstractNode, iface: NodeInterface): string | undefined {
    for (const [key, ni] of Object.entries(node.outputs)) {
        if (ni.id === iface.id) return key;
    }
    for (const [key, ni] of Object.entries(node.inputs)) {
        if (ni.id === iface.id) return key;
    }
    return undefined;
}

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

/** Start all selected generator nodes. */
async function playSelected() {
    for (const node of baklava.displayedGraph.selectedNodes) {
        if (getNodeKind(node) !== "Generator") continue;
        const id = Number((node.inputs as any).backendNodeId?.value);
        if (id) {
            try {
                (node.inputs as any).playing.value = true;
                await graphStartNode(id);
            } catch (e) {
                notifications.error(`Failed to start node: ${e}`);
            }
        }
    }
}

/** Stop all selected generator nodes. */
async function stopSelected() {
    for (const node of baklava.displayedGraph.selectedNodes) {
        if (getNodeKind(node) !== "Generator") continue;
        const id = Number((node.inputs as any).backendNodeId?.value);
        if (id) {
            try {
                (node.inputs as any).playing.value = false;
                await graphStopNode(id);
            } catch (e) {
                notifications.error(`Failed to stop node: ${e}`);
            }
        }
    }
}

onMounted(async () => {
    const metadata = await getGraphMetadata();
    registerNodesFromMetadata(baklava.editor, metadata);

    const graph = baklava.editor.graph;

    // --- Node added ---
    graph.events.addNode.subscribe("graph-bridge", async (node) => {
        const kind = getNodeKind(node);
        // Use the machine-readable type_id for generators; fall back to title for filters/sinks
        const typeId = (node.inputs as any).nodeTypeId?.value ?? node.title;
        try {
            const backendId = await graphAddNode(typeId, kind, [0, 0]);
            node.inputs.backendNodeId!.value = String(backendId);
            isDirty.value = true;
        } catch (e) {
            notifications.error(`Failed to add node "${node.title}": ${e}`);
            return;
        }

        // Subscribe to playing toggle for generators
        if (kind === "Generator" && node.inputs.playing) {
            node.inputs.playing.events.setValue.subscribe("graph-bridge-play", (value) => {
                const backendId = Number(node.inputs.backendNodeId?.value);
                if (backendId) {
                    if (value) {
                        graphStartNode(backendId).catch((e) =>
                            notifications.error(`Failed to start node: ${e}`)
                        );
                    } else {
                        graphStopNode(backendId).catch((e) =>
                            notifications.error(`Failed to stop node: ${e}`)
                        );
                    }
                }
            });

            // Open envelope editor when double-clicking a generator node
            // We watch for node selection changes instead
        }

        // Subscribe to parameter changes
        const MIX_MODE_MAP: Record<string, number> = { Sum: 0, Average: 1, Max: 2, Min: 3 };
        for (const [key, iface] of Object.entries(node.inputs)) {
            if (key === "backendNodeId" || key === "nodeTypeId" || key === "playing"
                || key.startsWith("in_") || key.startsWith("mod_")) continue;
            iface.events.setValue.subscribe("graph-bridge-param", (value: any) => {
                const nodeId = Number(node.inputs.backendNodeId?.value);
                if (nodeId) {
                    // mix_mode arrives as a string ("Sum", "Average", …); map to ordinal.
                    const numValue =
                        typeof value === "string" && value in MIX_MODE_MAP
                            ? MIX_MODE_MAP[value]!
                            : Number(value);
                    if (!isNaN(numValue)) {
                        graphSetParameter(nodeId, key, numValue).catch((e) =>
                            notifications.error(`Failed to set parameter "${key}": ${e}`)
                        );
                    }
                }
            });
        }
    });

    // --- Node removed ---
    graph.events.removeNode.subscribe("graph-bridge", (node) => {
        const backendId = Number(node.inputs.backendNodeId?.value);
        if (backendId) {
            isDirty.value = true;
            graphRemoveNode(backendId).catch((e) =>
                notifications.error(`Failed to remove node: ${e}`)
            );
        }
        // Close envelope panel if the removed node was being edited
        if (envelopeNode.value && envelopeNode.value.id === node.id) {
            envelopeNode.value = null;
        }
    });

    // --- Connection added ---
    graph.events.addConnection.subscribe("graph-bridge", (conn) => {
        const fromIface = conn.from;
        const toIface = conn.to;

        const fromNode = graph.nodes.find((n) => n.id === fromIface.nodeId);
        const toNode = graph.nodes.find((n) => n.id === toIface.nodeId);
        if (!fromNode || !toNode) return;

        const fromKey = findInterfaceKey(fromNode, fromIface);
        const toKey = findInterfaceKey(toNode, toIface);
        if (!fromKey || !toKey) return;

        const fromBackendId = Number(fromNode.inputs.backendNodeId?.value);
        const toBackendId = Number(toNode.inputs.backendNodeId?.value);
        if (!fromBackendId || !toBackendId) return;

        isDirty.value = true;
        if (toKey.startsWith("mod_")) {
            // CV modulation connection
            const paramName = toKey.slice(4);
            graphModulateParameter(fromBackendId, toBackendId, paramName).catch(
                (e) => notifications.error(`Failed to add modulation: ${e}`)
            );
        } else {
            graphConnect(fromBackendId, getPortIndex(fromKey), toBackendId, getPortIndex(toKey)).catch(
                (e) => notifications.error(`Failed to connect nodes: ${e}`)
            );
        }
    });

    // --- Connection removed ---
    graph.events.removeConnection.subscribe("graph-bridge", (conn) => {
        const fromIface = conn.from;
        const toIface = conn.to;

        const fromNode = graph.nodes.find((n) => n.id === fromIface.nodeId);
        const toNode = graph.nodes.find((n) => n.id === toIface.nodeId);
        if (!fromNode || !toNode) return;

        const fromBackendId = Number(fromNode.inputs.backendNodeId?.value);
        const toBackendId = Number(toNode.inputs.backendNodeId?.value);
        if (!fromBackendId || !toBackendId) return;

        const toKey = findInterfaceKey(toNode, toIface);
        isDirty.value = true;
        if (toKey?.startsWith("mod_")) {
            const paramName = toKey.slice(4);
            graphDemodulateParameter(fromBackendId, toBackendId, paramName).catch((e) =>
                notifications.error(`Failed to remove modulation: ${e}`)
            );
        } else {
            graphDisconnect(fromBackendId, toBackendId).catch((e) =>
                notifications.error(`Failed to disconnect nodes: ${e}`)
            );
        }
    });

});

// Watch selectedNodes from the renderer (reactive via baklava's view model)
watch(
    () => (baklava.displayedGraph as any).selectedNodes as AbstractNode[],
    (selected) => {
        const firstGenerator = selected?.find((n) => getNodeKind(n) === "Generator") ?? null;
        // Keep panel open if the same node is still selected
        if (firstGenerator) {
            envelopeNode.value = firstGenerator;
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
