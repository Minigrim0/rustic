<template>
    <div class="flex h-full flex-col overflow-hidden bg-gray-100 text-gray-900 dark:bg-gray-950 dark:text-gray-100">
        <!-- Header bar -->
        <div class="flex items-center justify-between border-b border-gray-200 px-4 py-2 dark:border-white/10">
            <div>
                <h1 class="text-sm font-semibold">Graph Playground</h1>
                <p class="text-xs text-gray-500 dark:text-gray-400">Build and preview audio signal graphs</p>
            </div>
        </div>

        <!-- Graph canvas -->
        <BaklavaEditor :view-model="baklava" />
    </div>
</template>

<script setup lang="ts">
import { onMounted } from "vue";
import { BaklavaEditor, useBaklava } from "@baklavajs/renderer-vue";
import "@baklavajs/themes/dist/syrup-dark.css";
import { getGraphMetadata, setRenderMode, graphAddNode, graphRemoveNode, graphConnect, graphDisconnect, graphStartNode, graphStopNode, graphSetParameter } from "@/utils/tauri-api";
import { registerNodesFromMetadata, getNodeKind, getPortIndex } from "@/graph/nodes";
import type { AbstractNode, NodeInterface } from "@baklavajs/core";

const baklava = useBaklava();

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

onMounted(async () => {
    const metadata = await getGraphMetadata();
    registerNodesFromMetadata(baklava.editor, metadata);

    // Change render mode in the backend
    await setRenderMode("graph");

    const graph = baklava.editor.graph;

    // --- Node added ---
    graph.events.addNode.subscribe("graph-bridge", async (node) => {
        const kind = getNodeKind(node);
        const backendId = await graphAddNode(node.title, kind, [0, 0]);
        node.inputs.backendNodeId.value = String(backendId);

        // Subscribe to playing toggle for generators
        if (kind === "Generator" && node.inputs.playing) {
            node.inputs.playing.events.setValue.subscribe("graph-bridge-play", ([value]) => {
                const backendId = Number(node.inputs.backendNodeId.value);
                if (backendId) {
                    if (value) {
                        graphStartNode(backendId);
                    } else {
                        graphStopNode(backendId);
                    }
                }
            });
        }

        // Subscribe to parameter changes
        for (const [key, iface] of Object.entries(node.inputs)) {
            if (key === "backendNodeId" || key === "playing" || key.startsWith("in_")) continue;
            iface.events.setValue.subscribe("graph-bridge-param", (value: any) => {
                const nodeId = Number(node.inputs.backendNodeId.value);
                if (nodeId) {
                    graphSetParameter(nodeId, key, Number(value));
                }
            });
        }
    });

    // --- Node removed ---
    graph.events.removeNode.subscribe("graph-bridge", (node) => {
        const backendId = Number(node.inputs.backendNodeId?.value);
        if (backendId) {
            graphRemoveNode(backendId);
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

        const fromBackendId = Number(fromNode.inputs.backendNodeId.value);
        const toBackendId = Number(toNode.inputs.backendNodeId.value);
        if (!fromBackendId || !toBackendId) return;

        graphConnect(fromBackendId, getPortIndex(fromKey), toBackendId, getPortIndex(toKey));
    });

    // --- Connection removed ---
    graph.events.removeConnection.subscribe("graph-bridge", (conn) => {
        const fromIface = conn.from;
        const toIface = conn.to;

        const fromNode = graph.nodes.find((n) => n.id === fromIface.nodeId);
        const toNode = graph.nodes.find((n) => n.id === toIface.nodeId);
        if (!fromNode || !toNode) return;

        const fromBackendId = Number(fromNode.inputs.backendNodeId.value);
        const toBackendId = Number(toNode.inputs.backendNodeId.value);
        if (!fromBackendId || !toBackendId) return;

        graphDisconnect(fromBackendId, toBackendId);
    });
});
</script>
