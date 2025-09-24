//! Command-line tool for embedding and retrieving messages in PNG files.

#![deny(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(rust_2018_idioms)]

// PNG signature indicating the remainder of the datastream contains a single
// PNG image, consisting of a series of chunks beginning with an `IHDR` chunk
// and ending with an `IEND` chunk.
//const MAGIC: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]

mod chunk_type;

use anyhow::Result;

fn main() -> Result<()> {
    todo!()
}
