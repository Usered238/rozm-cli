use rozm_cli::{
    engine,
    export::{AudioExporter, wav::WavExporter},
    resources::ResourceLocation,
};
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
    let Some(mut request) = rozm_cli::cli::parse(arguments.iter().cloned())? else {
        println!(
            "rozm-cli (--text TEXT | --stdin) [--output PATH|-] [--voice 1..3] [--speed 1..9]\n\nInput: --stdin reads UTF-8 text until EOF (32768 bytes maximum).\nOutput: WAV only; '-' streams binary WAV to stdout. Default ./out.wav, overwritten without confirmation.\nDefaults: voice 1, speed 5.\nResources: ROZM_DATA_DIR or data/ beside the executable."
        );
        return Ok(());
    };
    let stdout = request.output.as_os_str() == "-";
    if !stdout {
        rozm_cli::export::ExportFormat::from_path(&request.output)?;
    }
    if request.stdin {
        use std::io::Read;
        let mut bytes = Vec::new();
        std::io::stdin()
            .lock()
            .take(32769)
            .read_to_end(&mut bytes)?;
        if bytes.len() > 32768 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "text exceeds 32768 UTF-8 byte limit",
            ));
        }
        request.text = String::from_utf8(bytes).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "stdin is not valid UTF-8")
        })?;
    }
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
    if stdout {
        use std::io::Write;
        let mut output = std::io::stdout().lock();
        WavExporter.export(&audio, &mut output)?;
        output.flush()
    } else {
        rozm_cli::export::write_atomic(&WavExporter, &audio, &request.output)
    }
}
