import { reactive, watch } from "vue";

const STORAGE_KEY = "rustic:spectrogram-settings";

export type ColorScheme = "heat" | "plasma" | "viridis" | "grayscale";
export type FreqScale = "linear" | "log";

export interface SpectrogramSettings {
    colorScheme: ColorScheme;
    /** Apply gamma correction (0.3) for better contrast on quiet signals. */
    enhancedContrast: boolean;
    freqScale: FreqScale;
}

const DEFAULTS: SpectrogramSettings = {
    colorScheme: "heat",
    enhancedContrast: true,
    freqScale: "linear",
};

function load(): SpectrogramSettings {
    try {
        const raw = localStorage.getItem(STORAGE_KEY);
        if (raw) return { ...DEFAULTS, ...JSON.parse(raw) };
    } catch { /* ignore corrupt data */ }
    return { ...DEFAULTS };
}

/** Mutable settings — bind inputs to this. */
export const spectrogramSettings = reactive<SpectrogramSettings>(load());

// Persist on change
watch(spectrogramSettings, (val) => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(val));
});