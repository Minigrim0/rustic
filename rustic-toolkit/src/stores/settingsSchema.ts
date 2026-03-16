import type { FormKitSchemaNode } from "@formkit/core";

export interface SettingsSection {
    id: string;
    label: string;
    /** SVG path for the section icon (24x24 viewBox, stroke). */
    icon: string;
    schema: FormKitSchemaNode[];
    /** Where the setting is persisted. "local" = localStorage, "engine" = config.toml */
    storage?: "local" | "engine";
    /** For engine settings: the top-level key in EngineConfig (e.g. "system", "audio", "logging"). */
    engineKey?: string;
    /** If true, changes require an app restart to take effect. */
    requiresRestart?: boolean;
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
    toolkitLogging: {
        terminalLogLevel: "info",
        fileLogLevel: "trace",
    },
    system: {
        sample_rate: 44100,
        master_volume: 1.0,
    },
    audio: {
        cpal_buffer_size: 64,
        render_chunk_size: 256,
        audio_ring_buffer_size: 88200,
        message_ring_buffer_size: 1024,
        target_latency_ms: 50.0,
    },
    logging: {
        level: "info",
        log_to_file: false,
        log_file: "rustic.log",
        log_to_stdout: true,
    },
};

export const sections: SettingsSection[] = [
    {
        id: "spectrum",
        label: "Frequency Spectrum",
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
    {
        id: "toolkitLogging",
        label: "Toolkit Logging",
        storage: "local",
        // terminal icon
        icon: "M6.75 7.5l3 2.25-3 2.25m4.5 0h3m-9 8.25h13.5A2.25 2.25 0 0 0 21 18V6a2.25 2.25 0 0 0-2.25-2.25H5.25A2.25 2.25 0 0 0 3 6v12a2.25 2.25 0 0 0 2.25 2.25Z",
        schema: [
            {
                $formkit: "select",
                name: "terminalLogLevel",
                label: "Terminal log level",
                help: "Minimum severity shown in the terminal.",
                options: {
                    trace: "Trace",
                    debug: "Debug",
                    info: "Info",
                    warn: "Warn",
                    error: "Error",
                },
            },
            {
                $formkit: "select",
                name: "fileLogLevel",
                label: "File log level",
                help: "Minimum severity written to the log file.",
                options: {
                    trace: "Trace",
                    debug: "Debug",
                    info: "Info",
                    warn: "Warn",
                    error: "Error",
                },
            },
        ],
    },
    {
        id: "system",
        label: "System",
        storage: "engine",
        engineKey: "system",
        // cpu/chip icon
        icon: "M8.25 3v1.5M4.5 8.25H3m18 0h-1.5M4.5 12H3m18 0h-1.5M4.5 15.75H3m18 0h-1.5M8.25 19.5V21M12 3v1.5m0 15V21m3.75-18v1.5m0 15V21m-9-1.5h10.5a2.25 2.25 0 0 0 2.25-2.25V6.75a2.25 2.25 0 0 0-2.25-2.25H6.75A2.25 2.25 0 0 0 4.5 6.75v10.5a2.25 2.25 0 0 0 2.25 2.25Z",
        schema: [
            {
                $formkit: "number",
                name: "sample_rate",
                label: "Sample rate (Hz)",
                help: "Audio sample rate. Requires restart.",
                min: 8000,
                max: 192000,
                step: 100,
            },
            {
                $formkit: "number",
                name: "master_volume",
                label: "Master volume",
                help: "Global output volume (0.0 – 1.0).",
                min: 0,
                max: 1,
                step: 0.01,
            },
        ],
        requiresRestart: true,
    },
    {
        id: "audio",
        label: "Audio Engine",
        storage: "engine",
        engineKey: "audio",
        requiresRestart: true,
        // speaker-wave icon
        icon: "M19.114 5.636a9 9 0 0 1 0 12.728M16.463 8.288a5.25 5.25 0 0 1 0 7.424M6.75 8.25l4.72-4.72a.75.75 0 0 1 1.28.53v15.88a.75.75 0 0 1-1.28.53l-4.72-4.72H4.51c-.88 0-1.704-.507-1.938-1.354A9.009 9.009 0 0 1 2.25 12c0-.83.112-1.633.322-2.396C2.806 8.756 3.63 8.25 4.51 8.25H6.75Z",
        schema: [
            {
                $formkit: "number",
                name: "cpal_buffer_size",
                label: "CPAL buffer size",
                help: "Buffer size in samples (lower = less latency, more CPU).",
                min: 16,
                max: 2048,
                step: 16,
            },
            {
                $formkit: "number",
                name: "render_chunk_size",
                label: "Render chunk size",
                help: "Audio render chunk size in samples.",
                min: 64,
                max: 4096,
                step: 64,
            },
            {
                $formkit: "number",
                name: "audio_ring_buffer_size",
                label: "Ring buffer size",
                help: "Audio ring buffer size in samples.",
                min: 4096,
                max: 262144,
                step: 1024,
            },
            {
                $formkit: "number",
                name: "message_ring_buffer_size",
                label: "Message buffer size",
                help: "Message ring buffer capacity.",
                min: 64,
                max: 8192,
                step: 64,
            },
            {
                $formkit: "number",
                name: "target_latency_ms",
                label: "Target latency (ms)",
                help: "Target maximum audio latency in milliseconds.",
                min: 1,
                max: 500,
                step: 1,
            },
        ],
    },
    {
        id: "logging",
        label: "Engine Logging",
        storage: "engine",
        engineKey: "logging",
        // document-text icon
        icon: "M19.5 14.25v-2.625a3.375 3.375 0 0 0-3.375-3.375h-1.5A1.125 1.125 0 0 1 13.5 7.125v-1.5a3.375 3.375 0 0 0-3.375-3.375H8.25m0 12.75h7.5m-7.5 3H12M10.5 2.25H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 0 0-9-9Z",
        schema: [
            {
                $formkit: "select",
                name: "level",
                label: "Log level",
                help: "Minimum log severity for the engine.",
                options: {
                    trace: "Trace",
                    debug: "Debug",
                    info: "Info",
                    warn: "Warn",
                    error: "Error",
                },
            },
            {
                $formkit: "checkbox",
                name: "log_to_file",
                label: "Log to file",
                help: "Write engine logs to a file.",
            },
            {
                $formkit: "text",
                name: "log_file",
                label: "Log file name",
                help: "File name for engine logs (relative to config directory).",
            },
            {
                $formkit: "checkbox",
                name: "log_to_stdout",
                label: "Log to stdout",
                help: "Print engine logs to the terminal.",
            },
        ],
    },
];
