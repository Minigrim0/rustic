<script lang="ts">
// import { RouterLink, RouterView } from 'vue-router'
// import HelloWorld from './components/HelloWorld.vue'

import { invoke } from '@tauri-apps/api/core';

import { BaklavaEditor, useBaklava } from '@baklavajs/renderer-vue';
import "@baklavajs/themes/dist/syrup-dark.css";
import { defineNode, NodeInterface, NumberInterface } from "baklavajs";

import SumCombinator from "./components/nodes/testnode";

export default {
    data() {
        return {
            baklava: useBaklava(),
        }
    },
    components: {
        // HelloWorld,
        // RouterLink,
        // RouterView,
        BaklavaEditor,
    },
    mounted() {
        invoke('get_filters').then((values: [{ name: string, description: string, inputs: number, outputs: number }] | Error) => {
            if (values instanceof Error) {
                console.error(values);
                return;
            }
            values.map(
                (value) => {
                    console.log("working on: ", value);
                    let inputs: { [id: string]: NumberInterface } = {};
                    for (var input_index = 0; input_index < value.inputs; input_index ++) {
                        inputs["input" + input_index.toString()] = () => new NumberInterface("Input", 0);
                    }

                    let outputs: { [id: string]: NodeInterface } = {};
                    for (var output_index = 0; output_index < value.outputs; output_index ++) {
                        outputs["output" + output_index.toString()] = () => new NodeInterface("Output", 0);
                    }

                    return defineNode({
                        type: value.name,
                        inputs: inputs,
                        outputs: outputs,
                    })
                }
            )
            .forEach((node) => {
                console.log("Registering new node: ", node);
                this.baklava.editor.registerNodeType(node);
            });
        });
    }
};
</script>

<template>
    <nav>
        <h1>Rustic</h1>
    </nav>

    <div class="editor">
        <BaklavaEditor :view-model="baklava" />
    </div>

  <!-- <RouterView /> -->
</template>
