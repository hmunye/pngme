//! Subcommands for the `pngme` program.

use std::fs;
use std::io::{self, BufReader, Cursor, Read};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::chunk::Chunk;
use crate::chunk_type::ChunkType;
use crate::png::PNG;

use anyhow::{Context, Result};

const DEFAULT_OUTPUT: &str = "out.png";

fn png_parse(file_path: &Path) -> Result<PNG> {
    let f = fs::File::open(file_path)
        .with_context(|| format!("failed to open '{}'", file_path.display()))?;

    let metadata = f
        .metadata()
        .with_context(|| format!("failed to query metadata for '{}'", file_path.display()))?;

    let mut reader = BufReader::new(f);
    let mut buf = Vec::with_capacity(metadata.len() as usize);

    reader
        .read_to_end(&mut buf)
        .with_context(|| format!("failed to read from '{}'", file_path.display()))?;

    PNG::try_from(&buf[..])
}

fn png_write_to_file(png: &PNG, output_path: Option<PathBuf>) -> Result<()> {
    let bytes = png.as_bytes();
    let mut cursor = Cursor::new(&bytes);

    let outfile = if let Some(outfile) = output_path {
        outfile
    } else {
        PathBuf::from(DEFAULT_OUTPUT)
    };

    let mut output = fs::File::create(&outfile)
        .with_context(|| format!("failed to open/create '{}'", outfile.display()))?;

    io::copy(&mut cursor, &mut output)
        .with_context(|| format!("failed to write PNG datastream to '{}'", outfile.display()))?;

    Ok(())
}

/// Encodes a message into the PNG file given its chunk type.
///
/// Writes the modifications to a new PNG file, or the output path if provided.
pub fn invoke_encode(
    png_path: PathBuf,
    chunk_type: String,
    message: String,
    out_path: Option<PathBuf>,
) -> Result<()> {
    let file_path = png_path.as_path();

    let mut png = png_parse(file_path)?;
    let chunk_type = ChunkType::from_str(&chunk_type)?;

    if chunk_type.is_critical()
        || chunk_type.is_public()
        || !chunk_type.is_valid()
        || !chunk_type.is_safe_to_copy()
    {
        anyhow::bail!(
            "failed to append chunk with chunk type '{}' to '{}': must be exactly 4 ASCII letters with specific casing (e.g., \"ruSt\")",
            chunk_type,
            file_path.display()
        );
    }

    let chunk = Chunk::new(chunk_type, message.into())?;

    png.append_chunk(chunk);

    png_write_to_file(&png, out_path)?;

    Ok(())
}

/// Decodes a message from the PNG file given the chunk type, returning the
/// message of the chunk, or `None` if it could not be found.
pub fn invoke_decode(png_path: PathBuf, chunk_type: String) -> Result<Option<String>> {
    let png = png_parse(png_path.as_path())?;
    let chunk = png.chunk_by_type(&chunk_type);

    Ok(chunk.map(|c| c.to_string()))
}

/// Removes a message from the PNG file given the chunk type, returning the
/// message of the chunk, or `None` if it could not be found.
///
/// Writes the modifications to a new PNG file, or the output path if provided.
pub fn invoke_remove(
    png_path: PathBuf,
    chunk_type: String,
    out_path: Option<PathBuf>,
) -> Result<Option<String>> {
    let mut png = png_parse(png_path.as_path())?;

    let chunk = png.remove_chunk(&chunk_type);
    let message = chunk.map(|c| c.to_string());

    png_write_to_file(&png, out_path)?;

    Ok(message)
}
