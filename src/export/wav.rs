//! WAV scaffold. It never pretends to have written audio successfully.

use std::io::{self, Write};

use crate::engine::PcmAudio;

use super::{AudioExporter, ExportFormat};

pub struct WavExporter;

impl AudioExporter for WavExporter {
    fn format(&self) -> ExportFormat {
        ExportFormat::Wav
    }

    fn export(&self, _audio: &PcmAudio, _output: &mut dyn Write) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "WAV encoding is not implemented in this scaffold yet",
        ))
    }
}
