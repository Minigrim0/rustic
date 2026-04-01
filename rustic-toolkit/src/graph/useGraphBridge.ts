/**
 * useGraphBridge — composable that bridges BaklavaJS graph events to the Rustic backend.
 *
 * Subscribes to all graph add/remove/connect/disconnect events and forwards
 * them as Tauri IPC calls. Also handles generator play-state changes (▶ ⏸ ✕)
 * and Trigger node group-start/stop orchestration.
 *
 * Returns `isDirty` and `envelopeNode` refs consumed by `Graph.vue`.
 */
import { ref, nextTick, onMounted } from "vue";
import type { AbstractNode, NodeInterface } from "@baklavajs/core";
import type { useBaklava } from "@baklavajs/renderer-vue";
import {
    graphAddNode,
    graphRemoveNode,
    graphConnect,
    graphDisconnect,
    graphStartNode,
    graphStopNode,
    graphKillNode,
    graphSetParameter,
    graphModulateParameter,
    graphDemodulateParameter,
    graphTriggerPlay,
    graphTriggerStop,
    graphTriggerKill,
} from "@/utils/tauri-api";
import { getNodeKind, getPortIndex } from "@/graph/nodes";
import { notifications } from "@/stores/notifications";

/** Find the interface key (e.g. "out_0", "in_1") for a given NodeInterface within a node. */
function findInterfaceKey(node: AbstractNode, iface: NodeInterface): string | undefined {
    for (const [key, ni] of Object.entries(node.outputs)) {
        if (ni.id === iface.id) return key;
    }
    for (const [key, ni] of Object.entries(node.inputs)) {
        if (ni.id === iface.id) return key;
    }
    return undefined;
}

const MIX_MODE_MAP: Record<string, number> = { Sum: 0, Average: 1, Max: 2, Min: 3 };

export function useGraphBridge(baklava: ReturnType<typeof useBaklava>) {
    const isDirty = ref(false);
    const envelopeNode = ref<AbstractNode | null>(null);

    onMounted(async () => {
        const graph = baklava.editor.graph;

        // --- Node added ---
        graph.events.addNode.subscribe("graph-bridge", async (node) => {
            const kind = getNodeKind(node);
            const typeId = (node.inputs as any).nodeTypeId?.value ?? node.type;
            try {
                const backendId = await graphAddNode(typeId, kind, [0, 0]);
                node.inputs.backendNodeId!.value = String(backendId);
                isDirty.value = true;
            } catch (e) {
                notifications.error(`Failed to add node "${node.title}": ${e}`);
                return;
            }

            // Generator play-state subscription
            if (kind === "Generator" && node.inputs.playing) {
                node.inputs.playing.events.setValue.subscribe("graph-bridge-play", (value: any) => {
                    const id = Number((node.inputs as any).backendNodeId?.value);
                    if (!id) return;
                    if (value === "playing") {
                        graphStartNode(id).catch((e) =>
                            notifications.error(`Failed to start node: ${e}`)
                        );
                    } else if (value === "releasing") {
                        graphStopNode(id).catch((e) =>
                            notifications.error(`Failed to stop node: ${e}`)
                        );
                    } else if (value === "killed") {
                        graphKillNode(id).catch((e) =>
                            notifications.error(`Failed to kill node: ${e}`)
                        );
                        // Reset to idle after kill
                        nextTick(() => {
                            (node.inputs as any).playing.value = "idle";
                        });
                    }
                });
            }

            // Trigger node play-state subscription
            if (node.title === "Trigger" && node.inputs.playing) {
                node.inputs.playing.events.setValue.subscribe("graph-bridge-trigger", (value: any) => {
                    const triggerId = Number((node.inputs as any).backendNodeId?.value);
                    if (!triggerId) return;

                    // Find all generator nodes connected to any in_* port of this trigger
                    const upstream = baklava.displayedGraph.connections
                        .filter((c) => c.to.nodeId === node.id)
                        .map((c) =>
                            baklava.displayedGraph.nodes.find((n) => n.id === c.from.nodeId)
                        )
                        .filter((n) => n && getNodeKind(n) === "Generator");

                    if (value === "playing") {
                        graphTriggerPlay(triggerId).catch((e) =>
                            notifications.error(`Trigger play failed: ${e}`)
                        );
                        for (const upNode of upstream) {
                            const id = Number((upNode!.inputs as any).backendNodeId?.value);
                            if (!id) continue;
                            (upNode!.inputs as any).playing.value = "playing";
                            graphStartNode(id).catch((e) =>
                                notifications.error(`Failed to start upstream node: ${e}`)
                            );
                        }
                    } else if (value === "releasing") {
                        graphTriggerStop(triggerId).catch((e) =>
                            notifications.error(`Trigger stop failed: ${e}`)
                        );
                        for (const upNode of upstream) {
                            const id = Number((upNode!.inputs as any).backendNodeId?.value);
                            if (!id) continue;
                            (upNode!.inputs as any).playing.value = "releasing";
                            graphStopNode(id).catch((e) =>
                                notifications.error(`Failed to stop upstream node: ${e}`)
                            );
                        }
                    } else if (value === "killed") {
                        graphTriggerKill(triggerId).catch((e) =>
                            notifications.error(`Trigger kill failed: ${e}`)
                        );
                        for (const upNode of upstream) {
                            const id = Number((upNode!.inputs as any).backendNodeId?.value);
                            if (!id) continue;
                            (upNode!.inputs as any).playing.value = "idle";
                            graphKillNode(id).catch((e) =>
                                notifications.error(`Failed to kill upstream node: ${e}`)
                            );
                        }
                        nextTick(() => {
                            (node.inputs as any).playing.value = "idle";
                        });
                    }
                });
            }

            // Subscribe to parameter changes for all non-structural inputs
            for (const [key, iface] of Object.entries(node.inputs)) {
                if (
                    key === "backendNodeId" ||
                    key === "nodeTypeId" ||
                    key === "playing" ||
                    key.startsWith("in_") ||
                    key.startsWith("mod_")
                )
                    continue;
                iface.events.setValue.subscribe("graph-bridge-param", (value: any) => {
                    const nodeId = Number((node.inputs as any).backendNodeId?.value);
                    if (nodeId) {
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
            const backendId = Number((node.inputs as any).backendNodeId?.value);
            if (backendId) {
                isDirty.value = true;
                graphRemoveNode(backendId).catch((e) =>
                    notifications.error(`Failed to remove node: ${e}`)
                );
            }
            if (envelopeNode.value && envelopeNode.value.id === node.id) {
                envelopeNode.value = null;
            }
        });

        // --- Connection added ---
        graph.events.addConnection.subscribe("graph-bridge", (conn) => {
            const fromNode = graph.nodes.find((n) => n.id === conn.from.nodeId);
            const toNode = graph.nodes.find((n) => n.id === conn.to.nodeId);
            if (!fromNode || !toNode) return;

            const fromKey = findInterfaceKey(fromNode, conn.from);
            const toKey = findInterfaceKey(toNode, conn.to);
            if (!fromKey || !toKey) return;

            const fromBackendId = Number((fromNode.inputs as any).backendNodeId?.value);
            const toBackendId = Number((toNode.inputs as any).backendNodeId?.value);
            if (!fromBackendId || !toBackendId) return;

            isDirty.value = true;
            if (toKey.startsWith("mod_")) {
                const paramName = toKey.slice(4);
                graphModulateParameter(fromBackendId, toBackendId, paramName).catch((e) =>
                    notifications.error(`Failed to add modulation: ${e}`)
                );
            } else {
                graphConnect(
                    fromBackendId,
                    getPortIndex(fromKey),
                    toBackendId,
                    getPortIndex(toKey)
                ).catch((e) => notifications.error(`Failed to connect nodes: ${e}`));
            }
        });

        // --- Connection removed ---
        graph.events.removeConnection.subscribe("graph-bridge", (conn) => {
            const fromNode = graph.nodes.find((n) => n.id === conn.from.nodeId);
            const toNode = graph.nodes.find((n) => n.id === conn.to.nodeId);
            if (!fromNode || !toNode) return;

            const fromBackendId = Number((fromNode.inputs as any).backendNodeId?.value);
            const toBackendId = Number((toNode.inputs as any).backendNodeId?.value);
            if (!fromBackendId || !toBackendId) return;

            const toKey = findInterfaceKey(toNode, conn.to);
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

    return { isDirty, envelopeNode };
}
