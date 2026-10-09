use std::process::ExitCode;

fn main() -> ExitCode {
    // Read OS strings without panicking on non-Unicode process arguments.
    let _arguments: Vec<_> = std::env::args_os().skip(1).collect();
    eprintln!("rozm-cli: project scaffold; text-to-WAV synthesis is not implemented yet");
    ExitCode::FAILURE
}
