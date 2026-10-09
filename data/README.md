# Speech data

The native CLI reads the following files from `data/` beside the executable,
or from `ROZM_DATA_DIR` when set. The package scripts copy them from `orig/`.

```text
BSNbn
skfs/skf
skfs/wif
skfs/wlf
snf1/ip2f   snf1/lp2f   snf1/Sd2f
snf2/ip2f   snf2/lp2f   snf2/Sd2f
snf3/ip2f   snf3/lp2f   snf3/Sd2f
```

BSNbn contains compressed Ukrainian words and stress markers. The skfs files
contain corrections. Each voice has 42×42 little-endian i32 offsets/lengths
(7056 bytes per table) and a sample bank. Voice 1 embeds speed descriptors;
voices 2/3 store PCM directly. Loader checks sizes, counts, record boundaries
and sample ranges before synthesis. Banks end one byte before their last
silence transition; the original zero-initialized trailing byte is reproduced.

Keep the names and case, especially `Sd2f` on Linux. Files are read-only and
local copies are ignored by Git. Use the same data version as the tested
original (see docs/original-engine.md). Rozm.exe, LAME, wavhead.wav and skfs
backup files are not runtime dependencies. English EUtDic.txt is absent from
this source bundle; the CLI uses Latin fallback transliteration.
