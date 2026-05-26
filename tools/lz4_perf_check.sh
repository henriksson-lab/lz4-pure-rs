#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORKDIR="${LZ4_PURE_PERF_DIR:-/tmp/lz4-pure-perf}"
RUST_LZ4="$ROOT/target/release/lz4"
SYSTEM_LZ4="${SYSTEM_LZ4:-lz4}"
RUNS="${LZ4_PURE_PERF_RUNS:-1}"
PARITY_SWEEP="${LZ4_PURE_PARITY_SWEEP:-0}"
SOURCE_REPEAT_STAMP="$WORKDIR/source-repeat.sha256"
CORPUS_NAMES=(
    random64
    zeros64
    source-repeat
    loglike
    fasta-like
    dictionary-heavy
    binary-artifact
    many-small.tar
    already-compressed
)

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
    local source_hash
    source_hash="$(sha256sum "$ROOT/src/sys.rs" | awk '{print $1}')"
    if [ ! -f "$WORKDIR/source-repeat.bin" ] || [ ! -f "$SOURCE_REPEAT_STAMP" ] || [ "$(cat "$SOURCE_REPEAT_STAMP")" != "$source_hash" ]; then
        : > "$WORKDIR/source-repeat.bin"
        for _ in $(seq 1 256); do
            cat "$ROOT/src/sys.rs" >> "$WORKDIR/source-repeat.bin"
        done
        printf '%s\n' "$source_hash" > "$SOURCE_REPEAT_STAMP"
    fi
    if [ ! -f "$WORKDIR/loglike.bin" ]; then
        : > "$WORKDIR/loglike.bin"
        for i in $(seq 1 200000); do
            printf '{"time":"2026-04-16T12:%02d:%02dZ","level":"INFO","worker":%d,"message":"lz4 pure rust benchmark line","value":%d}\n' \
                "$((i % 60))" "$(((i / 60) % 60))" "$((i % 32))" "$i" >> "$WORKDIR/loglike.bin"
        done
    fi
    if [ ! -f "$WORKDIR/fasta-like.bin" ]; then
        : > "$WORKDIR/fasta-like.bin"
        for i in $(seq 1 250000); do
            printf '>read_%06d length=160 sample=lz4-pure-rs\n' "$i" >> "$WORKDIR/fasta-like.bin"
            printf 'ACGTGCAANNNNACGTACGTGGTTAACCGGTTACGTACGTGCAATTAACCGGTTNNNNACGTACGTGCAATTAACCGGTTACGTGCAANNNNACGTACGTGGTTAACCGGTTACGTACGTGCAATTAACCGGTTNNNN\n' >> "$WORKDIR/fasta-like.bin"
        done
    fi
    if [ ! -f "$WORKDIR/dictionary-heavy.bin" ]; then
        : > "$WORKDIR/dictionary-heavy.bin"
        for i in $(seq 1 400000); do
            printf 'customer=%05d region=%02d event=checkout status=ok payload=sku-12345,sku-22222,sku-33333 note=shared-prefix-%04d\n' \
                "$((i % 10000))" "$((i % 32))" "$((i % 4096))" >> "$WORKDIR/dictionary-heavy.bin"
        done
    fi
    if [ ! -f "$WORKDIR/binary-artifact.bin" ]; then
        cp "$RUST_LZ4" "$WORKDIR/binary-artifact.bin"
        find "$ROOT/target/release/deps" -maxdepth 1 -type f \( -name '*.rlib' -o -name '*.so' -o -perm -111 \) \
            -print -quit | while IFS= read -r artifact; do
                cat "$artifact" >> "$WORKDIR/binary-artifact.bin"
            done
    fi
    if [ ! -f "$WORKDIR/many-small.tar.bin" ]; then
        local smalldir="$WORKDIR/many-small-src"
        rm -rf "$smalldir"
        mkdir -p "$smalldir"
        for i in $(seq 1 2000); do
            printf 'small-file=%04d\ncommon-prefix=lz4-pure-rs\nvalue=%08d\n' "$i" "$((i * 17))" > "$smalldir/file-$i.txt"
        done
        tar -C "$smalldir" -cf "$WORKDIR/many-small.tar.bin" .
    fi
    if [ ! -f "$WORKDIR/already-compressed.bin" ]; then
        "$SYSTEM_LZ4" -q -f "$WORKDIR/source-repeat.bin" "$WORKDIR/already-compressed.bin"
    fi
}

time_command() {
    local label="$1"
    local timefile="$2"
    local stdoutfile="$3"
    shift 3
    local times=()
    printf '%-34s' "$label"
    for _ in $(seq 1 "$RUNS"); do
        /usr/bin/time -f '%e' -o "$timefile" "$@" > "$stdoutfile"
        times+=("$(cat "$timefile")")
    done
    mapfile -t times < <(printf '%s\n' "${times[@]}" | sort -n)
    local median="${times[$((RUNS / 2))]}"
    if [ "$RUNS" -eq 1 ]; then
        printf '%s s\n' "$median"
    else
        printf 'median %s s (%s runs, best %s s)\n' "$median" "$RUNS" "${times[0]}"
    fi
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
    cmp "$rust_lz4" "$system_lz4"

    printf '%-34s%s bytes\n' "rust compressed size" "$(wc -c < "$rust_lz4")"
    printf '%-34s%s bytes\n' "system compressed size" "$(wc -c < "$system_lz4")"

    time_command "rust decompress system frame" "$WORKDIR/$name.rust.decompress.time" "$rust_out" "$RUST_LZ4" -d -c "$system_lz4"
    time_command "system decompress system frame" "$WORKDIR/$name.system.decompress.time" "$system_out" "$SYSTEM_LZ4" -q -d -c "$system_lz4"
    cmp "$rust_out" "$input"
    cmp "$system_out" "$input"
}

run_parity_sweep() {
    echo
    echo "== byte-parity sweep: default and levels 1..12 =="
    "$SYSTEM_LZ4" --version 2>&1 | sed 's/^/system /'
    local level rust_args system_args suffix input rust_lz4 system_lz4
    for name in "${CORPUS_NAMES[@]}"; do
        input="$WORKDIR/$name.bin"
        for level in default $(seq 1 12); do
            suffix="$level"
            rust_lz4="$WORKDIR/$name.rust.level-$suffix.lz4"
            system_lz4="$WORKDIR/$name.system.level-$suffix.lz4"
            rust_args=()
            system_args=(-q)
            if [ "$level" != default ]; then
                rust_args=(-l "$level")
                system_args+=("-$level")
            fi
            "$RUST_LZ4" "${rust_args[@]}" -f "$input" "$rust_lz4"
            "$SYSTEM_LZ4" "${system_args[@]}" -f "$input" "$system_lz4"
            cmp "$rust_lz4" "$system_lz4"
        done
        printf '  %-24s ok\n' "$name"
    done
}

make_corpus
for name in "${CORPUS_NAMES[@]}"; do
    run_one "$name"
done

for level in 9 10 11 12; do
    echo
    echo "== source-repeat HC level $level ($(wc -c < "$WORKDIR/source-repeat.bin") bytes) =="
    time_command "rust hc$level compress" "$WORKDIR/source-repeat.rust.hc$level.compress.time" /dev/null "$RUST_LZ4" -l "$level" -f "$WORKDIR/source-repeat.bin" "$WORKDIR/source-repeat.rust.hc$level.lz4"
    time_command "system hc$level compress" "$WORKDIR/source-repeat.system.hc$level.compress.time" /dev/null "$SYSTEM_LZ4" -q "-$level" -f "$WORKDIR/source-repeat.bin" "$WORKDIR/source-repeat.system.hc$level.lz4"
    "$SYSTEM_LZ4" -q -t "$WORKDIR/source-repeat.rust.hc$level.lz4" >/dev/null
    "$RUST_LZ4" -t "$WORKDIR/source-repeat.system.hc$level.lz4" >/dev/null
    cmp "$WORKDIR/source-repeat.rust.hc$level.lz4" "$WORKDIR/source-repeat.system.hc$level.lz4"
    printf '%-34s%s bytes\n' "rust hc$level compressed size" "$(wc -c < "$WORKDIR/source-repeat.rust.hc$level.lz4")"
    printf '%-34s%s bytes\n' "system hc$level compressed size" "$(wc -c < "$WORKDIR/source-repeat.system.hc$level.lz4")"
done

if [ "$PARITY_SWEEP" = "1" ]; then
    run_parity_sweep
fi

cat "$WORKDIR/source-repeat.system.lz4" "$WORKDIR/random64.system.lz4" > "$WORKDIR/concat.system.lz4"
cat "$WORKDIR/source-repeat.bin" "$WORKDIR/random64.bin" > "$WORKDIR/concat.expected"
"$RUST_LZ4" -d -c "$WORKDIR/concat.system.lz4" > "$WORKDIR/concat.rust.out"
cmp "$WORKDIR/concat.rust.out" "$WORKDIR/concat.expected"

echo
echo "concatenated-frame decode: ok"
echo "workdir: $WORKDIR"
