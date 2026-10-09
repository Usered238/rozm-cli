#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
binary=${1:-dist/ubuntu-x86_64/rozm-cli}
mkdir -p target/shell-smoke-linux
output=target/shell-smoke-linux/quoted-newline.wav
env -u ROZM_DATA_DIR -u DISPLAY -u WAYLAND_DISPLAY "$binary" --text $'"а"\nа' --output "$output"
tail -c +45 "$output" > target/shell-smoke-linux/actual.pcm
{ tail -c +45 tests/fixtures/original-a-v1-s5.wav; tail -c +45 tests/fixtures/original-a-v1-s5.wav; } > target/shell-smoke-linux/expected.pcm
cmp target/shell-smoke-linux/actual.pcm target/shell-smoke-linux/expected.pcm
echo 'Bash quoted text and real newline without desktop variables: PASS'
