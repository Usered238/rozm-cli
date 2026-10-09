#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
binary=${1:-target/release/rozm-cli}
destination=${2:-dist/ubuntu-x86_64}
data_root=${3:-orig}
[ -f "$binary" ] || { echo "Build the release binary first: $binary" >&2; exit 1; }
for name in BSNbn skfs/skf skfs/wif skfs/wlf snf1/ip2f snf1/lp2f snf1/Sd2f snf2/ip2f snf2/lp2f snf2/Sd2f snf3/ip2f snf3/lp2f snf3/Sd2f; do
    [ -f "$data_root/$name" ] || { echo "Missing resource: $data_root/$name" >&2; exit 1; }
done
mkdir -p "$destination"
cp "$binary" "$destination/rozm-cli"
chmod +x "$destination/rozm-cli"
for name in BSNbn skfs/skf skfs/wif skfs/wlf snf1/ip2f snf1/lp2f snf1/Sd2f snf2/ip2f snf2/lp2f snf2/Sd2f snf3/ip2f snf3/lp2f snf3/Sd2f; do
    mkdir -p "$(dirname "$destination/data/$name")"
    cp "$data_root/$name" "$destination/data/$name"
done
cp docs/package-readme.md "$destination/README.md"
echo "Packaged: $destination"
