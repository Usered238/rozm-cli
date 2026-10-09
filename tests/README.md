# Planned checks

The JSON fixtures are inputs, not completed tests. Once argument parsing and
synthesis exist, process integration tests must pass these texts directly as
argv, check exit codes/timeouts and validate the WAV. Also test malformed Unicode
and long input. NUL cannot be passed in argv; its fixture is for the text API.

Run on Ubuntu 24.04 x86_64 first, then Windows x86. Check Bash/PowerShell quoting
separately from direct process invocation, default ./out.wav, overwrite, missing
resources and PCM equivalence to the original.
