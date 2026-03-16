export interface SystemConfig {
    sample_rate: number;
    master_volume: number;
}

export interface AudioConfig {
    cpal_buffer_size: number;
    render_chunk_size: number;
    audio_ring_buffer_size: number;
    message_ring_buffer_size: number;
    target_latency_ms: number;
}

export interface LogConfig {
    level: string;
    log_to_file: boolean;
    log_file: string;
    log_to_stdout: boolean;
}

export interface EngineConfig {
    system: SystemConfig;
    audio: AudioConfig;
    logging: LogConfig;
}
