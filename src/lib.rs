//! ## Algorithm
//!
//! Fast VLQ is a variant of value-length quantity encoding, with a focus on encoding and
//! decoding speed. The total number of bytes can always be derived from the very first byte.
//!
//! Supported types:
//! - `Vu32` / `Vi32`: unsigned/signed 32-bit (max 5 bytes)
//! - `Vu64` / `Vi64`: unsigned/signed 64-bit (max 9 bytes)
//! - `Vu128` / `Vi128`: unsigned/signed 128-bit (max 18 bytes)
//!
//! Signed types use zigzag encoding for efficient storage of small absolute values.
//!
//! This crate does not enforce an invariant that a number may only have one representation,
//! which means that it is possible to encode `1` as, for example, both `0b1000_0001` and
//! `0b0100_0000_0000_0001`.
//!
//! ## Usage
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! fastvlq = "2"
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::cast_lossless)]
#![deny(missing_docs)]

#[cfg(any(feature = "async-futures", feature = "async-tokio"))]
mod ext;
#[cfg(feature = "async-futures")]
mod futures;
#[cfg(feature = "async-tokio")]
mod tokio;

#[macro_use]
mod macros;

mod vi128;
mod vi32;
mod vi64;
mod vu128;
mod vu32;
mod vu64;

#[cfg(feature = "std")]
use std::io::{Read, Result as IoResult, Write};

pub use vi32::{Vi32, decode_vi32, encode_vi32};
pub use vi64::{Vi64, decode_vi64, encode_vi64};
pub use vi128::{Vi128, decode_vi128, encode_vi128};
pub use vu32::{Vu32, decode_vu32, encode_vu32};
pub use vu64::{Vu64, decode_vu64, encode_vu64};
pub use vu128::{Vu128, decode_vu128, encode_vu128};

#[cfg(any(feature = "async-futures", feature = "async-tokio"))]
pub use ext::{AsyncReadVlqExt, AsyncWriteVlqExt};

#[cfg(feature = "std")]
/// Extension trait for reading VLQ-encoded integers from a reader.
pub trait ReadVlqExt {
    /// Read a variable-length `u32`.
    fn read_vu32(&mut self) -> IoResult<u32>;
    /// Read a variable-length `i32`.
    fn read_vi32(&mut self) -> IoResult<i32>;
    /// Read a variable-length `u64`.
    fn read_vu64(&mut self) -> IoResult<u64>;
    /// Read a variable-length `i64`.
    fn read_vi64(&mut self) -> IoResult<i64>;
    /// Read a variable-length `u128`.
    fn read_vu128(&mut self) -> IoResult<u128>;
    /// Read a variable-length `i128`.
    fn read_vi128(&mut self) -> IoResult<i128>;
}

#[cfg(feature = "std")]
/// Extension trait for writing VLQ-encoded integers to a writer.
pub trait WriteVlqExt {
    /// Write a variable-length `u32`.
    fn write_vu32(&mut self, n: u32) -> IoResult<()>;
    /// Write a variable-length `i32`.
    fn write_vi32(&mut self, n: i32) -> IoResult<()>;
    /// Write a variable-length `u64`.
    fn write_vu64(&mut self, n: u64) -> IoResult<()>;
    /// Write a variable-length `i64`.
    fn write_vi64(&mut self, n: i64) -> IoResult<()>;
    /// Write a variable-length `u128`.
    fn write_vu128(&mut self, n: u128) -> IoResult<()>;
    /// Write a variable-length `i128`.
    fn write_vi128(&mut self, n: i128) -> IoResult<()>;
}

#[cfg(feature = "std")]
impl<R: Read> ReadVlqExt for R {
    fn read_vu32(&mut self) -> IoResult<u32> {
        let mut buf = [0u8; vu32::VU32_BUF_SIZE];
        self.read_exact(&mut buf[0..1])?;
        let len = vu32::decode_len_vu32(buf[0]) as usize;
        if len > 1 {
            self.read_exact(&mut buf[1..len])?;
        }
        Ok(decode_vu32(vu32::Vu32(buf)))
    }

    fn read_vi32(&mut self) -> IoResult<i32> {
        self.read_vu32().map(vi32::zigzag_decode_i32)
    }

    fn read_vu64(&mut self) -> IoResult<u64> {
        let mut buf = [0u8; vu64::VU64_BUF_SIZE];
        self.read_exact(&mut buf[0..1])?;
        let len = vu64::decode_len_vu64(buf[0]) as usize;
        if len > 1 {
            self.read_exact(&mut buf[1..len])?;
        }
        Ok(decode_vu64(vu64::Vu64(buf)))
    }

    fn read_vi64(&mut self) -> IoResult<i64> {
        self.read_vu64().map(vi64::zigzag_decode_i64)
    }

    fn read_vu128(&mut self) -> IoResult<u128> {
        let mut buf = [0u8; vu128::VU128_BUF_SIZE];
        self.read_exact(&mut buf[0..1])?;
        // Need second byte to determine extended length
        if buf[0] == 0 {
            self.read_exact(&mut buf[1..2])?;
        }
        let len = vu128::decode_len_vu128(buf[0], buf[1]) as usize;
        if len > 2 {
            self.read_exact(&mut buf[2..len])?;
        } else if len == 2 && buf[0] != 0 {
            // Standard 2-byte (not extended), already read first byte
            self.read_exact(&mut buf[1..2])?;
        }
        Ok(decode_vu128(vu128::Vu128(buf)))
    }

    fn read_vi128(&mut self) -> IoResult<i128> {
        self.read_vu128().map(vi128::zigzag_decode_i128)
    }
}

#[cfg(feature = "std")]
impl<W: Write> WriteVlqExt for W {
    fn write_vu32(&mut self, n: u32) -> IoResult<()> {
        self.write_all(encode_vu32(n).as_slice())
    }

    fn write_vi32(&mut self, n: i32) -> IoResult<()> {
        self.write_vu32(vi32::zigzag_encode_i32(n))
    }

    fn write_vu64(&mut self, n: u64) -> IoResult<()> {
        self.write_all(encode_vu64(n).as_slice())
    }

    fn write_vi64(&mut self, n: i64) -> IoResult<()> {
        self.write_vu64(vi64::zigzag_encode_i64(n))
    }

    fn write_vu128(&mut self, n: u128) -> IoResult<()> {
        self.write_all(encode_vu128(n).as_slice())
    }

    fn write_vi128(&mut self, n: i128) -> IoResult<()> {
        self.write_vu128(vi128::zigzag_encode_i128(n))
    }
}
