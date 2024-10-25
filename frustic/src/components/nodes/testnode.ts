import { defineNode, NodeInterface, NumberInterface, SelectInterface } from "baklavajs";

export default defineNode({
    type: "SumCombinator",
    inputs: {
        input1: () => new NumberInterface("Number", 0.0),
        input2: () => new NumberInterface("Number", 0.0),
        // operation: () => new SelectInterface("Operation", "Add", ["Add", "Subtract"]).setPort(false),
    },
    outputs: {
        output: () => new NodeInterface("Output", 0),
    },
});
