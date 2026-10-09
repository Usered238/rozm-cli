# Original Rozm investigation

Findings from static inspection and original procedure execution under a
development-only x86 emulator. Full GUI export was not exercised:

- Native Delphi/VCL x86 PE32, GUI subsystem; entry point 0x46C7A8.
- No user-defined CLI dispatch found; GetCommandLineA is used by Delphi RTL.
- Resource init 0x463EF4..0x464508 mixes data, VCL and waveOut initialization.
  CWD becomes the resource base via 0x463F24 -> 0x402914 -> 0x46FC5C.
- Orchestration 0x4678B4 invokes text/phonetic routines and core 0x466E7C.
  WAV export calls it synchronously.
- WAV flag 0x46E710 bypasses playback. Writer 0x46756A..0x46768A creates a
  44-byte header and appends PCM; offsets 4 and 40 become filelen-8/filelen-44.
- PCM mono, 11025 Hz, unsigned 8-bit samples.
- Defaults: voice byte 0x46E6E8 = '1'; speed dword 0x46E6E4 = 5.
- Voice directives access a VCL label at 0x46796F..0x46797D: preserve their
  semantics when porting, without this GUI behavior.
- Original MP3 export invokes LAME 3.94 MMX beta 1; MP3 is deferred here.

orig/Rozm.txt anchors: WAV handler line 519; path 524; PCM writer 348; LAME 952;
numbered MP3 export 976.

SHA-256 orig/Rozm.exe:
90E26B39A4CA518DF3B1EBCF4BBAF90A8543ED90A31273241D580D22B39E1A54

Addresses belong to this binary, not to a Linux process. The runtime is portable
Rust and reads explicit little-endian fields rather than Delphi process pointers.

## Recovered file path

Text stages: 0x465888 (symbols/initials), 0x4659DC (numbers), 0x465E9C (Latin),
0x46621C (normalization), 0x465210 (dictionary stress), 0x46662C (phonetics),
0x466E7C (PCM). The Latin fallback routine is 0x465CF8. Input is Windows-1251
internally; the Rust CLI accepts Unicode and validates conversion explicitly.

BSNbn records encode letters with a byte alphabet, followed by stress markers
38..80; final standalone 0x21 is the sentinel. skfs/wif starts with a u32 count,
then u32 offsets; skfs/wlf supplies byte lengths; skfs/skf ends each record with
a stress marker. Corrections precede BSNbn lookup.

Phoneme codes are 0xD2..0xFB; 0xFA is silence. Transition index is
`(first-0xD2)*42+(second-0xD2)`. Each ip2f/lp2f table has 1764 i32 entries.
Missing transitions use nonpositive lengths. Sd2f banks end one byte short of
the last silence transition; original BSS supplies a zero byte.

Voice 1 strips a header at normal speed and modifies silence/chunks/periods at
other speeds. Voices 2/3 consume PCM directly; speed has no effect in their
original core path. Voice 2 remaps selected transitions to silence. Adjacent
blocks use a 17-sample crossfade with ties-to-even rounding. Different text
clauses append without crossfade. The old WAV writes an unpadded 44-byte header
plus PCM, even when data length is odd; fixtures and output retain that layout.

See tools/README.md for reproduction and tests/README.md for the scope of the
equivalence checks. No embedded original instructions, emulator or Delphi ABI
are needed by the shipping binary.
