import type { Readable } from 'node:stream';

export interface AudioStreamOptions {
  /** Defaults to 1. Original voices 2 and 3 do not change with speed. */
  voice?: 1 | 2 | 3;
  /** Integer 1..9; defaults to 5. */
  speed?: number;
  /** WAV is the only implemented exporter. */
  format?: 'wav';
  /** Override the bundled native executable; relative paths use process.cwd(). */
  binaryPath?: string;
  /** Override speech data. Otherwise use ROZM_DATA_DIR or data beside the binary. */
  dataDir?: string;
  /** Aborting destroys the stream with AbortError and stops synthesis. */
  signal?: AbortSignal;
}

/**
 * Return binary WAV chunks (PCM mono, 11025 Hz, unsigned 8-bit).
 * Argument errors throw synchronously. Native failures emit 'error' on the stream.
 * Consume the stream with pipeline/async iteration, or destroy it to cancel.
 * PCM synthesis completes in Rust before the first WAV byte is emitted.
 */
export function createAudioStream(text: string, options?: AudioStreamOptions): Readable;

export interface SynthesisError extends Error {
  code: 'ROZM_EXIT';
  exitCode: number | null;
  signal: string | null;
  stderr: string;
}
