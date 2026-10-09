//! Legacy Rozm mono unsigned 8-bit PCM WAV encoding.

use std::io::{self, Write};

use crate::engine::PcmAudio;

use super::{AudioExporter, ExportFormat};

pub struct WavExporter;

impl AudioExporter for WavExporter {
    fn format(&self) -> ExportFormat {
        ExportFormat::Wav
    }

    fn export(&self, audio: &PcmAudio, output: &mut dyn Write) -> io::Result<()> {
        let length = u32::try_from(audio.samples.len())
            .map_err(|_| io::Error::other("audio exceeds WAV size limit"))?;
        let riff_size = length
            .checked_add(36)
            .ok_or_else(|| io::Error::other("audio exceeds WAV size limit"))?;
        output.write_all(b"RIFF")?;
        output.write_all(&riff_size.to_le_bytes())?;
        output.write_all(b"WAVEfmt ")?;
        output.write_all(&16_u32.to_le_bytes())?;
        output.write_all(&1_u16.to_le_bytes())?;
        output.write_all(&audio.channels.to_le_bytes())?;
        output.write_all(&audio.sample_rate.to_le_bytes())?;
        output.write_all(&(audio.sample_rate * u32::from(audio.channels)).to_le_bytes())?;
        output.write_all(&audio.channels.to_le_bytes())?;
        output.write_all(&8_u16.to_le_bytes())?;
        output.write_all(b"data")?;
        output.write_all(&length.to_le_bytes())?;
        output.write_all(&audio.samples)
    }
}
