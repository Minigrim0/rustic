// Audio.js - Helper functions for audio processing in the browser

class AudioProcessor {
    constructor() {
        this.audioContext = null;
        this.analyser = null;
        this.source = null;
        this.isInitialized = false;
        this.audioBuffer = null;
    }

    // Initialize the audio context
    async initialize() {
        try {
            this.audioContext = new (window.AudioContext || window.webkitAudioContext)();
            this.analyser = this.audioContext.createAnalyser();
            this.analyser.fftSize = 2048;
            this.isInitialized = true;
            console.log("Audio context initialized");
            return true;
        } catch (error) {
            console.error("Failed to initialize audio context:", error);
            return false;
        }
    }

    // Load an audio file from the given File object
    async loadAudioFile(file) {
        if (!this.isInitialized) {
            await this.initialize();
        }

        try {
            const arrayBuffer = await file.arrayBuffer();
            const audioBuffer = await this.audioContext.decodeAudioData(arrayBuffer);
            this.audioBuffer = audioBuffer;
            console.log("Audio file loaded", {
                duration: audioBuffer.duration,
                numberOfChannels: audioBuffer.numberOfChannels,
                sampleRate: audioBuffer.sampleRate
            });
            return {
                duration: audioBuffer.duration,
                numberOfChannels: audioBuffer.numberOfChannels,
                sampleRate: audioBuffer.sampleRate
            };
        } catch (error) {
            console.error("Failed to load audio file:", error);
            throw new Error(`Failed to load audio file: ${error.message}`);
        }
    }

    // Get the samples from the audio buffer
    getSamples() {
        if (!this.audioBuffer) {
            return null;
        }

        // Get the first channel (mono)
        const samples = this.audioBuffer.getChannelData(0);
        return Array.from(samples);
    }

    // Get the sample rate
    getSampleRate() {
        return this.audioBuffer ? this.audioBuffer.sampleRate : 0;
    }

    // Perform FFT on the loaded audio and return frequency data
    getFrequencyData() {
        if (!this.audioBuffer) {
            return null;
        }

        // Create a source node from the buffer
        const source = this.audioContext.createBufferSource();
        source.buffer = this.audioBuffer;

        // Create an analyzer
        const analyser = this.audioContext.createAnalyser();
        analyser.fftSize = 2048;

        // Connect source to analyzer
        source.connect(analyser);

        // Create a typed array to hold the frequency data
        const frequencyData = new Uint8Array(analyser.frequencyBinCount);
        
        // Get the frequency data
        analyser.getByteFrequencyData(frequencyData);

        // Convert to regular array and normalize to 0-1
        return Array.from(frequencyData).map(val => val / 255);
    }

    // Play the loaded audio
    play() {
        if (!this.audioBuffer) {
            console.error("No audio loaded");
            return false;
        }

        // Create a source node from the buffer
        const source = this.audioContext.createBufferSource();
        source.buffer = this.audioBuffer;
        
        // Connect to destination
        source.connect(this.audioContext.destination);
        
        // Start playing
        source.start();
        this.source = source;
        
        return true;
    }

    // Stop playback
    stop() {
        if (this.source) {
            try {
                this.source.stop();
            } catch (e) {
                console.log("Source already stopped");
            }
        }
    }

    // Clean up resources
    cleanup() {
        this.stop();
        if (this.audioContext) {
            this.audioContext.close();
        }
        this.audioContext = null;
        this.analyser = null;
        this.source = null;
        this.audioBuffer = null;
        this.isInitialized = false;
    }
}

// Export the AudioProcessor class
window.AudioProcessor = AudioProcessor;