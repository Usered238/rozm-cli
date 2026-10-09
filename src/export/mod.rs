//! Export boundary for WAV now and other formats only when requested.

use std::io::{self, Write};

use crate::engine::PcmAudio;

pub mod wav;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Wav,
}

/// Encoders write PCM to a destination supplied by the file-handling layer.
pub trait AudioExporter {
    fn format(&self) -> ExportFormat;
    fn export(&self, audio: &PcmAudio, output: &mut dyn Write) -> io::Result<()>;
}
