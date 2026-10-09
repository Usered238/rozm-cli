"""Verify golden WAVs using original Rozm x86 procedures; --write regenerates."""
import argparse
import os
import struct
from pathlib import Path

from original_oracle import Oracle

ROOT = Path(__file__).resolve().parent.parent
os.chdir(ROOT)
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--write", action="store_true")
args = parser.parse_args()

cases = [
    ("original-a-v1-s5.wav", ["а"], 1, 5, False),
    ("original-a-v1-s1.wav", ["а"], 1, 1, False),
    ("original-a-v1-s9.wav", ["а"], 1, 9, False),
    ("original-pryvit-v1-s5.wav", ["Привіт"], 1, 5, False),
    ("original-greeting-v1-s5.wav", ["Привіт,", " світе!"], 1, 5, False),
    ("original-123-v1-s5.wav", ["123"], 1, 5, False),
    ("original-symbols-v1-s5.wav", ["2+3=5"], 1, 5, False),
    ("original-initial-v1-s5.wav", ["А."], 1, 5, False),
    ("original-hello-v1-s5.wav", ["hello"], 1, 5, True),
]
corpus = ["Україна", "п’ять", "для мене", "ніч", "щастя", "дзвін",
          "невідомеслово", r"а\том", "Ґанок"]
cases.extend((f"original-corpus-{i}.wav", [text], 1, 5, False)
             for i, text in enumerate(corpus))
cases.extend((f"original-a-v{voice}-s5.wav", ["а"], voice, 5, False)
             for voice in (2, 3))

oracle = None
current_voice = None
for filename, clauses, voice, speed, latin in cases:
    if voice != current_voice:
        oracle = Oracle()
        oracle.voice(voice)
        current_voice = voice
    oracle.put(0x46E6E4, speed)
    pcm = bytearray()
    for clause in clauses:
        text = clause.replace("’", "'").encode("cp1251")
        if latin:
            text = oracle.run(0x465CF8, text) + b" "
            stages = (0x46621C, 0x465210, 0x46662C)
        else:
            text += b" "
            stages = (0x465888, 0x4659DC, 0x465E9C, 0x46621C, 0x465210, 0x46662C)
        for address in stages:
            text = oracle.run(address, text, reference=address == 0x465210)
        oracle.run(0x466E7C, text)
        pcm.extend(oracle.pcm)
    header = bytearray((ROOT / "orig/wavhead.wav").read_bytes())
    struct.pack_into("<I", header, 4, 36 + len(pcm))
    struct.pack_into("<I", header, 40, len(pcm))
    wav = header + pcm
    path = ROOT / "tests/fixtures" / filename
    if args.write:
        path.write_bytes(wav)
    elif path.read_bytes() != wav:
        raise SystemExit(f"Original procedure result differs from fixture: {filename}")
    print(f"{'Wrote' if args.write else 'Verified'} {filename}: {len(pcm)} PCM bytes", flush=True)
