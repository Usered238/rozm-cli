use rozm_cli::{engine, export::wav::WavExporter, resources::ResourceLocation};
use std::process::ExitCode;

fn main() -> ExitCode {
    // Read OS strings without panicking on non-Unicode process arguments.
    let arguments: Vec<_> = std::env::args_os().skip(1).collect();
    match run(&arguments) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("rozm-cli: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(arguments: &[std::ffi::OsString]) -> std::io::Result<()> {
    let Some(request) = rozm_cli::cli::parse(arguments.iter().cloned())? else {
        println!(
            "rozm-cli --text TEXT [--output PATH] [--voice 1..3] [--speed 1..9]\n\nOutput: WAV only; default ./out.wav, overwritten without confirmation.\nDefaults: voice 1, speed 5.\nResources: ROZM_DATA_DIR or data/ beside the executable."
        );
        return Ok(());
    };
    rozm_cli::export::ExportFormat::from_path(&request.output)?;
    rozm_cli::text::validate(&request.text)?;
    let location = ResourceLocation::discover()?;
    let dictionary = location.load_dictionary()?;
    let mut audio = engine::PcmAudio {
        sample_rate: 11025,
        channels: 1,
        samples: Vec::new(),
    };
    let mut current_voice = request.voice;
    let mut voice = location.load_voice(current_voice)?;
    for clause in rozm_cli::text::clauses(&request.text, request.voice) {
        if clause.voice != current_voice {
            voice = location.load_voice(clause.voice)?;
            current_voice = clause.voice;
        }
        let phonemes = rozm_cli::text::phonemes(&clause.text, &dictionary)?;
        let part = engine::synthesize(&phonemes, &voice, request.speed)?;
        if audio.samples.len().saturating_add(part.samples.len()) > engine::MAX_AUDIO_BYTES {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "audio exceeds 64 MiB size limit",
            ));
        }
        audio.samples.extend_from_slice(&part.samples);
    }
    if audio.samples.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "text produces no speech",
        ));
    }
    rozm_cli::export::write_atomic(&WavExporter, &audio, &request.output)
}
