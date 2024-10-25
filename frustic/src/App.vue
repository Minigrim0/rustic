<script setup lang="ts">
// import { RouterLink, RouterView } from 'vue-router'
// import HelloWorld from './components/HelloWorld.vue'

import { BaklavaEditor, useBaklava } from '@baklavajs/renderer-vue';
import "@baklavajs/themes/dist/syrup-dark.css";
import SumCombinator from "./components/nodes/testnode";

const baklava = useBaklava();
baklava.editor.registerNodeType(SumCombinator);
// return { baklava };
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
