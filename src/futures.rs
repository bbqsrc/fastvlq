//! Async VLQ trait implementations for futures-io.

use futures_io::{AsyncRead, AsyncWrite};
use futures_util::{AsyncReadExt, AsyncWriteExt};

use crate::ext::{AsyncReadVintExt, AsyncWriteVintExt};
use crate::{decode_vu32_slice, decode_vu64_slice, encode_vu32, encode_vu64};
use crate::{vi32, vi64, vu32, vu64};

impl<R: AsyncRead + Unpin> AsyncReadVintExt for R {
    async fn read_vu32(&mut self) -> std::io::Result<u32> {
        let mut buf = [0u8; vu32::VU32_BUF_SIZE];
        AsyncReadExt::read_exact(self, &mut buf[0..1]).await?;
        let len = vu32::decode_len_vu32(buf[0]) as usize;
        if len > 1 {
            AsyncReadExt::read_exact(self, &mut buf[1..len]).await?;
        }
        Ok(decode_vu32_slice(&buf[..len]).0)
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
        Ok(decode_vu64_slice(&buf[..len]).0)
    }

    async fn read_vi64(&mut self) -> std::io::Result<i64> {
        self.read_vu64().await.map(vi64::zigzag_decode_i64)
    }
}

impl<W: AsyncWrite + Unpin> AsyncWriteVintExt for W {
    async fn write_vu32(&mut self, n: u32) -> std::io::Result<()> {
        let v = encode_vu32(n);
        AsyncWriteExt::write_all(self, &v.bytes()[..v.len() as usize]).await
    }

    async fn write_vi32(&mut self, n: i32) -> std::io::Result<()> {
        self.write_vu32(vi32::zigzag_encode_i32(n)).await
    }

    async fn write_vu64(&mut self, n: u64) -> std::io::Result<()> {
        let v = encode_vu64(n);
        AsyncWriteExt::write_all(self, &v.bytes()[..v.len() as usize]).await
    }

    async fn write_vi64(&mut self, n: i64) -> std::io::Result<()> {
        self.write_vu64(vi64::zigzag_encode_i64(n)).await
    }
}
