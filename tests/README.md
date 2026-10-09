# CLI process tests

The agreed test seam is **only the CLI process**. No library or private-function
tests are used. `tests/cli.rs` launches the real Cargo-built executable with
direct argv, isolated working directories and a 30-second timeout per launch.
Put the original binary resource files under `orig/`; no original EXE or Python
is needed to run this Rust test suite.

Coverage includes defaults, exact WAV bytes, Ukrainian dictionary/context/manual
stress, softened consonants, Latin fallback, numbers, symbols, initials, clauses,
three voices, extreme speeds, directives, quotes/apostrophes, LF/CRLF/CR versus
literal backslash-n, shell metacharacters, Unicode paths, overwrite, unsupported
characters/options/formats, text/audio/resource size limits, missing/damaged data,
data beside the executable, output errors and temporary-file cleanup.
Unix checks an invalid UTF-8 argv value; Windows checks an unpaired UTF-16
surrogate through the real process. NUL cannot be delivered through argv,
so its exploratory JSON case is excluded from executable tests.

`original-*.wav` fixtures come from the **original x86 machine-code procedures**
in Rozm.exe, run under a development-only Unicorn harness with actual original
resources. They are independent of the Rust implementation. Helpers emulate
Delphi string operations; text/stress/phonetic/synthesis procedures execute
original instructions. `tools/regenerate-fixtures.py` reproduces these files.
The greeting uses explicit original clauses; Latin uses original fallback
0x465CF8 because this bundle has no English dictionary. This proves equality
for the covered inputs, not every possible sentence or a full GUI export.

```sh
cargo test --offline --test cli
# Windows x86:
cargo test --offline --target i686-pc-windows-msvc --test cli
```

See docs/validation.md for the actual platform runs and RED → GREEN evidence.
