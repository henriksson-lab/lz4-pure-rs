#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORKDIR="${LZ4_PURE_PERF_DIR:-/tmp/lz4-pure-perf}"
RUST_LZ4="$ROOT/target/release/lz4"
SYSTEM_LZ4="${SYSTEM_LZ4:-lz4}"

mkdir -p "$WORKDIR"

if ! command -v "$SYSTEM_LZ4" >/dev/null 2>&1; then
    echo "system lz4 not found; set SYSTEM_LZ4=/path/to/lz4" >&2
    exit 1
fi

cargo build --release --features cli --manifest-path "$ROOT/Cargo.toml" >/dev/null

make_corpus() {
    if [ ! -f "$WORKDIR/random64.bin" ]; then
        dd if=/dev/urandom of="$WORKDIR/random64.bin" bs=1M count=64 status=none
    fi
    if [ ! -f "$WORKDIR/zeros64.bin" ]; then
        dd if=/dev/zero of="$WORKDIR/zeros64.bin" bs=1M count=64 status=none
    fi
    if [ ! -f "$WORKDIR/source-repeat.bin" ]; then
        : > "$WORKDIR/source-repeat.bin"
        for _ in $(seq 1 256); do
            cat "$ROOT/src/sys.rs" >> "$WORKDIR/source-repeat.bin"
        done
    fi
    if [ ! -f "$WORKDIR/loglike.bin" ]; then
        : > "$WORKDIR/loglike.bin"
        for i in $(seq 1 200000); do
            printf '{"time":"2026-04-16T12:%02d:%02dZ","level":"INFO","worker":%d,"message":"lz4 pure rust benchmark line","value":%d}\n' \
                "$((i % 60))" "$(((i / 60) % 60))" "$((i % 32))" "$i" >> "$WORKDIR/loglike.bin"
        done
    fi
}

time_command() {
    local label="$1"
    local timefile="$2"
    local stdoutfile="$3"
    shift 3
    printf '%-34s' "$label"
    /usr/bin/time -f '%e s' -o "$timefile" "$@" > "$stdoutfile"
    cat "$timefile"
}

run_one() {
    local name="$1"
    local input="$WORKDIR/$name.bin"
    local rust_lz4="$WORKDIR/$name.rust.lz4"
    local system_lz4="$WORKDIR/$name.system.lz4"
    local rust_out="$WORKDIR/$name.rust.out"
    local system_out="$WORKDIR/$name.system.out"

    echo
    echo "== $name ($(wc -c < "$input") bytes) =="
    time_command "rust compress" "$WORKDIR/$name.rust.compress.time" /dev/null "$RUST_LZ4" -f "$input" "$rust_lz4"
    time_command "system compress" "$WORKDIR/$name.system.compress.time" /dev/null "$SYSTEM_LZ4" -q -f "$input" "$system_lz4"
    "$SYSTEM_LZ4" -q -t "$rust_lz4" >/dev/null
    "$RUST_LZ4" -t "$system_lz4" >/dev/null

    printf '%-34s%s bytes\n' "rust compressed size" "$(wc -c < "$rust_lz4")"
    printf '%-34s%s bytes\n' "system compressed size" "$(wc -c < "$system_lz4")"

    time_command "rust decompress system frame" "$WORKDIR/$name.rust.decompress.time" "$rust_out" "$RUST_LZ4" -d -c "$system_lz4"
    time_command "system decompress system frame" "$WORKDIR/$name.system.decompress.time" "$system_out" "$SYSTEM_LZ4" -q -d -c "$system_lz4"
    cmp "$rust_out" "$input"
    cmp "$system_out" "$input"
}

make_corpus
run_one random64
run_one zeros64
run_one source-repeat
run_one loglike

cat "$WORKDIR/source-repeat.system.lz4" "$WORKDIR/random64.system.lz4" > "$WORKDIR/concat.system.lz4"
cat "$WORKDIR/source-repeat.bin" "$WORKDIR/random64.bin" > "$WORKDIR/concat.expected"
"$RUST_LZ4" -d -c "$WORKDIR/concat.system.lz4" > "$WORKDIR/concat.rust.out"
cmp "$WORKDIR/concat.rust.out" "$WORKDIR/concat.expected"

echo
echo "concatenated-frame decode: ok"
echo "workdir: $WORKDIR"
