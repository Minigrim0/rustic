import { reactive, readonly, watch, type DeepReadonly } from "vue";

const STORAGE_KEY = "rustic:spectrum-settings";

export interface SpectrumSettings {
    /** Minimum Hz separation between picked peaks. */
    minPeakDistance: number;
    /** Number of top peaks to display. */
    topCount: number;
    /** Number of harmonic overtones to overlay. */
    numHarmonics: number;
}

const DEFAULTS: SpectrumSettings = {
    minPeakDistance: 50,
    topCount: 10,
    numHarmonics: 8,
};

function load(): SpectrumSettings {
    try {
        const raw = localStorage.getItem(STORAGE_KEY);
        if (raw) return { ...DEFAULTS, ...JSON.parse(raw) };
    } catch { /* ignore corrupt data */ }
    return { ...DEFAULTS };
}

/** Mutable settings — bind inputs to this. */
export const spectrumSettings = reactive<SpectrumSettings>(load());

// Persist on change
watch(spectrumSettings, (val) => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(val));
});

/**
 * Debounced snapshot of settings — watch this for triggering expensive operations.
 * Updates 500ms after the user stops editing.
 */
const _applied = reactive<SpectrumSettings>({ ...spectrumSettings });
export const appliedSettings: DeepReadonly<SpectrumSettings> = readonly(_applied);

let debounceTimer: ReturnType<typeof setTimeout> | null = null;

watch(spectrumSettings, (val) => {
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
        Object.assign(_applied, val);
    }, 500);
});