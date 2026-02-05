/**
 * Audio utilities for the Rustic Analyser
 *
 * Most analysis (RMS, peak frequency, pitch, note) is now handled by the
 * backend.  This file keeps only client-side helpers.
 */

/**
 * Processes an ArrayBuffer containing audio file data.
 * Helper for handling file uploads before sending to the backend.
 */
export function processArrayBuffer(buffer: ArrayBuffer): Uint8Array {
    return new Uint8Array(buffer);
}

/**
 * Debounces a function to limit how often it can be called.
 * Useful for resize handlers or other high-frequency events.
 */
export function debounce<T extends (...args: unknown[]) => void>(
    fn: T,
    delay: number,
): (...args: Parameters<T>) => void {
    let timeout: ReturnType<typeof setTimeout>;
    return function (this: unknown, ...args: Parameters<T>) {
        clearTimeout(timeout);
        timeout = setTimeout(() => fn.apply(this, args), delay);
    };
}