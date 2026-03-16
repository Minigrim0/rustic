import { reactive, watch } from "vue";
import { defaults, sections } from "./settingsSchema";
import { getEngineConfig, setEngineConfig } from "@/utils/tauri-api";
import type { EngineConfig } from "@/types";

const STORAGE_KEY = "rustic:settings";

type SettingsData = Record<string, Record<string, unknown>>;

// ---------- local (localStorage) store ----------

function buildDefaults(): SettingsData {
    const result: SettingsData = {};
    for (const section of sections) {
        if (!section.storage || section.storage === "local") {
            result[section.id] = { ...(defaults[section.id] ?? {}) };
        }
    }
    return result;
}

function load(): SettingsData {
    const base = buildDefaults();
    try {
        const raw = localStorage.getItem(STORAGE_KEY);
        if (raw) {
            const saved = JSON.parse(raw) as SettingsData;
            for (const sectionId of Object.keys(base)) {
                if (saved[sectionId]) {
                    Object.assign(base[sectionId]!, saved[sectionId]);
                }
            }
        }
    } catch { /* ignore corrupt data */ }
    return base;
}

/** Central settings store for local (localStorage) sections. */
export const settingsStore = reactive<SettingsData>(load());

watch(settingsStore, (val) => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(val));
}, { deep: true });

/** Reset a single local section to its schema defaults. */
export function resetSection(sectionId: string): void {
    const sectionDefaults = defaults[sectionId];
    if (!sectionDefaults || !settingsStore[sectionId]) return;
    Object.assign(settingsStore[sectionId]!, { ...sectionDefaults });
}

/** Reset all local sections to their schema defaults. */
export function resetAll(): void {
    for (const sectionId of Object.keys(settingsStore)) {
        resetSection(sectionId);
    }
    resetAllEngine();
}

// ---------- engine (config.toml) store ----------

function buildEngineDefaults(): SettingsData {
    const result: SettingsData = {};
    for (const section of sections) {
        if (section.storage === "engine") {
            result[section.id] = { ...(defaults[section.id] ?? {}) };
        }
    }
    return result;
}

/** Reactive store for engine settings (backed by config.toml). */
export const engineStore = reactive<SettingsData>(buildEngineDefaults());

let engineSaveTimer: ReturnType<typeof setTimeout> | null = null;

/** Load engine settings from the backend. */
export async function loadEngineSettings(): Promise<void> {
    try {
        const config = await getEngineConfig();
        // Map EngineConfig fields into the reactive store by section
        for (const section of sections) {
            if (section.storage === "engine" && section.engineKey) {
                const data = config[section.engineKey as keyof EngineConfig];
                if (data && typeof data === "object") {
                    if (!engineStore[section.id]) {
                        engineStore[section.id] = {};
                    }
                    Object.assign(engineStore[section.id]!, data);
                }
            }
        }
    } catch (e) {
        console.error("Failed to load engine settings:", e);
    }
}

/** Save engine settings back to config.toml via the backend. */
export async function saveEngineSettings(): Promise<void> {
    // Build an EngineConfig from the reactive store
    const config: Record<string, unknown> = {};
    for (const section of sections) {
        if (section.storage === "engine" && section.engineKey) {
            config[section.engineKey] = { ...engineStore[section.id] };
        }
    }
    try {
        await setEngineConfig(config as unknown as EngineConfig);
    } catch (e) {
        console.error("Failed to save engine settings:", e);
    }
}

// Debounced auto-save on engine store changes
watch(engineStore, () => {
    if (engineSaveTimer) clearTimeout(engineSaveTimer);
    engineSaveTimer = setTimeout(() => {
        saveEngineSettings();
    }, 500);
}, { deep: true });

/** Reset a single engine section to its defaults. */
export function resetEngineSection(sectionId: string): void {
    const sectionDefaults = defaults[sectionId];
    if (!sectionDefaults || !engineStore[sectionId]) return;
    Object.assign(engineStore[sectionId]!, { ...sectionDefaults });
}

/** Reset all engine sections to their defaults. */
export function resetAllEngine(): void {
    for (const section of sections) {
        if (section.storage === "engine") {
            resetEngineSection(section.id);
        }
    }
}
