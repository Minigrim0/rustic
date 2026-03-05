import { defineNode, NodeInterface, type Editor } from "@baklavajs/core";
import {
    NumberInterface,
    SliderInterface,
    CheckboxInterface,
    IntegerInterface,
    SelectInterface,
} from "@baklavajs/renderer-vue";
import { markRaw } from "vue";
import type { GraphMetadata } from "@/types";
import type { Parameter } from "../../src-tauri/bindings/Parameter";
import NodeControls from "@/components/graph/NodeControls.vue";

type ParamStr = Parameter<string>;

function sanitizeType(name: string): string {
    return name.replace(/\s+/g, "");
}

function createParameterInterface(param: ParamStr): () => NodeInterface<any> {
    if ("Range" in param) {
        const { title, default: def, min, max } = param.Range;
        return () => new SliderInterface(title, def, min, max).setPort(false);
    }
    if ("Float" in param) {
        const { title, default: def } = param.Float;
        return () => new NumberInterface(title, def).setPort(false);
    }
    if ("Toggle" in param) {
        const { title, default: def } = param.Toggle;
        return () => new CheckboxInterface(title, def).setPort(false);
    }
    if ("Int" in param) {
        const { title, default: def, min, max } = param.Int;
        return () =>
            new IntegerInterface(title, def, min ?? undefined, max ?? undefined).setPort(false);
    }
    // List parameters are skipped for now
    return () => new NumberInterface("Unknown", 0).setPort(false);
}

function getFieldName(param: ParamStr): string {
    const inner = Object.values(param)[0] as Record<string, any>;
    return inner.field_name;
}

function buildParameterInputs(params: ParamStr[]): Record<string, () => NodeInterface<any>> {
    const inputs: Record<string, () => NodeInterface<any>> = {};
    for (const param of params) {
        if ("List" in param) continue;
        inputs[getFieldName(param)] = createParameterInterface(param);
    }
    return inputs;
}

/**
 * Parameters managed exclusively by the envelope editor panel.
 * They must exist as node interface entries so the setValue subscription bridge
 * can forward changes to the backend, but they must not render as visible controls
 * inside the node itself — that would duplicate the panel's interactive SVG.
 */
const ENVELOPE_PARAMS = new Set([
    "attack", "decay", "sustain", "release",
    "attack_curve", "decay_curve", "release_curve",
    "attack_cp_t", "decay_cp_t", "release_cp_t",
]);

/**
 * Like buildParameterInputs but for generator nodes.
 * Params listed in ENVELOPE_PARAMS become hidden NodeInterface<number> entries
 * instead of visible sliders; everything else goes through createParameterInterface
 * and also gets a connectable `mod_<field>` CV input port.
 */
function buildGeneratorInputs(params: ParamStr[]): Record<string, () => NodeInterface<any>> {
    const inputs: Record<string, () => NodeInterface<any>> = {};
    for (const param of params) {
        if ("List" in param) continue;
        const field = getFieldName(param);
        if (ENVELOPE_PARAMS.has(field)) {
            // Default value: use whatever the metadata says so the backend starts in sync.
            const def = "Range" in param ? param.Range.default
                      : "Float" in param ? param.Float.default
                      : 0;
            inputs[field] = () => new NodeInterface<number>(field, def).setPort(false).setHidden(true);
        } else {
            inputs[field] = createParameterInterface(param);
            // CV modulation input port (connectable socket, not a value control)
            inputs[`mod_${field}`] = () => new NodeInterface<number>(`↗ ${field}`, 0);
        }
    }
    return inputs;
}

/** Infer the node kind from its interface keys. */
export function getNodeKind(node: any): "Generator" | "Filter" | "Sink" {
    const outputKeys = Object.keys(node.outputs ?? {});
    const inputKeys = Object.keys(node.inputs ?? {});
    const hasOut = outputKeys.some((k) => k.startsWith("out_"));
    const hasIn = inputKeys.some((k) => k.startsWith("in_"));
    if (hasOut && !hasIn) return "Generator";
    if (!hasOut && hasIn) return "Sink";
    return "Filter";
}

/** Extract the numeric port index from an interface key like `in_0` or `out_1`. */
export function getPortIndex(interfaceKey: string): number {
    const m = interfaceKey.match(/_(\d+)$/);
    return m ? parseInt(m[1]!, 10) : 0;
}

export function registerNodesFromMetadata(editor: Editor, metadata: GraphMetadata): void {
    // Register generators
    for (const gen of metadata.generators) {
        const outputs: Record<string, () => NodeInterface<number>> = {};
        for (let i = 0; i < gen.output_count; i++) {
            const label = gen.output_count === 1 ? "Output" : `Output ${i + 1}`;
            outputs[`out_${i}`] = () => new NodeInterface(label, 0);
        }

        const paramInputs = buildGeneratorInputs(gen.parameters);
        const typeId = gen.type_id;
        const inputs = {
            backendNodeId: () => new NodeInterface<string>("Backend Node ID", "").setHidden(true),
            nodeTypeId: () => new NodeInterface<string>("Node Type ID", typeId).setHidden(true),
            playing: () =>
                new NodeInterface<string>("Controls", "idle")
                    .setComponent(markRaw(NodeControls))
                    .setPort(false),
            ...paramInputs,
        };

        const GeneratorNode = defineNode({
            type: sanitizeType(gen.name),
            title: gen.name,
            inputs,
            outputs,
        });
        editor.registerNodeType(GeneratorNode, { category: "Generators" });
    }

    // Register filters
    for (const filter of metadata.filters) {
        // Trigger node gets special treatment: no mix_mode, hidden ADSR params, NodeControls
        if (filter.name === "Trigger") {
            const inputs: Record<string, () => NodeInterface<any>> = {
                backendNodeId: () =>
                    new NodeInterface<string>("Backend Node ID", "").setHidden(true),
                playing: () =>
                    new NodeInterface<string>("Controls", "idle")
                        .setComponent(markRaw(NodeControls))
                        .setPort(false),
            };

            let audioPortIdx = 0;
            for (const inp of filter.inputs) {
                if (inp.parameter === null) {
                    // Audio input port
                    const idx = audioPortIdx++;
                    const label = inp.label ?? `Input ${idx + 1}`;
                    inputs[`in_${idx}`] = () => new NodeInterface(label, 0);
                } else {
                    // ADSR parameters — hidden (managed by envelope editor panel)
                    const field = getFieldName(inp.parameter);
                    if (ENVELOPE_PARAMS.has(field)) {
                        const def =
                            "Range" in inp.parameter
                                ? inp.parameter.Range.default
                                : "Float" in inp.parameter
                                ? inp.parameter.Float.default
                                : 0;
                        inputs[field] = () =>
                            new NodeInterface<number>(field, def)
                                .setPort(false)
                                .setHidden(true);
                    }
                    // gate is not in metadata — skip
                }
            }

            const outputs: Record<string, () => NodeInterface<any>> = {
                out_0: () => new NodeInterface("Output", 0),
            };

            const TriggerNode = defineNode({
                type: "Trigger",
                title: "Trigger",
                inputs,
                outputs,
            });
            editor.registerNodeType(TriggerNode, { category: "Filters" });
            continue;
        }

        // Normal filter registration
        const inputs: Record<string, () => NodeInterface<any>> = {
            backendNodeId: () => new NodeInterface<string>("Backend Node ID", "").setHidden(true),
            mix_mode: () =>
                new SelectInterface("Mix Mode", "Sum", ["Sum", "Average", "Max", "Min"]).setPort(
                    false
                ),
        };

        const audioInputCount = filter.inputs.filter((inp) => inp.parameter === null).length;
        let audioPortIdx = 0;
        for (const input of filter.inputs) {
            if (input.parameter === null) {
                const idx = audioPortIdx++;
                const label = input.label ?? (audioInputCount === 1 ? "Input" : `Input ${idx + 1}`);
                inputs[`in_${idx}`] = () => new NodeInterface(label, 0);
            } else {
                const fieldName = getFieldName(input.parameter);
                inputs[fieldName] = createParameterInterface(input.parameter);
                // CV modulation input port (skip mix_mode — it's an enum selector, not a number)
                if (fieldName !== "mix_mode") {
                    inputs[`mod_${fieldName}`] = () =>
                        new NodeInterface<number>(`↗ ${fieldName}`, 0);
                }
            }
        }

        const outputs: Record<string, () => NodeInterface<any>> = {};
        for (let i = 0; i < filter.outputs; i++) {
            const label = filter.outputs === 1 ? "Output" : `Output ${i + 1}`;
            outputs[`out_${i}`] = () => new NodeInterface(label, 0);
        }

        const FilterNode = defineNode({
            type: sanitizeType(filter.name),
            title: filter.name,
            inputs,
            outputs,
        });
        editor.registerNodeType(FilterNode, { category: "Filters" });
    }

    // Register sinks
    for (const sink of metadata.sinks) {
        const inputs: Record<string, () => NodeInterface<any>> = {
            backendNodeId: () => new NodeInterface<string>("Backend Node ID", "").setHidden(true),
        };
        for (let i = 0; i < sink.input_count; i++) {
            const label = sink.input_count === 1 ? "Input" : `Input ${i + 1}`;
            inputs[`in_${i}`] = () => new NodeInterface(label, 0);
        }

        const SinkNode = defineNode({
            type: sanitizeType(sink.name),
            title: sink.name,
            inputs,
        });
        editor.registerNodeType(SinkNode, { category: "Sinks" });
    }
}
