# Rozm CLI — native text → WAV

Keep this executable and its data/ directory together. No Python, Wine,
Rozm.exe, audio device or desktop is required. Linux package: Ubuntu 24.04
x86_64. Windows package: Win32 x86 on modern Windows.

Linux / Bash:

```bash
./rozm-cli --text 'Привіт, світе!'
./rozm-cli --text '123' --output result.wav --voice 1 --speed 7
./rozm-cli --text $'"Привіт"\nсвіте'
```

Windows / PowerShell:

```powershell
.\rozm-cli.exe --text 'Привіт, світе!'
.\rozm-cli.exe --text "Привіт`nсвіте" --output result.wav --voice 1 --speed 7
```

Required: --text (one argument) or --stdin (UTF-8 until EOF), mutually exclusive.
Defaults: --output ./out.wav, --voice 1,
--speed 5. Voice range: 1..3; speed range: 1..9. Use --help for help.
One call writes one mono 11025 Hz unsigned 8-bit PCM WAV. Existing output is
replaced without confirmation. Only .wav output is supported; MP3 is deferred.
Use --output - to export binary WAV to stdout without creating a file. Native
synthesis completes in memory before WAV export starts. The separate rozm-tts
npm package provides createAudioStream(text, options) for Node.js 22+.
Errors return exit code 1 with stderr diagnostics, retaining previous output.
The destination directory must already exist.

The original Ukrainian engine's pronunciation is retained: apostrophe or
backslash after a vowel sets stress; #1/#2/#3 change voice. Speed changes only
voice 1 (Anatol). Voices 2/3 retain their original speed. ґ/Ґ are omitted by
original normalization. Latin words use fallback transliteration; no English
dictionary was available. Emoji and combining accents produce explicit errors.
Real line breaks split clauses; literal backslash-n is not an escape sequence.

Resource lookup: ROZM_DATA_DIR when set, otherwise data/ beside the executable.
Names are case-sensitive on Linux. Limits: 32768 UTF-8 text bytes, 32 MiB per
resource file, 64 MiB PCM; the OS may impose a smaller argv limit.
