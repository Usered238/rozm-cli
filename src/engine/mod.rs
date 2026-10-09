//! Portable synthesis. Original x86 pointers and Delphi ABI must not leak here.
//! The text-to-PCM algorithm remains to be reconstructed.

#[derive(Debug)]
pub struct PcmAudio {
    pub sample_rate: u32,
    pub channels: u16,
    /// Unsigned 8-bit PCM, matching the original WAV template.
    pub samples: Vec<u8>,
}
