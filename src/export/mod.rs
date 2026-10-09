//! Export boundary for WAV now and other formats only when requested.

use std::io::{self, Write};

use crate::engine::PcmAudio;

pub mod wav;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Wav,
}

impl ExportFormat {
    pub fn from_path(path: &std::path::Path) -> io::Result<Self> {
        if path
            .extension()
            .and_then(|s| s.to_str())
            .is_some_and(|s| s.eq_ignore_ascii_case("wav"))
        {
            Ok(Self::Wav)
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "only WAV output is available; use a .wav extension",
            ))
        }
    }
}

/// Encoders write PCM to a destination supplied by the file-handling layer.
pub trait AudioExporter {
    fn format(&self) -> ExportFormat;
    fn export(&self, audio: &PcmAudio, output: &mut dyn Write) -> io::Result<()>;
}

pub fn write_atomic(
    exporter: &dyn AudioExporter,
    audio: &PcmAudio,
    path: &std::path::Path,
) -> io::Result<()> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| std::path::Path::new("."));
    let (temporary, mut file) = loop {
        let temporary = parent.join(format!(
            ".rozm-{}-{}.tmp",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
        {
            Ok(file) => break (temporary, file),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    };
    let result = (|| {
        exporter.export(audio, &mut file)?;
        file.flush()?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(&temporary, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result
}
