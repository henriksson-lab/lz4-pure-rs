//! `lz4` command-line interface, gated on the `cli` feature.
//!
//! Mirrors the upstream `lz4(1)` CLI for the subset of flags this binary
//! supports: compress/decompress/test, write to stdout, force overwrite,
//! and a numeric compression level (`-l 0..=12`). When neither `-z` nor
//! `-d`/`-t` is given, the mode is inferred from the input filename — a
//! `.lz4` extension selects decompression, anything else selects
//! compression.
//!
//! Frame preferences match the upstream `lz4` CLI defaults (and therefore
//! diverge from this crate's library defaults): 4 MiB blocks for inputs
//! larger than 64 KiB (64 KiB blocks otherwise), independent blocks,
//! content checksum enabled, block checksum disabled, no content-size
//! field, and a level-to-mode remap where levels `0`, `1`, `2` use fast
//! mode and level `3+` uses HC. See `CLAUDE.md` and
//! `upstream/lz4/programs/lz4.1.md` for full upstream behaviour.

use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use clap::Parser;
use lz4::liblz4::{BlockChecksum, BlockMode, BlockSize, ContentChecksum};

/// Buffer size used when shuttling bytes between reader/encoder and
/// decoder/writer. Sized to match the 4 MiB upstream block size so a full
/// block is handed to `LZ4F_compressUpdate` in one call.
const COPY_BUFFER_SIZE: usize = 4 * 1024 * 1024;

/// Parsed command-line arguments. See the module-level docs for the
/// upstream CLI subset this struct models.
#[derive(Debug, Parser)]
#[command(name = "lz4")]
#[command(about = "Compress or decompress .lz4 files")]
struct Cli {
    /// Decompress input.
    #[arg(short = 'd', long = "decompress", conflicts_with = "compress")]
    decompress: bool,

    /// Compress input.
    #[arg(short = 'z', long = "compress")]
    compress: bool,

    /// Test compressed input integrity without writing decoded output.
    #[arg(short = 't', long = "test", conflicts_with = "compress")]
    test: bool,

    /// Write to standard output.
    #[arg(short = 'c', long = "stdout")]
    stdout: bool,

    /// Overwrite output files.
    #[arg(short = 'f', long = "force")]
    force: bool,

    /// Compression level, 0-12. Levels 10-12 use high compression.
    #[arg(short = 'l', long = "level", default_value_t = 0)]
    level: u32,

    /// Input file, or '-' for standard input.
    input: Option<PathBuf>,

    /// Output file. Use '-' for standard output.
    output: Option<PathBuf>,
}

/// Operating mode selected from the CLI flags and/or input filename.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Mode {
    Compress,
    Decompress,
}

/// CLI entry point. Parses arguments, dispatches to [`run`], and exits
/// with status 1 (printing the error to stderr) on failure.
fn main() {
    if let Err(err) = run(Cli::parse()) {
        eprintln!("lz4: {err}");
        std::process::exit(1);
    }
}

/// Validates arguments, opens input/output, and dispatches to [`compress`]
/// or [`decompress_concatenated`] based on the selected [`Mode`].
fn run(cli: Cli) -> io::Result<()> {
    if cli.level > 12 {
        return Err(invalid_input("compression level must be between 0 and 12"));
    }

    let mode = mode(&cli);
    let input = cli.input.as_deref();
    let output = output_path(&cli, mode)?;

    match mode {
        Mode::Compress => {
            let mut reader = open_input(input)?;
            let writer = create_output(output.as_deref(), cli.force)?;
            let input_size = regular_file_size(input);
            compress(&mut reader, writer, cli.level, input_size)
        }
        Mode::Decompress if cli.test => {
            let mut reader = open_input(input)?;
            decompress_concatenated(&mut reader, &mut io::sink())
        }
        Mode::Decompress => {
            let mut reader = open_input(input)?;
            let mut writer = create_output(output.as_deref(), cli.force)?;
            decompress_concatenated(&mut reader, &mut writer)?;
            writer.flush()
        }
    }
}

/// Chooses [`Mode::Compress`] or [`Mode::Decompress`] from the CLI flags,
/// falling back to a `.lz4`-extension probe of the input filename when
/// neither `-z` nor `-d`/`-t` was supplied.
fn mode(cli: &Cli) -> Mode {
    if cli.compress {
        return Mode::Compress;
    }
    if cli.decompress || cli.test {
        return Mode::Decompress;
    }
    match cli.input.as_deref() {
        Some(path) if path != Path::new("-") && path.extension() == Some(OsStr::new("lz4")) => {
            Mode::Decompress
        }
        _ => Mode::Compress,
    }
}

/// Resolves the output file path. Returns `Ok(None)` to indicate stdout
/// (for `-c`, explicit `-` output, or `-t` test mode) and otherwise infers
/// a path by appending `.lz4` (compress) or stripping it (decompress)
/// when no explicit output was provided.
fn output_path(cli: &Cli, mode: Mode) -> io::Result<Option<PathBuf>> {
    if cli.test {
        return Ok(None);
    }
    if cli.stdout || cli.output.as_deref() == Some(Path::new("-")) {
        return Ok(None);
    }
    if let Some(output) = &cli.output {
        return Ok(Some(output.clone()));
    }

    let Some(input) = &cli.input else {
        return Err(invalid_input("missing output path when reading from stdin"));
    };
    if input == Path::new("-") {
        return Err(invalid_input("missing output path when reading from stdin"));
    }

    match mode {
        Mode::Compress => {
            let mut out = input.as_os_str().to_os_string();
            out.push(".lz4");
            Ok(Some(PathBuf::from(out)))
        }
        Mode::Decompress => strip_lz4_suffix(input).map(Some),
    }
}

/// Strips a trailing `.lz4` extension to produce the implicit
/// decompression output name, or returns an error if the input does not
/// end in `.lz4`.
fn strip_lz4_suffix(input: &Path) -> io::Result<PathBuf> {
    let name = input.as_os_str().to_string_lossy();
    let Some(stripped) = name.strip_suffix(".lz4") else {
        return Err(invalid_input(
            "cannot infer output name for decompression; pass an output path",
        ));
    };
    if stripped.is_empty() {
        return Err(invalid_input("invalid .lz4 input name"));
    }
    Ok(PathBuf::from(stripped))
}

/// Opens the input as a buffered reader, falling back to stdin when the
/// path is `None` or `-`.
fn open_input(path: Option<&Path>) -> io::Result<Box<dyn BufRead>> {
    match path {
        None => Ok(Box::new(BufReader::new(io::stdin()))),
        Some(path) if path == Path::new("-") => Ok(Box::new(BufReader::new(io::stdin()))),
        Some(path) => Ok(Box::new(BufReader::new(File::open(path)?))),
    }
}

/// Opens the output as a buffered writer. When `path` is `None`, returns
/// a writer over stdout. When `force` is `false`, refuses to overwrite an
/// existing file (`create_new`); when `true`, truncates any existing file.
fn create_output(path: Option<&Path>, force: bool) -> io::Result<Box<dyn Write>> {
    match path {
        None => Ok(Box::new(BufWriter::new(io::stdout()))),
        Some(path) => {
            let file = File::options()
                .write(true)
                .create_new(!force)
                .create(force)
                .truncate(force)
                .open(path)?;
            Ok(Box::new(BufWriter::new(file)))
        }
    }
}

/// Returns the size of `path` if it refers to a regular file on disk, so
/// the encoder can pick a smaller block size for short inputs. Returns
/// `None` for stdin, missing files, and non-regular files.
fn regular_file_size(path: Option<&Path>) -> Option<u64> {
    let path = path?;
    if path == Path::new("-") {
        return None;
    }
    let metadata = std::fs::metadata(path).ok()?;
    metadata.is_file().then_some(metadata.len())
}

/// Picks the LZ4 frame block size to match upstream `lz4` CLI behaviour:
/// 64 KiB blocks for inputs at most 64 KiB, 4 MiB blocks otherwise (or
/// when the size is unknown, e.g. stdin).
fn cli_block_size(input_size: Option<u64>) -> BlockSize {
    match input_size {
        Some(size) if size <= 64 * 1024 => BlockSize::Max64KB,
        _ => BlockSize::Max4MB,
    }
}

/// Compresses `reader` into `writer` as a single `.lz4` frame using the
/// CLI's upstream-matching preferences (see module docs). `level` follows
/// the upstream remap where `0..=2` route through fast mode and `3..=12`
/// through HC, and `input_size` (when known) selects between 64 KiB and
/// 4 MiB frame blocks.
fn compress(
    reader: &mut dyn Read,
    writer: Box<dyn Write>,
    level: u32,
    input_size: Option<u64>,
) -> io::Result<()> {
    // Match upstream `lz4` CLI's level→mode mapping: levels 0, 1, 2 are
    // fast-mode (acceleration variants of `LZ4_compress_fast`), level 3+
    // is HC. Without this, our CLI's `-l 1`/`-l 2` would route through
    // HC level 1/2 (LZ4MID) and produce a different .lz4 file than
    // `lz4 -1`/`lz4 -2`. The library API (`LZ4_compress_HC` at level 1)
    // is unaffected — this remap only happens at the CLI layer.
    let effective_level = if level < 3 { 0 } else { level };
    let mut builder = lz4::EncoderBuilder::new();
    builder
        .level(effective_level)
        .block_size(cli_block_size(input_size))
        .block_mode(BlockMode::Independent)
        .block_checksum(BlockChecksum::NoBlockChecksum)
        .checksum(ContentChecksum::ChecksumEnabled);
    let mut encoder = builder.build(writer)?;
    // Use a 4 MiB staging buffer instead of `io::copy`'s 8 KiB stack buffer
    // so the frame block size (also 4 MiB for larger files) is fed to the
    // encoder in one shot and we don't pay per-8KB `memmove` overhead.
    copy_large(reader, &mut encoder)?;
    let (mut writer, result) = encoder.finish();
    result?;
    writer.flush()
}

/// Decompresses one or more concatenated `.lz4` frames from `reader`,
/// emitting the decompressed bytes to `writer`. Stops cleanly when the
/// reader reports EOF between frames.
fn decompress_concatenated(reader: &mut dyn BufRead, writer: &mut dyn Write) -> io::Result<()> {
    loop {
        if reader.fill_buf()?.is_empty() {
            return Ok(());
        }
        let mut decoder = lz4::Decoder::new(&mut *reader)?;
        copy_large(&mut decoder, writer)?;
        let (_, result) = decoder.finish();
        result?;
    }
}

/// Copies `reader` to `writer` using a [`COPY_BUFFER_SIZE`] staging buffer
/// (4 MiB) instead of `io::copy`'s default 8 KiB, so the encoder receives
/// full frame blocks in one call.
fn copy_large(reader: &mut dyn Read, writer: &mut dyn Write) -> io::Result<u64> {
    let mut buffer = vec![0u8; COPY_BUFFER_SIZE];
    let mut total = 0u64;
    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            return Ok(total);
        }
        writer.write_all(&buffer[..n])?;
        total += n as u64;
    }
}

/// Builds an [`io::ErrorKind::InvalidInput`] error with a static message.
fn invalid_input(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}
