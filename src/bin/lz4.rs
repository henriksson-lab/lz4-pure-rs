use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Cursor, Read, Write};
use std::path::{Path, PathBuf};

use clap::Parser;
use lz4::liblz4::{BlockChecksum, BlockMode, BlockSize, ContentChecksum};

const COPY_BUFFER_SIZE: usize = 4 * 1024 * 1024;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Mode {
    Compress,
    Decompress,
}

fn main() {
    if let Err(err) = run(Cli::parse()) {
        eprintln!("lz4: {err}");
        std::process::exit(1);
    }
}

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

fn open_input(path: Option<&Path>) -> io::Result<Box<dyn Read>> {
    match path {
        None => Ok(Box::new(BufReader::new(io::stdin()))),
        Some(path) if path == Path::new("-") => Ok(Box::new(BufReader::new(io::stdin()))),
        Some(path) => Ok(Box::new(BufReader::new(File::open(path)?))),
    }
}

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

fn regular_file_size(path: Option<&Path>) -> Option<u64> {
    let path = path?;
    if path == Path::new("-") {
        return None;
    }
    let metadata = std::fs::metadata(path).ok()?;
    metadata.is_file().then_some(metadata.len())
}

fn cli_block_size(input_size: Option<u64>) -> BlockSize {
    match input_size {
        Some(size) if size <= 64 * 1024 => BlockSize::Max64KB,
        _ => BlockSize::Max4MB,
    }
}

fn compress(
    reader: &mut dyn Read,
    writer: Box<dyn Write>,
    level: u32,
    input_size: Option<u64>,
) -> io::Result<()> {
    let mut builder = lz4::EncoderBuilder::new();
    builder
        .level(level)
        .block_size(cli_block_size(input_size))
        .block_mode(BlockMode::Independent)
        .block_checksum(BlockChecksum::NoBlockChecksum)
        .checksum(ContentChecksum::ChecksumEnabled);
    let mut encoder = builder.build(writer)?;
    io::copy(reader, &mut encoder)?;
    let (mut writer, result) = encoder.finish();
    result?;
    writer.flush()
}

fn decompress_concatenated(reader: &mut dyn Read, writer: &mut dyn Write) -> io::Result<()> {
    let mut input = Vec::new();
    reader.read_to_end(&mut input)?;
    let mut offset = 0usize;

    while offset < input.len() {
        let cursor = Cursor::new(&input[offset..]);
        let mut decoder = lz4::Decoder::new(cursor)?;
        copy_large(&mut decoder, writer)?;
        let (cursor, result) = decoder.finish();
        result?;
        let consumed = cursor.position() as usize;
        if consumed == 0 {
            return Err(invalid_input("empty lz4 frame"));
        }
        offset += consumed;
    }
    Ok(())
}

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

fn invalid_input(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}
