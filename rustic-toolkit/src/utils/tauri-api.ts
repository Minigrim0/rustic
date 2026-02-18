/**
 * Tauri IPC layer for the Rustic Toolkit.
 *
 * All functions map 1-to-1 to backend commands.
 * Types are auto-generated from Rust via ts-rs.
 */
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import type {
    AudioSummary,
    FrequencyData,
    WaveformData,
    SpectrumData,
    SpectrogramData,
    GraphMetadata,
    EngineConfig,
} from "@/types";

/** Load an audio file and return a global summary. */
export async function analyzeAudioFile(path: string): Promise<AudioSummary> {
    return invoke<AudioSummary>("analyze_audio_file", { path });
}

/** Get waveform samples for a time window, downsampled for display. */
export async function getWaveform(
    start: number,
    end: number,
    targetPoints: number,
): Promise<WaveformData> {
    return invoke<WaveformData>("get_waveform", {
        start,
        end,
        targetPoints,
    });
}

/** Get FFT frequency data for a time window. */
export async function getSpectrum(
    start: number,
    end: number,
    topCount: number,
    minPeakDistance: number,
): Promise<SpectrumData> {
    return invoke<SpectrumData>("get_spectrum", { start, end, topCount, minPeakDistance });
}

/** Get top frequency peaks within a frequency range (peak-picked in Rust). */
export async function getTopFrequencies(
    start: number,
    end: number,
    freqLo: number,
    freqHi: number,
    topCount: number,
    minPeakDistance: number,
): Promise<FrequencyData[]> {
    return invoke<FrequencyData[]>("get_top_frequencies", {
        start,
        end,
        freqLo,
        freqHi,
        topCount,
        minPeakDistance,
    });
}

/** Get spectrogram (STFT) data for a time window. */
export async function getSpectrogram(
    start: number,
    end: number,
): Promise<SpectrogramData> {
    return invoke<SpectrogramData>("get_spectrogram", { start, end });
}

export async function getGraphMetadata(): Promise<GraphMetadata> {
    return invoke<GraphMetadata>("get_graph_metadata");
}

/** Convert a frequency (Hz) to the nearest musical note name. */
export async function frequencyToNote(frequency: number): Promise<string> {
    return invoke<string>("frequency_to_note_command", { frequency });
}

/** Save an AudioSummary as JSON to the given path. */
export async function saveAnalysis(
    path: string,
    summary: AudioSummary,
): Promise<void> {
    return invoke<void>("save_analysis", { path, summary });
}

/** Listen for a native menu event by item ID. Returns an unlisten function. */
export function onMenuEvent(
    menuId: string,
    handler: () => void,
): Promise<UnlistenFn> {
    return listen(menuId, handler);
}

export async function setRenderMode(
    render_mode: "graph" | "instrument"
): Promise<void> {
    return invoke<void>("change_render_mode", {renderMode: render_mode});
}

// ── Graph commands ──────────────────────────────────────────────────

/** Add a node to the backend audio graph. Returns the assigned backend ID. */
export async function graphAddNode(
    nodeType: string,
    kind: "Generator" | "Filter" | "Sink",
    position: [number, number],
): Promise<number> {
    return invoke<number>("graph_add_node", { nodeType, kind, position });
}

/** Remove a node from the backend audio graph. */
export async function graphRemoveNode(id: number): Promise<void> {
    return invoke<void>("graph_remove_node", { id });
}

/** Connect two nodes in the backend audio graph. */
export async function graphConnect(
    from: number,
    fromPort: number,
    to: number,
    toPort: number,
): Promise<void> {
    return invoke<void>("graph_connect", { from, fromPort, to, toPort });
}

/** Disconnect two nodes in the backend audio graph. */
export async function graphDisconnect(from: number, to: number): Promise<void> {
    return invoke<void>("graph_disconnect", { from, to });
}

/** Start a specific generator node in the audio graph. */
export async function graphStartNode(id: number): Promise<void> {
    return invoke<void>("graph_start_node", { id });
}

/** Stop a specific generator node in the audio graph. */
export async function graphStopNode(id: number): Promise<void> {
    return invoke<void>("graph_stop_node", { id });
}

/** Set a parameter on a node in the backend audio graph. */
export async function graphSetParameter(
    nodeId: number,
    paramName: string,
    value: number,
): Promise<void> {
    return invoke<void>("graph_set_parameter", { nodeId, paramName, value });
}

/** Read the engine configuration from ~/.config/rustic/config.toml (or defaults). */
export async function getEngineConfig(): Promise<EngineConfig> {
    return invoke<EngineConfig>("get_engine_config");
}

/** Write the engine configuration back to ~/.config/rustic/config.toml. */
export async function setEngineConfig(config: EngineConfig): Promise<void> {
    return invoke<void>("set_engine_config", { config });
}
