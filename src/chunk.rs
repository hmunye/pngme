//! PNG chunks consist of three or four fields:
//!
//! - Length: A PNG four-byte unsigned integer giving the number of bytes in the
//!   chunk's data field. The length counts only the data field, not itself, the
//!   chunk type, or the CRC. Zero is a valid length. Although encoders and
//!   decoders should treat the length as unsigned, its value shall not exceed
//!   2^31-1 [`i32::MAX`] bytes.
//!
//! - [Chunk Type]
//!
//! - Chunk Data: The data bytes appropriate to the chunk type, if any. This
//!   field can be of zero length.
//!
//! - CRC: A four-byte CRC calculated on the preceding bytes in the chunk,
//!   including the chunk type field and chunk data fields, but not including
//!   the length field. The CRC can be used to check for corruption of the data.
//!   The CRC is always present, even for chunks containing no data.
//!
//! [Chunk Type]: crate::chunk_type

use std::convert::TryFrom;
use std::{fmt, mem, result};

use crate::chunk_type::ChunkType;

use anyhow::Result;

/// Table of CRCs of all 8-bit messages.
const CRC_TABLE: [u32; 256] = precompute_crc_table();

/// Returns a CRC table for all byte values (0-255), computed at compile-time.
const fn precompute_crc_table() -> [u32; 256] {
    let mut table = [0u32; 256];

    let mut i = 0;
    while i < 256 {
        let mut crc = i as u32;

        let mut j = 0;
        while j < 8 {
            if crc & 1 == 1 {
                crc = 0xEDB88320 ^ (crc >> 1);
            } else {
                crc = crc >> 1;
            }

            j += 1;
        }

        table[i as usize] = crc;
        i += 1;
    }

    table
}

/// A single chunk within a PNG datastream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chunk {
    length: u32,
    chunk_type: ChunkType,
    data: Vec<u8>,
    crc: u32,
}

impl Chunk {
    /// Creates a new PNG chunk with the specified chunk type and data.
    ///
    /// # Errors
    ///
    /// Returns an error if the data's length in bytes exceeds [`i32::MAX`].
    pub fn new(chunk_type: ChunkType, data: Vec<u8>) -> Result<Chunk> {
        anyhow::ensure!(
            data.len() <= i32::MAX as usize,
            "invalid PNG chunk: data length must not exceed i32::MAX bytes, but received: {}",
            data.len()
        );

        let crc = Chunk::compute_crc(chunk_type.bytes(), &data);

        Ok(Chunk {
            // SAFETY: `data.len()` is <= i32::MAX.
            length: data.len() as u32,
            chunk_type,
            data,
            crc,
        })
    }

    /// Returns the length of the chunk.
    #[inline]
    pub const fn length(&self) -> u32 {
        self.length
    }

    /// Returns the `ChunkType` of the chunk.
    #[inline]
    pub const fn chunk_type(&self) -> ChunkType {
        self.chunk_type
    }

    /// Returns a shared reference to the chunk data.
    #[inline]
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Returns the CRC of the chunk.
    #[inline]
    pub const fn crc(&self) -> u32 {
        self.crc
    }

    /// Returns the size in bytes of the chunk.
    #[inline]
    pub const fn size(&self) -> usize {
        mem::size_of::<u32>() * 2 + mem::size_of::<ChunkType>() + self.data.len()
    }

    /// Returns the memory representation of the chunk as a byte array in
    /// big-endian (network) byte order.
    pub fn as_bytes(&self) -> Vec<u8> {
        let chunk_size = self.size();

        let mut bytes = Vec::with_capacity(chunk_size);
        // SAFETY: Each bytes is being initialized before returning.
        #[allow(clippy::uninit_vec)]
        unsafe {
            bytes.set_len(chunk_size);
        }

        let mut offset = 0;
        bytes[offset..offset + 4].copy_from_slice(&self.length.to_be_bytes());
        offset += 4;
        bytes[offset..offset + 4].copy_from_slice(&self.chunk_type.bytes());
        offset += 4;
        if !self.data.is_empty() {
            bytes[offset..offset + self.data.len()].copy_from_slice(&self.data);
            offset += self.data.len();
        }
        bytes[offset..offset + 4].copy_from_slice(&self.crc.to_be_bytes());

        debug_assert!(
            offset + 4 == chunk_size, // Account for the last 4 bytes copied.
            "failed to properly initialized PNG chunk byte representation: offset: {}, total_length: {}",
            offset + 4,
            chunk_size
        );

        bytes
    }

    /// Returns the computed CRC (Cyclic Redundancy Check) of the chunk using
    /// the `ChunkType` and chunk data bytes.
    fn compute_crc(chunk_ty: [u8; 4], data: &[u8]) -> u32 {
        let mut crc: u32 = 0xFFFFFFFF;

        for b in chunk_ty.iter().chain(data.iter()) {
            // Use precomputed CRC table for fast check.
            crc = CRC_TABLE[((crc ^ *b as u32) & 0xFF) as usize] ^ (crc >> 8);
        }

        crc ^ 0xFFFFFFFF
    }
}

impl TryFrom<&[u8]> for Chunk {
    type Error = anyhow::Error;

    fn try_from(bytes: &[u8]) -> result::Result<Self, Self::Error> {
        let chunk_len = bytes.len();

        // Should contain at least the length, chunk type, and CRC, which totals
        // to 12 bytes.
        anyhow::ensure!(
            chunk_len >= 12,
            "invalid PNG chunk: input must be at least 12 bytes, but received: {}",
            chunk_len
        );

        let length = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let chunk_type = ChunkType::try_from([bytes[4], bytes[5], bytes[6], bytes[7]])?;
        let data = &bytes[8..chunk_len - 4];

        anyhow::ensure!(
            length as usize == data.len(),
            "invalid PNG chunk: chunk length does not match length of chunk data: expected {}, but received {}",
            length,
            data.len()
        );

        let crc = u32::from_be_bytes([
            bytes[chunk_len - 4],
            bytes[chunk_len - 3],
            bytes[chunk_len - 2],
            bytes[chunk_len - 1],
        ]);

        let computed_crc = Chunk::compute_crc(chunk_type.bytes(), data);

        anyhow::ensure!(
            crc == computed_crc,
            "invalid PNG chunk: CRC verification fail: expected 0x{:x}, computed 0x{:x}",
            crc,
            computed_crc
        );

        Ok(Chunk {
            length,
            chunk_type,
            data: data.to_vec(),
            crc,
        })
    }
}

impl fmt::Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk_type::ChunkType;
    use std::str::FromStr;

    fn generate_chunk() -> Chunk {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        Chunk::try_from(chunk_data.as_ref()).unwrap()
    }

    #[test]
    fn test_new_chunk() {
        let chunk_type = ChunkType::from_str("RuSt").unwrap();
        let data = "This is where your secret message will be!"
            .as_bytes()
            .to_vec();
        let chunk = Chunk::new(chunk_type, data.clone()).unwrap();
        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.data(), &data);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_chunk_length() {
        let chunk = generate_chunk();
        assert_eq!(chunk.length(), 42);
    }

    #[test]
    fn test_chunk_type() {
        let chunk = generate_chunk();
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
    }

    #[test]
    fn test_chunk_string() {
        let chunk = generate_chunk();
        let chunk_string = chunk.to_string();
        let expected_chunk_string = String::from("This is where your secret message will be!");
        assert_eq!(chunk_string, expected_chunk_string);
    }

    #[test]
    fn test_chunk_crc() {
        let chunk = generate_chunk();
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_chunk_as_bytes() {
        let chunk = generate_chunk();
        // Serialize to network-order byte array.
        let chunk_bytes = chunk.as_bytes();

        let roundtrip = Chunk::try_from(chunk_bytes.as_slice()).unwrap();

        assert_eq!(chunk, roundtrip)
    }

    #[test]
    fn test_valid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref()).unwrap();

        let chunk_string = chunk.to_string();
        let expected_chunk_string = String::from("This is where your secret message will be!");

        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
        assert_eq!(chunk_string, expected_chunk_string);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_invalid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656333;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref());

        assert!(chunk.is_err());
    }

    #[test]
    fn test_chunk_trait_impls() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk: Chunk = TryFrom::try_from(chunk_data.as_ref()).unwrap();

        let _chunk_string = format!("{}", chunk);
    }
}
