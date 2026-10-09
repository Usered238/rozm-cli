# Original Rozm investigation

Findings from static inspection, not a dynamically verified engine:

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

Addresses belong to this binary, not to a Linux process. Reconstruct portable
data formats and algorithms instead of relying on Delphi memory layout. Full
formats, text limits, encoding and normal activation behavior remain unverified.
