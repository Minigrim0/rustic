/**
 * Audio utilities for the Rustic Analyser
 * 
 * This file contains helper functions for audio processing and analysis
 */

/**
 * Normalizes audio samples to a range of -1 to 1
 * 
 * @param {Array<number>} samples - Raw audio samples
 * @returns {Array<number>} Normalized samples
 */
export function normalizeSamples(samples) {
  if (!samples || samples.length === 0) return [];
  
  // Find max amplitude
  let maxAmplitude = 0;
  for (const sample of samples) {
    const absValue = Math.abs(sample);
    if (absValue > maxAmplitude) {
      maxAmplitude = absValue;
    }
  }
  
  // Normalize if needed
  if (maxAmplitude > 0) {
    return samples.map(sample => sample / maxAmplitude);
  }
  
  return samples;
}

/**
 * Calculates the RMS (Root Mean Square) level of audio samples
 * 
 * @param {Array<number>} samples - Audio samples
 * @returns {number} RMS level
 */
export function calculateRmsLevel(samples) {
  if (!samples || samples.length === 0) return 0;
  
  const sumOfSquares = samples.reduce((sum, sample) => sum + sample * sample, 0);
  return Math.sqrt(sumOfSquares / samples.length);
}

/**
 * Finds the peak frequency from a frequency analysis result
 * 
 * @param {Array<[number, number]>} frequencies - Array of [frequency, magnitude] pairs
 * @returns {number} Peak frequency in Hz
 */
export function findPeakFrequency(frequencies) {
  if (!frequencies || frequencies.length === 0) return 0;
  
  let peakFreq = 0;
  let peakMag = 0;
  
  for (const [freq, mag] of frequencies) {
    if (mag > peakMag) {
      peakMag = mag;
      peakFreq = freq;
    }
  }
  
  return peakFreq;
}

/**
 * Converts a frequency to a musical note
 * 
 * @param {number} frequency - Frequency in Hz
 * @returns {string} Musical note name with octave
 */
export function frequencyToNote(frequency) {
  if (!frequency || frequency <= 0) return '';
  
  const noteNames = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];
  const a4 = 440.0; // A4 frequency in Hz
  const a4Index = 69; // MIDI note number for A4
  
  // Calculate the number of half steps away from A4
  const halfSteps = Math.round(12 * Math.log2(frequency / a4));
  
  // Calculate the MIDI note number
  const midiNote = a4Index + halfSteps;
  
  if (midiNote < 0 || midiNote > 127) return 'Out of range';
  
  // Get the note name and octave
  const noteName = noteNames[midiNote % 12];
  const octave = Math.floor(midiNote / 12) - 1;
  
  return `${noteName}${octave}`;
}

/**
 * Processes an ArrayBuffer containing audio file data
 * Helper function for handling file uploads
 * 
 * @param {ArrayBuffer} buffer - ArrayBuffer containing audio file data
 * @returns {Uint8Array} Byte array of the file data
 */
export function processArrayBuffer(buffer) {
  return new Uint8Array(buffer);
}

/**
 * Debounces a function to limit how often it can be called
 * Useful for processing intensive operations
 * 
 * @param {Function} fn - Function to debounce
 * @param {number} delay - Delay in milliseconds
 * @returns {Function} Debounced function
 */
export function debounce(fn, delay) {
  let timeout;
  
  return function(...args) {
    clearTimeout(timeout);
    timeout = setTimeout(() => fn.apply(this, args), delay);
  };
}