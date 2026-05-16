# lz4-pure-rs

Pure Rust LZ4 library with the same public API as `lz4-rs`.

Translated from upstream LZ4 commit `9da37b2eebf082bfab6e57c49be71cc41119a40d`.

* 2026-05-16: docstrings added
* 2026-04-25: On par with speed of original LZ4. Passes a fair number of tests and produces exactly the same input/output. More testing should however be done

## This is an LLM-mediated faithful (hopefully) translation, not the original code! 

Most users should probably first see if the existing original code works for them, unless they have reason otherwise. The original source
may have newer features and it has had more love in terms of fixing bugs. In fact, we aim to replicate bugs if they are present, for the
sake of reproducibility! (but then we might have added a few more in the process)

There are however cases when you might prefer this Rust version. We generally agree with [this manifesto](https://rewrites.bio/) but more specifically:
* We have had many issues with ensuring that our software works using existing containers (Docker, PodMan, Singularity). One size does not fit all and it eats our resources trying to keep up with every way of delivering software
* Common package managers do not work well. It was great when we had a few Linux distributions with stable procedures, but now there are just too many ecosystems (Homebrew, Conda). Conda has an NP-complete resolver which does not scale. Homebrew is only so-stable. And our dependencies in Python still break. These can no longer be considered professional serious options. Meanwhile, Cargo enables multiple versions of packages to be available, even within the same program(!)
* The future is the web. We deploy software in the web browser, and until now that has meant Javascript. This is a language where even the == operator is broken. Typescript is one step up, but a game changer is the ability to compile Rust code into webassembly, enabling performance and sharing of code with the backend. Translating code to Rust enables new ways of deployment and running code in the browser has especial benefits for science - researchers do not have deep pockets to run servers, so pushing compute to the user enables deployment that otherwise would be impossible
* Old CLI-based utilities are bad for the environment(!). A large amount of compute resources are spent creating and communicating via small files, which we can bypass by using code as libraries. Even better, we can avoid frequent reloading of databases by hoisting this stage, with up to 100x speedups in some cases. Less compute means faster compute and less electricity wasted
* LLM-mediated translations may actually be safer to use than the original code. This article shows that [running the same code on different operating systems can give somewhat different answers](https://doi.org/10.1038/nbt.3820). This is a gap that Rust+Cargo can reduce. Typesafe interfaces also reduce coding mistakes and error handling, as opposed to typical command-line scripting

But:

* **This approach should still be considered experimental**. The LLM technology is immature and has sharp corners. But there are opportunities to reap, and the genie is not going back into the bottle. This translation is as much aimed to learn how to improve the technology and get feedback on the results.
* Translations are not endorsed by the original authors unless otherwise noted. **Do not send bug reports to the original developers**. Use our Github issues page instead.
* **Do not trust the benchmarks on this page**. They are used to help evaluate the translation. If you want improved performance, you generally have to use this code as a library, and use the additional tricks it offers. We generally accept performance losses in order to reduce our dependency issues
* **Check the original Github pages for information about the package**. This README is kept sparse on purpose. It is not meant to be the primary source of information
* **If you are the author of the original code and wish to move to Rust, you can obtain ownership of this repository and crate**. Until then, our commitment is to offer an as-faithful-as-possible translation of a snapshot of your code. If we find serious bugs, we will report them to you. Otherwise we will just replicate them, to ensure comparability across studies that claim to use package XYZ v.666. Think of this like a fancy Ubuntu .deb-package of your software - that is how we treat it

This blurb might be out of date. Go to [this page](https://github.com/henriksson-lab/rustification) for the latest information and further information about how we approach translation



## API

The crate currently implements the `lz4-rs` safe block, encoder, and decoder
APIs on top of a pure Rust translation. It also exposes the C-shaped `sys`
surface used by those APIs, including block compression/decompression, streaming
state APIs, frame APIs, and LZ4HC entry points.

## CLI

The optional `cli` feature builds a single `lz4` binary using `clap`:

```sh
cargo run --features cli --bin lz4 -- -f input output.lz4
cargo run --features cli --bin lz4 -- -d output.lz4 restored
```

## Testing

Frame and block output is format-compatible with upstream LZ4 on the tested
paths. The test suite includes byte fixtures generated from upstream C for fast
compression, dictionary compression, frame compression, and HC compression, plus
negative frame tests for malformed headers, checksums, content-size mismatches,
oversized block headers, linked blocks, and skippable frames.

`tools/lz4_perf_check.sh` compares the release CLI against the installed system
`lz4` on generated random, zero-filled, source-like, JSON/log-like, FASTA-like,
dictionary-heavy, binary-artifact, tar/many-small-file, and already-compressed
samples. As of April 25, 2026, **CLI output is byte-identical to system
`lz4 1.9.4` at every level** (default plus `-l 1` through `-l 12`) for every
corpus input, and both implementations validate each other's output.

### Speed vs system `lz4 1.9.4` (5-run median)

Wall-clock from `tools/lz4_perf_check.sh`. Compressed sizes are byte-identical
to system at every level shown.

**Compression:**

| Input | Size | Rust | System | Δ |
|---|---:|---:|---:|---:|
| random64 | 64 MiB | 0.27 s | 0.25 s | +8% |
| zeros64 | 64 MiB | 0.04 s | 0.05 s | Rust −20% |
| source-repeat | 78 MiB | 0.31 s | 0.28 s | +11% |
| loglike | 22 MiB | 0.03 s | 0.03 s | parity |
| fasta-like | 43 MiB | 0.04 s | 0.05 s | Rust −20% |
| dictionary-heavy | 43 MiB | 0.08 s | 0.07 s | +14% |
| already-compressed | 17 MiB | 0.08 s | 0.08 s | parity |
| HC level 9 | 78 MiB | 3.52 s | 3.62 s | Rust −3% |
| HC level 10 | 78 MiB | 5.02 s | 4.53 s | +11% |
| HC level 11 | 78 MiB | 10.71 s | 10.64 s | parity |
| HC level 12 | 78 MiB | 9.64 s | 9.22 s | +5% |

**Decompression** (decoding a system-generated `.lz4`):

| Input | Rust | System | Δ |
|---|---:|---:|---:|
| random64 | 0.11 s | 0.10 s | +10% |
| zeros64 | 0.09 s | 0.10 s | Rust −10% |
| source-repeat | 0.13 s | 0.15 s | Rust −13% |
| loglike | 0.04 s | 0.04 s | parity |
| fasta-like | 0.05 s | 0.07 s | Rust −29% |
| dictionary-heavy | 0.06 s | 0.07 s | Rust −14% |
| already-compressed | 0.04 s | 0.03 s | +33% (10 ms abs) |

In short: Rust is at parity or faster than `lz4 1.9.4` on most workloads.
Variance between runs is ±5% on these short timings, so several "+5–10%"
entries cross to parity in any given run. The remaining real (rather than
noise-band) gaps are HC level 10 (+11%), `already-compressed` decompress
(33% relative but only 10 ms absolute on a 17 MiB file), and source-repeat
default compress (+11%).

## License

License is the same as the LZ4 library, i.e. it is provided as open-source software using BSD 2-Clause license.
