# Rozm CLI

Rust scaffold for native text-to-WAV synthesis using algorithms and, where
needed, binary speech data from the original Rozm application.

TTS, argument parsing and WAV encoding are not implemented yet. The executable
returns an error and creates no audio. See [PLAN.md](PLAN.md) for the CLI contract.

First target: Ubuntu 24.04 x86_64 (`x86_64-unknown-linux-gnu`); then Win32 x86
(`i686-pc-windows-msvc`). No GUI/audio device is required by the planned engine.
MP3 implementation is deferred until explicitly requested.

```sh
cargo check
cargo fmt --check
cargo build --release --target x86_64-unknown-linux-gnu
```

Build Linux on Linux with its toolchain/linker; selecting a target on Windows
does not supply a cross-compilation environment. Win32 requires the Rust target
and Windows x86 linker/SDK.

Planned arguments: required --text, optional --output (./out.wav), --voice (1),
--speed (5). Existing output is overwritten without confirmation. Only WAV is
initially accepted.

Code: src/{text,resources,engine,export}. Fixtures: tests/fixtures. Local originals,
voice data, builds and generated audio are ignored; Cargo.lock is tracked.
