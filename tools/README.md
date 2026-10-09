# Original-procedure oracle (development only)

These tools run the original Rozm PE32 text/synthesis procedures under Unicorn.
They are not shipped with the CLI and are not invoked by Cargo tests. Python,
Unicorn and pefile are optional research dependencies, not runtime dependencies.

`original_oracle.py` maps the original PE and data at original addresses, supplies
Delphi string helpers, and captures unsigned PCM at 0x4674F2 before playback.
No GUI procedure, shell or LAME is executed. Resource names are relative to the
repository's `orig/`. Only the documented original hash is accepted.

```sh
python -m pip install -r tools/requirements.txt
# Verify all checked-in fixtures independently, without overwriting:
python tools/regenerate-fixtures.py
# Explicit regeneration when investigating the original:
python tools/regenerate-fixtures.py --write
```

Keep orig/Rozm.exe, BSNbn, skfs, snf1/2/3 and wavhead.wav available. The generator
compares the original procedures, not the Rust implementation. Latin fixtures
execute original fallback transliteration because EUtDic.txt is missing.
Greeting clauses are explicitly listed; complete VCL orchestration is not emulated.
The harness emulates required RTL helpers, so corpus equivalence is evidence for
the covered inputs rather than a proof for arbitrary original GUI behavior.
