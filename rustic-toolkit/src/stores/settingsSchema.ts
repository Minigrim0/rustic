import type { FormKitSchemaNode } from "@formkit/core";

export interface SettingsSection {
    id: string;
    label: string;
    /** SVG path for the section icon (24x24 viewBox, stroke). */
    icon: string;
    schema: FormKitSchemaNode[];
}

/** Default values per section. Keys match section `id`. */
export const defaults: Record<string, Record<string, unknown>> = {
    spectrum: {
        minPeakDistance: 50,
        topCount: 10,
        numHarmonics: 8,
    },
    spectrogram: {
        colorScheme: "heat",
        enhancedContrast: true,
        freqScale: "linear",
    },
};

export const sections: SettingsSection[] = [
    {
        id: "spectrum",
        label: "Frequency Spectrum",
        // waveform icon
        icon: "M3 12h2l3-8 4 16 4-12 3 6h2",
        schema: [
            {
                $formkit: "number",
                name: "minPeakDistance",
                label: "Min peak distance (Hz)",
                help: "Minimum Hz separation between detected peaks.",
                min: 1,
                max: 1000,
                step: 1,
            },
            {
                $formkit: "number",
                name: "topCount",
                label: "Top peaks shown",
                help: "Number of top frequency peaks to display.",
                min: 1,
                max: 50,
                step: 1,
            },
            {
                $formkit: "number",
                name: "numHarmonics",
                label: "Harmonic overtones",
                help: "Number of harmonic overtones to overlay on the chart.",
                min: 0,
                max: 32,
                step: 1,
            },
        ],
    },
    {
        id: "spectrogram",
        label: "Spectrogram",
        // grid/squares icon
        icon: "M3.75 6A2.25 2.25 0 0 1 6 3.75h2.25A2.25 2.25 0 0 1 10.5 6v2.25a2.25 2.25 0 0 1-2.25 2.25H6a2.25 2.25 0 0 1-2.25-2.25V6ZM3.75 15.75A2.25 2.25 0 0 1 6 13.5h2.25a2.25 2.25 0 0 1 2.25 2.25V18a2.25 2.25 0 0 1-2.25 2.25H6A2.25 2.25 0 0 1 3.75 18v-2.25ZM13.5 6a2.25 2.25 0 0 1 2.25-2.25H18A2.25 2.25 0 0 1 20.25 6v2.25A2.25 2.25 0 0 1 18 10.5h-2.25a2.25 2.25 0 0 1-2.25-2.25V6ZM13.5 15.75a2.25 2.25 0 0 1 2.25-2.25H18a2.25 2.25 0 0 1 2.25 2.25V18A2.25 2.25 0 0 1 18 20.25h-2.25A2.25 2.25 0 0 1 13.5 18v-2.25Z",
        schema: [
            {
                $formkit: "select",
                name: "colorScheme",
                label: "Color scheme",
                options: {
                    heat: "Heat",
                    plasma: "Plasma",
                    viridis: "Viridis",
                    grayscale: "Grayscale",
                },
            },
            {
                $formkit: "checkbox",
                name: "enhancedContrast",
                label: "Enhanced contrast",
                help: "Apply gamma correction for better visibility on quiet signals.",
            },
            {
                $formkit: "select",
                name: "freqScale",
                label: "Frequency scale",
                options: {
                    linear: "Linear",
                    log: "Logarithmic",
                },
            },
        ],
    },
];