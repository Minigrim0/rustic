import { reactive, watch } from "vue";
import { defaults, sections } from "./settingsSchema";

const STORAGE_KEY = "rustic:settings";

type SettingsData = Record<string, Record<string, unknown>>;

function buildDefaults(): SettingsData {
    const result: SettingsData = {};
    for (const section of sections) {
        result[section.id] = { ...(defaults[section.id] ?? {}) };
    }
    return result;
}

function load(): SettingsData {
    const base = buildDefaults();
    try {
        const raw = localStorage.getItem(STORAGE_KEY);
        if (raw) {
            const saved = JSON.parse(raw) as SettingsData;
            // Merge saved values into defaults (handles new fields gracefully)
            for (const sectionId of Object.keys(base)) {
                if (saved[sectionId]) {
                    Object.assign(base[sectionId]!, saved[sectionId]);
                }
            }
        }
    } catch { /* ignore corrupt data */ }
    return base;
}

/** Central settings store. Keyed by section id, e.g. `settingsStore.spectrum.topCount`. */
export const settingsStore = reactive<SettingsData>(load());

// Persist on any change
watch(settingsStore, (val) => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(val));
}, { deep: true });

/** Reset a single section to its schema defaults. */
export function resetSection(sectionId: string): void {
    const sectionDefaults = defaults[sectionId];
    if (!sectionDefaults || !settingsStore[sectionId]) return;
    Object.assign(settingsStore[sectionId]!, { ...sectionDefaults });
}

/** Reset all sections to their schema defaults. */
export function resetAll(): void {
    for (const sectionId of Object.keys(defaults)) {
        resetSection(sectionId);
    }
}