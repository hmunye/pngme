//! PNG chunk types are four-byte sequences corresponding to readable labels.
//! The first four are termed critical chunks, which consist of:
//!
//! - IHDR: image header, which is the first chunk in a PNG datastream.
//! - PLTE: palette table associated with indexed PNG images.
//! - IDAT: image data chunks.
//! - IEND: image trailer, which is the last chunk in a PNG datastream.
//!
//! The remaining chunk types are termed ancillary chunk types, which encoders
//! may generate and decoders may interpret:
//!
//! - Transparency information: tRNS.
//! - Color space information: cHRM, gAMA, iCCP, sBIT, sRGB, cICP, mDCV.
//! - Textual information: iTXt, tEXt, zTXt.
//! - Miscellaneous information: bKGD, hIST, pHYs, sPLT, eXIf.
//! - Time information: tIME.
//! - Animation information: acTL, fcTL, fdAT.

use std::convert::TryFrom;
use std::str::{self, FromStr};
use std::{fmt, result};

/// Chunk type of a chunk within a PNG datastream.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ChunkType {
    inner: [u8; 4],
}

impl ChunkType {
    /// Returns the raw bytes of the chunk type.
    #[inline]
    pub const fn bytes(&self) -> [u8; 4] {
        self.inner
    }

    /// Returns `true` if the chunk type is valid.
    #[inline]
    pub fn is_valid(&self) -> bool {
        // Other validation occurs when constructing the `ChunkType`.
        self.is_reserved_bit_valid()
    }

    /// Returns `true` if the chunk type has the `critical` property bit set.
    #[inline]
    pub fn is_critical(&self) -> bool {
        ((self.inner[0] >> 5) & 0x01) == 0
    }

    /// Returns `true` if the chunk type has the `public` property bit set.
    #[inline]
    pub fn is_public(&self) -> bool {
        ((self.inner[1] >> 5) & 0x01) == 0
    }

    /// Returns `true` if the chunk type has a valid reserved bit according to  
    /// version 3.0 of the PNG specification.
    #[inline]
    pub fn is_reserved_bit_valid(&self) -> bool {
        ((self.inner[2] >> 5) & 0x01) == 0
    }

    /// Returns `true` if the chunk type has the `safe-to-copy` property bit
    /// set.
    #[inline]
    pub fn is_safe_to_copy(&self) -> bool {
        ((self.inner[3] >> 5) & 0x01) == 1
    }
}

impl TryFrom<[u8; 4]> for ChunkType {
    type Error = anyhow::Error;

    fn try_from(bytes: [u8; 4]) -> result::Result<Self, Self::Error> {
        // ASCII uppercase ('A' - 'Z') -> 0b01000001 - 0b01011010
        // ASCII lowercase ('a' - 'z') -> 0b01100001 - 0b01111010
        //                                    ^              ^
        //                                Difference only in bit 5
        for b in bytes {
            // Clear the 5th bit so the byte can be normalized.
            let upper = b & !0x20; // 0b1101_1111
            if !upper.is_ascii_uppercase() {
                anyhow::bail!("invalid PNG chunk type: invalid byte '{}'", b as char)
            }
        }

        Ok(ChunkType { inner: bytes })
    }
}

impl FromStr for ChunkType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> result::Result<Self, Self::Err> {
        anyhow::ensure!(
            s.len() == 4,
            "invalid PNG chunk type: must be exactly 4 bytes: \"{}\"",
            s
        );

        let mut iter = s.bytes();
        ChunkType::try_from(std::array::from_fn(|_| {
            // SAFETY: Previously checked for byte length of 4.
            iter.next()
                .expect("byte iterator must yield exactly 4 bytes")
        }))
    }
}

impl fmt::Display for ChunkType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bytes = self.bytes();
        // SAFETY: Each byte of `ChunkType` is restricted to the hexadecimal
        // values 41 to 5A (A-Z) and 61 to 7A (a-z).
        write!(f, "{}", unsafe { str::from_utf8_unchecked(&bytes) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use std::str::FromStr;

    #[test]
    fn test_chunk_type_from_bytes() {
        let expected = [82, 117, 83, 116];
        let actual = ChunkType::try_from([82, 117, 83, 116]).unwrap();

        assert_eq!(expected, actual.bytes());
    }

    #[test]
    fn test_chunk_type_from_str() {
        let expected = ChunkType::try_from([82, 117, 83, 116]).unwrap();
        let actual = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_chunk_type_is_critical() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_critical());
    }

    #[test]
    fn test_chunk_type_is_not_critical() {
        let chunk = ChunkType::from_str("ruSt").unwrap();
        assert!(!chunk.is_critical());
    }

    #[test]
    fn test_chunk_type_is_public() {
        let chunk = ChunkType::from_str("RUSt").unwrap();
        assert!(chunk.is_public());
    }

    #[test]
    fn test_chunk_type_is_not_public() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(!chunk.is_public());
    }

    #[test]
    fn test_chunk_type_is_reserved_bit_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_reserved_bit_valid());
    }

    #[test]
    fn test_chunk_type_is_reserved_bit_invalid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_reserved_bit_valid());
    }

    #[test]
    fn test_chunk_type_is_safe_to_copy() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_safe_to_copy());
    }

    #[test]
    fn test_chunk_type_is_unsafe_to_copy() {
        let chunk = ChunkType::from_str("RuST").unwrap();
        assert!(!chunk.is_safe_to_copy());
    }

    #[test]
    fn test_valid_chunk_is_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_valid());
    }

    #[test]
    fn test_invalid_chunk_is_valid() {
        // Third letter is not uppercase.
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_valid());

        // Contains byte outside the range (0x41..=0x5A) or (0x61..=0x7A).
        let chunk = ChunkType::from_str("Ru1t");
        assert!(chunk.is_err());
    }

    #[test]
    fn test_chunk_type_string() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(&chunk.to_string(), "RuSt");
    }

    #[test]
    fn test_chunk_type_trait_impls() {
        let chunk_type_1: ChunkType = TryFrom::try_from([82, 117, 83, 116]).unwrap();
        let chunk_type_2: ChunkType = FromStr::from_str("RuSt").unwrap();
        let _chunk_string = format!("{}", chunk_type_1);
        let _are_chunks_equal = chunk_type_1 == chunk_type_2;
    }
}
