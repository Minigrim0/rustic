<template>
    <div class="h-full overflow-y-auto bg-gray-50 p-4 dark:bg-gray-950">
        <h1 class="mb-4 text-sm font-semibold text-gray-700 dark:text-gray-300">Settings</h1>

        <div class="max-w-lg space-y-3">
            <section v-for="section in sections" :key="section.id"
                class="rounded-lg border border-gray-200 bg-white dark:border-white/10 dark:bg-gray-900">
                <!-- Section header -->
                <div class="flex items-center gap-2 border-b border-gray-100 px-3 py-2 dark:border-white/5">
                    <svg class="h-4 w-4 text-gray-400 dark:text-gray-500" fill="none" viewBox="0 0 24 24"
                        stroke="currentColor" stroke-width="1.5">
                        <path stroke-linecap="round" stroke-linejoin="round" :d="section.icon" />
                    </svg>
                    <span class="text-xs font-medium text-gray-600 dark:text-gray-400">{{ section.label }}</span>
                    <span v-if="section.requiresRestart"
                        class="ml-auto rounded bg-amber-100 px-1.5 py-0.5 text-[9px] font-semibold text-amber-700 dark:bg-amber-500/20 dark:text-amber-400">
                        Restart required
                    </span>
                </div>

                <!-- Fields -->
                <div class="px-3 py-3">
                    <FormKit type="group" v-model="storeFor(section)![section.id]">
                        <FormKitSchema :schema="section.schema" />
                    </FormKit>

                    <div class="mt-2 flex justify-end">
                        <button @click="resetSectionFor(section)"
                            class="rounded px-2 py-0.5 text-[10px] font-medium text-gray-400 transition-colors hover:bg-gray-100 hover:text-gray-600 dark:text-gray-500 dark:hover:bg-white/5 dark:hover:text-gray-300">
                            Reset to defaults
                        </button>
                    </div>
                </div>
            </section>

            <!-- Global reset -->
            <div class="flex justify-end pt-1">
                <button @click="resetAll"
                    class="rounded px-2 py-1 text-[11px] font-medium text-gray-400 transition-colors hover:bg-red-50 hover:text-red-500 dark:text-gray-500 dark:hover:bg-red-500/10 dark:hover:text-red-400">
                    Reset all settings
                </button>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { onMounted } from "vue";
import { FormKitSchema } from "@formkit/vue";
import { sections, type SettingsSection } from "../stores/settingsSchema";
import {
    settingsStore,
    engineStore,
    resetSection,
    resetEngineSection,
    resetAll,
    loadEngineSettings,
} from "../stores/settings";

/** Return the correct reactive store for a given section. */
function storeFor(section: SettingsSection) {
    return section.storage === "engine" ? engineStore : settingsStore;
}

/** Reset the correct store section. */
function resetSectionFor(section: SettingsSection) {
    if (section.storage === "engine") {
        resetEngineSection(section.id);
    } else {
        resetSection(section.id);
    }
}

onMounted(() => {
    loadEngineSettings();
});
</script>
