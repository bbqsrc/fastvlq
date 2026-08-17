//! ## Algorithm
//!
//! fastvint is a fast variable-length integer encoding, with a focus on encoding and
//! decoding speed. The total number of bytes can always be derived from the very first byte.
//!
//! Supported types:
//! - `Vu32` / `Vi32`: unsigned/signed 32-bit (max 5 bytes)
//! - `Vu64` / `Vi64`: unsigned/signed 64-bit (max 9 bytes)
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
//! fastvint = "1"
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::cast_lossless)]
#![deny(missing_docs)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(any(feature = "async-futures", feature = "async-tokio"))]
mod ext;
#[cfg(all(feature = "async-futures", not(feature = "async-tokio")))]
mod futures;
#[cfg(feature = "async-tokio")]
mod tokio;

#[macro_use]
mod macros;

mod vi32;
mod vi64;
mod vu32;
mod vu64;

pub mod batch;

#[cfg(any(test, feature = "bench"))]
#[doc(hidden)]
pub mod uleb128;

#[cfg(any(test, feature = "bench"))]
#[doc(hidden)]
pub mod ileb128;

#[cfg(feature = "std")]
use std::io::{Read, Result as IoResult, Write};

// Unsigned types
pub use vu32::{Vu32, decode_vu32, decode_vu32_slice, encode_vu32};
pub use vu64::{Vu64, decode_vu64, decode_vu64_slice, encode_vu64};

// Batch encoding
pub use batch::encode_vu64_batch;
#[cfg(feature = "alloc")]
pub use batch::encode_vu64_batch_alloc;

// Signed types
pub use vi32::{
    Vi32, decode_vi32, decode_vi32_slice, encode_vi32, zigzag_decode_i32, zigzag_encode_i32,
};
pub use vi64::{
    Vi64, decode_vi64, decode_vi64_slice, encode_vi64, zigzag_decode_i64, zigzag_encode_i64,
};

#[cfg(any(feature = "async-futures", feature = "async-tokio"))]
pub use ext::{AsyncReadVintExt, AsyncWriteVintExt};

#[cfg(feature = "std")]
/// Extension trait for reading VLQ-encoded integers from a reader.
pub trait ReadVintExt {
    /// Read a variable-length `u32`.
    fn read_vu32(&mut self) -> IoResult<u32>;
    /// Read a variable-length `i32`.
    fn read_vi32(&mut self) -> IoResult<i32>;
    /// Read a variable-length `u64`.
    fn read_vu64(&mut self) -> IoResult<u64>;
    /// Read a variable-length `i64`.
    fn read_vi64(&mut self) -> IoResult<i64>;
}

#[cfg(feature = "std")]
/// Extension trait for writing VLQ-encoded integers to a writer.
pub trait WriteVintExt {
    /// Write a variable-length `u32`.
    fn write_vu32(&mut self, n: u32) -> IoResult<()>;
    /// Write a variable-length `i32`.
    fn write_vi32(&mut self, n: i32) -> IoResult<()>;
    /// Write a variable-length `u64`.
    fn write_vu64(&mut self, n: u64) -> IoResult<()>;
    /// Write a variable-length `i64`.
    fn write_vi64(&mut self, n: i64) -> IoResult<()>;
}

#[cfg(feature = "std")]
impl<R: Read> ReadVintExt for R {
    fn read_vu32(&mut self) -> IoResult<u32> {
        let mut buf = [0u8; vu32::VU32_BUF_SIZE];
        self.read_exact(&mut buf[0..1])?;
        let len = vu32::decode_len_vu32(buf[0]) as usize;
        if len > 1 {
            self.read_exact(&mut buf[1..len])?;
        }
        Ok(decode_vu32_slice(&buf[..len]).0)
    }

    fn read_vi32(&mut self) -> IoResult<i32> {
        self.read_vu32().map(zigzag_decode_i32)
    }

    fn read_vu64(&mut self) -> IoResult<u64> {
        let mut buf = [0u8; vu64::VU64_BUF_SIZE];
        self.read_exact(&mut buf[0..1])?;
        let len = vu64::decode_len_vu64(buf[0]) as usize;
        if len > 1 {
            self.read_exact(&mut buf[1..len])?;
        }
        Ok(decode_vu64_slice(&buf[..len]).0)
    }

    fn read_vi64(&mut self) -> IoResult<i64> {
        self.read_vu64().map(zigzag_decode_i64)
    }
}

#[cfg(feature = "std")]
impl<W: Write> WriteVintExt for W {
    fn write_vu32(&mut self, n: u32) -> IoResult<()> {
        let v = encode_vu32(n);
        self.write_all(&v.bytes()[..v.len() as usize])
    }

    fn write_vi32(&mut self, n: i32) -> IoResult<()> {
        self.write_vu32(zigzag_encode_i32(n))
    }

    fn write_vu64(&mut self, n: u64) -> IoResult<()> {
        let v = encode_vu64(n);
        self.write_all(&v.bytes()[..v.len() as usize])
    }

    fn write_vi64(&mut self, n: i64) -> IoResult<()> {
        self.write_vu64(zigzag_encode_i64(n))
    }
}
