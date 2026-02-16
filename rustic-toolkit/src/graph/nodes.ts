import { defineNode, NodeInterface, type Editor } from "@baklavajs/core";
import {
    NumberInterface,
    SliderInterface,
    CheckboxInterface,
    IntegerInterface,
} from "@baklavajs/renderer-vue";
import { markRaw } from "vue";
import type { GraphMetadata } from "@/types";
import type { Parameter } from "../../src-tauri/bindings/Parameter";
import PlayButton from "@/components/graph/PlayButton.vue";

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

export function registerNodesFromMetadata(editor: Editor, metadata: GraphMetadata): void {
    // Register generators
    for (const gen of metadata.generators) {
        const outputs: Record<string, () => NodeInterface<number>> = {};
        for (let i = 0; i < gen.output_count; i++) {
            const label = gen.output_count === 1 ? "Output" : `Output ${i + 1}`;
            outputs[`out_${i}`] = () => new NodeInterface(label, 0);
        }

        const paramInputs = buildParameterInputs(gen.parameters);
        const inputs = {
            backendNodeId: () => new NodeInterface<string>("Backend Node ID", "").setHidden(true),
            playing: () => new NodeInterface<boolean>("Play", false)
                .setComponent(markRaw(PlayButton))
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
        const inputs: Record<string, () => NodeInterface<any>> = {
            backendNodeId: () => new NodeInterface<string>("Backend Node ID", "").setHidden(true),
        };
        for (let i = 0; i < filter.source_amount; i++) {
            const label = filter.source_amount === 1 ? "Input" : `Input ${i + 1}`;
            inputs[`in_${i}`] = () => new NodeInterface(label, 0);
        }

        const paramInputs = buildParameterInputs(filter.parameters);
        Object.assign(inputs, paramInputs);

        const FilterNode = defineNode({
            type: sanitizeType(filter.name),
            title: filter.name,
            inputs,
            outputs: {
                out_0: () => new NodeInterface("Output", 0),
            },
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