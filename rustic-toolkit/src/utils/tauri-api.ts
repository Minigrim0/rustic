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
    GraphMetadata
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
