//! Async VLQ traits for tokio.

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::{decode_vu32, decode_vu64, decode_vu128, encode_vu32, encode_vu64, encode_vu128};
use crate::{vi32, vi64, vi128, vu32, vu64, vu128};

/// Extension trait for reading VLQ-encoded integers from a tokio async reader.
pub trait TokioReadVlqExt {
    /// Read a variable-length `u32` asynchronously.
    fn read_vu32(&mut self) -> impl core::future::Future<Output = std::io::Result<u32>>;
    /// Read a variable-length `i32` asynchronously.
    fn read_vi32(&mut self) -> impl core::future::Future<Output = std::io::Result<i32>>;
    /// Read a variable-length `u64` asynchronously.
    fn read_vu64(&mut self) -> impl core::future::Future<Output = std::io::Result<u64>>;
    /// Read a variable-length `i64` asynchronously.
    fn read_vi64(&mut self) -> impl core::future::Future<Output = std::io::Result<i64>>;
    /// Read a variable-length `u128` asynchronously.
    fn read_vu128(&mut self) -> impl core::future::Future<Output = std::io::Result<u128>>;
    /// Read a variable-length `i128` asynchronously.
    fn read_vi128(&mut self) -> impl core::future::Future<Output = std::io::Result<i128>>;
}

/// Extension trait for writing VLQ-encoded integers to a tokio async writer.
pub trait TokioWriteVlqExt {
    /// Write a variable-length `u32` asynchronously.
    fn write_vu32(&mut self, n: u32) -> impl core::future::Future<Output = std::io::Result<()>>;
    /// Write a variable-length `i32` asynchronously.
    fn write_vi32(&mut self, n: i32) -> impl core::future::Future<Output = std::io::Result<()>>;
    /// Write a variable-length `u64` asynchronously.
    fn write_vu64(&mut self, n: u64) -> impl core::future::Future<Output = std::io::Result<()>>;
    /// Write a variable-length `i64` asynchronously.
    fn write_vi64(&mut self, n: i64) -> impl core::future::Future<Output = std::io::Result<()>>;
    /// Write a variable-length `u128` asynchronously.
    fn write_vu128(&mut self, n: u128) -> impl core::future::Future<Output = std::io::Result<()>>;
    /// Write a variable-length `i128` asynchronously.
    fn write_vi128(&mut self, n: i128) -> impl core::future::Future<Output = std::io::Result<()>>;
}

impl<R: AsyncRead + Unpin> TokioReadVlqExt for R {
    async fn read_vu32(&mut self) -> std::io::Result<u32> {
        let mut buf = [0u8; vu32::VU32_BUF_SIZE];
        AsyncReadExt::read_exact(self, &mut buf[0..1]).await?;
        let len = vu32::decode_len_vu32(buf[0]) as usize;
        if len > 1 {
            AsyncReadExt::read_exact(self, &mut buf[1..len]).await?;
        }
        Ok(decode_vu32(vu32::Vu32(buf)))
    }

    async fn read_vi32(&mut self) -> std::io::Result<i32> {
        self.read_vu32().await.map(vi32::zigzag_decode_i32)
    }

    async fn read_vu64(&mut self) -> std::io::Result<u64> {
        let mut buf = [0u8; vu64::VU64_BUF_SIZE];
        AsyncReadExt::read_exact(self, &mut buf[0..1]).await?;
        let len = vu64::decode_len_vu64(buf[0]) as usize;
        if len > 1 {
            AsyncReadExt::read_exact(self, &mut buf[1..len]).await?;
        }
        Ok(decode_vu64(vu64::Vu64(buf)))
    }

    async fn read_vi64(&mut self) -> std::io::Result<i64> {
        self.read_vu64().await.map(vi64::zigzag_decode_i64)
    }

    async fn read_vu128(&mut self) -> std::io::Result<u128> {
        let mut buf = [0u8; vu128::VU128_BUF_SIZE];
        AsyncReadExt::read_exact(self, &mut buf[0..1]).await?;
        if buf[0] == 0 {
            AsyncReadExt::read_exact(self, &mut buf[1..2]).await?;
        }
        let len = vu128::decode_len_vu128(buf[0], buf[1]) as usize;
        if len > 2 {
            AsyncReadExt::read_exact(self, &mut buf[2..len]).await?;
        } else if len == 2 && buf[0] != 0 {
            AsyncReadExt::read_exact(self, &mut buf[1..2]).await?;
        }
        Ok(decode_vu128(vu128::Vu128(buf)))
    }

    async fn read_vi128(&mut self) -> std::io::Result<i128> {
        self.read_vu128().await.map(vi128::zigzag_decode_i128)
    }
}

impl<W: AsyncWrite + Unpin> TokioWriteVlqExt for W {
    async fn write_vu32(&mut self, n: u32) -> std::io::Result<()> {
        AsyncWriteExt::write_all(self, encode_vu32(n).as_slice()).await
    }

    async fn write_vi32(&mut self, n: i32) -> std::io::Result<()> {
        self.write_vu32(vi32::zigzag_encode_i32(n)).await
    }

    async fn write_vu64(&mut self, n: u64) -> std::io::Result<()> {
        AsyncWriteExt::write_all(self, encode_vu64(n).as_slice()).await
    }

    async fn write_vi64(&mut self, n: i64) -> std::io::Result<()> {
        self.write_vu64(vi64::zigzag_encode_i64(n)).await
    }

    async fn write_vu128(&mut self, n: u128) -> std::io::Result<()> {
        AsyncWriteExt::write_all(self, encode_vu128(n).as_slice()).await
    }

    async fn write_vi128(&mut self, n: i128) -> std::io::Result<()> {
        self.write_vu128(vi128::zigzag_encode_i128(n)).await
    }
}
