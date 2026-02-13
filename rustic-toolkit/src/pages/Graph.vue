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
        <div ref="graphContainer" class="flex-1 overflow-hidden"></div>
    </div>
</template>

<script lang="ts">
export default {
    name: "GraphPlayground",
    data() {
        return {};
    },
    mounted() {},
    methods: {},
};
</script>