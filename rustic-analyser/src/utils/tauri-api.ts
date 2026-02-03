/**
 * Tauri API utilities for the Rustic Analyser
 * 
 * This file contains functions for interacting with the Tauri backend
 */
import { invoke } from '@tauri-apps/api/core';

/**
 * Processes an audio file and returns its data
 * 
 * @param {string} path - Path to the audio file
 * @returns {Promise<Object>} Audio data with samples and metadata
 */
export async function analyzeAudioFile(path: string) {
    try {
        // Call the Tauri backend to analyze the audio file
        const result: { sample_rate: number, duration: number } = await invoke('analyze_audio_file', { path });

        // Get samples and sample rate from backend
        const samples = await invoke('get_samples');

        return {
            samples,
            sample_rate: result.sample_rate,
            name: path.split('/').pop(),
            duration: result.duration
        };
    } catch (error: any) {
        console.error('Error analyzing audio file:', error);
        throw new Error(`Failed to analyze audio: ${error.message || error}`);
    }
}

/**
 * Gets the frequency analysis data from the backend
 * 
 * @returns {Promise<Array>} Array of frequency-magnitude pairs
 */
export async function getFrequencyData(): Promise<Array<[number, number]>> {
    try {
        const frequencies: Array<{ frequency: number, magnitude: number }> = await invoke('compute_fft_command');

        // Convert to [frequency, magnitude] pairs for the Vue component
        return frequencies.map(f => [f.frequency, f.magnitude]);
    } catch (error: any) {
        console.error('Error computing FFT:', error);
        throw new Error(`Failed to compute FFT: ${error.message || error}`);
    }
}

/**
 * Gets the spectrogram data from the backend
 * 
 * @returns {Promise<Array<Array<number>>>} 2D array of spectrogram data
 */
export async function getSpectrogramData(): Promise<Array<Array<number>>> {
    try {
        return await invoke('compute_spectrum_command');
    } catch (error: any) {
        console.error('Error computing spectrum:', error);
        throw new Error(`Failed to compute spectrum: ${error.message || error}`);
    }
}

/**
 * Estimates the pitch of the currently loaded audio
 * 
 * @returns {Promise<number|null>} Estimated pitch in Hz, or null if not detected
 */
export async function estimatePitch(): Promise<number|null> {
    try {
        return await invoke('estimate_pitch_command');
    } catch (error: any) {
        console.error('Error estimating pitch:', error);
        throw new Error(`Failed to estimate pitch: ${error.message || error}`);
    }
}

/**
 * Converts a frequency to a musical note
 * 
 * @param {number} frequency - Frequency in Hz
 * @returns {Promise<string>} Musical note representation
 */
export async function frequencyToNote(frequency: number) {
    try {
        return await invoke('frequency_to_note_command', { frequency });
    } catch (error: any) {
        console.error('Error converting frequency to note:', error);
        throw new Error(`Failed to convert frequency: ${error.message || error}`);
    }
}

/**
 * Writes file data to a temporary file
 * 
 * @param {string} filename - Name of the file
 * @param {Uint8Array} fileData - File data as a byte array
 * @returns {Promise<string>} Path to the temporary file
 */
export async function writeTempFile(filename: string, fileData: Uint8Array) {
    const tempPath = `/tmp/${filename}`;

    try {
        await invoke('plugin:fs|write_file', {
            path: tempPath,
            contents: fileData
        });

        return tempPath;
    } catch (error: any) {
        console.error('Error writing temp file:', error);
        throw new Error(`Failed to save file: ${error.message || error}`);
    }
}

/**
 * Performs complete audio analysis and returns all results
 * 
 * @param {Array<number>} audioData - Audio data with samples and sample rate
 * @returns {Promise<Object>} Complete analysis results
 */
export async function performCompleteAnalysis(audioData: {samples: Array<number>}) {
    try {
        // Get frequency data
        const frequencies = await getFrequencyData();

        // Get spectrogram
        const spectrogram = await getSpectrogramData();

        // Get pitch estimation
        const pitch = await estimatePitch();

        // Get note for the pitch if available
        const note = pitch ? await frequencyToNote(pitch) : null;

        // Find peak frequency
        const peak_frequency = frequencies.reduce(
            (max, [freq, mag]) => (mag > max.mag ? { freq, mag } : max),
            { freq: 0, mag: 0 }
        ).freq;

        // Calculate RMS level
        const rms_level = Math.sqrt(
            audioData.samples.reduce((sum, s) => sum + s * s, 0) / audioData.samples.length
        );

        return {
            frequencies,
            spectrogram,
            peak_frequency,
            pitch,
            note,
            rms_level
        };
    } catch (error: any) {
        console.error('Error performing complete analysis:', error);
        throw new Error(`Analysis failed: ${error.message || error}`);
    }
}