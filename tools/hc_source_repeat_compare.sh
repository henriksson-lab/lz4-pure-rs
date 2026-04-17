#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORKDIR="${TMPDIR:-/tmp}/lz4-pure-hc-compare"
BLOCK_SIZE=$((4 * 1024 * 1024))
BLOCKS="${1:-4}"
mkdir -p "$WORKDIR"

SOURCE_REPEAT="$WORKDIR/source-repeat.bin"
RUST_FRAME="$WORKDIR/source-repeat.rust.hc9.lz4"
PROBE="$WORKDIR/upstream_hc_block_probe"

source_hash="$(sha256sum "$ROOT/src/sys.rs" | awk '{print $1}')"
stamp="$WORKDIR/source-repeat.sha256"
if [ ! -f "$SOURCE_REPEAT" ] || [ ! -f "$stamp" ] || [ "$(cat "$stamp")" != "$source_hash" ]; then
    : > "$SOURCE_REPEAT"
    for _ in $(seq 1 256); do
        cat "$ROOT/src/sys.rs" >> "$SOURCE_REPEAT"
    done
    printf '%s\n' "$source_hash" > "$stamp"
fi

cargo build --release --features cli >/dev/null
"$ROOT/target/release/lz4" -l 9 -f "$SOURCE_REPEAT" "$RUST_FRAME" >/dev/null

gcc -O2 -I "$ROOT/upstream/lz4/lib" \
    "$ROOT/tools/upstream_hc_block_probe.c" \
    "$ROOT/upstream/lz4/lib/lz4.c" \
    "$ROOT/upstream/lz4/lib/lz4hc.c" \
    "$ROOT/upstream/lz4/lib/xxhash.c" \
    -o "$PROBE"

python3 - "$RUST_FRAME" "$SOURCE_REPEAT" "$PROBE" "$BLOCKS" "$BLOCK_SIZE" <<'PY'
import struct
import subprocess
import sys

frame_path, source_path, probe_path, blocks_s, block_size_s = sys.argv[1:]
blocks = int(blocks_s)
block_size = int(block_size_s)
frame = open(frame_path, "rb").read()
source_len = len(open(source_path, "rb").read())

def xxh32(data: bytes, seed: int = 0) -> int:
    def u32(value):
        return value & 0xFFFFFFFF
    def rol(value, bits):
        return u32((value << bits) | (value >> (32 - bits)))
    def read32(offset):
        return struct.unpack_from("<I", data, offset)[0]
    def round_(acc, value):
        return u32(rol(u32(acc + u32(value * 0x85EBCA77)), 13) * 0x9E3779B1)

    pos = 0
    length = len(data)
    if length >= 16:
        v1 = u32(seed + 0x9E3779B1 + 0x85EBCA77)
        v2 = u32(seed + 0x85EBCA77)
        v3 = seed
        v4 = u32(seed - 0x9E3779B1)
        limit = length - 16
        while pos <= limit:
            v1 = round_(v1, read32(pos)); pos += 4
            v2 = round_(v2, read32(pos)); pos += 4
            v3 = round_(v3, read32(pos)); pos += 4
            v4 = round_(v4, read32(pos)); pos += 4
        h = u32(rol(v1, 1) + rol(v2, 7) + rol(v3, 12) + rol(v4, 18))
    else:
        h = u32(seed + 0x165667B1)
    h = u32(h + length)
    while pos + 4 <= length:
        h = u32(rol(u32(h + u32(read32(pos) * 0xC2B2AE3D)), 17) * 0x27D4EB2F)
        pos += 4
    while pos < length:
        h = u32(rol(u32(h + data[pos] * 0x165667B1), 11) * 0x9E3779B1)
        pos += 1
    h ^= h >> 15
    h = u32(h * 0x85EBCA77)
    h ^= h >> 13
    h = u32(h * 0xC2B2AE3D)
    h ^= h >> 16
    return u32(h)

pos = 4
flg = frame[pos]
bd = frame[pos + 1]
pos += 2
if flg & 0x08:
    pos += 8
if flg & 0x01:
    pos += 4
pos += 1

print(f"source_len={source_len} rust_frame_len={len(frame)} flg=0x{flg:02x} bd=0x{bd:02x}")
print("block  upstream_len  upstream_xxh32  rust_len  rust_xxh32")
for index in range(blocks):
    if pos + 4 > len(frame):
        break
    header = struct.unpack_from("<I", frame, pos)[0]
    pos += 4
    if header == 0:
        break
    rust_len = header & 0x7FFF_FFFF
    raw = bool(header & 0x8000_0000)
    block = frame[pos:pos + rust_len]
    pos += rust_len
    if raw:
        rust_hash = "raw"
    else:
        rust_hash = f"{xxh32(block):08x}"
    length = min(block_size, source_len - index * block_size)
    upstream = subprocess.check_output(
        [probe_path, source_path, str(index * block_size), str(length), "9"],
        text=True,
    ).strip()
    parts = dict(item.split("=") for item in upstream.split())
    print(
        f"{index:<5} {parts['len']:>12}  {parts['xxh32']:>13}  "
        f"{rust_len:>8}  {rust_hash}"
    )
PY
