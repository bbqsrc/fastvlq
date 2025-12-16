//! Async VLQ trait implementations for futures-io.

use core::marker::PhantomData;

use futures_io::{AsyncRead, AsyncWrite};
use futures_util::{AsyncReadExt, AsyncWriteExt};

use crate::ext::{AsyncReadVlqExt, AsyncWriteVlqExt};
use crate::{
    decode_vu32_be, decode_vu32_le, decode_vu64_be, decode_vu64_le, decode_vu128_be,
    decode_vu128_le, encode_vu32_be, encode_vu32_le, encode_vu64_be, encode_vu64_le,
    encode_vu128_be, encode_vu128_le,
};
use crate::{vi32, vi64, vi128, vu32, vu64, vu128};

impl<R: AsyncRead + Unpin> AsyncReadVlqExt for R {
    async fn read_vu32_be(&mut self) -> std::io::Result<u32> {
        let mut buf = [0u8; vu32::VU32_BUF_SIZE];
        AsyncReadExt::read_exact(self, &mut buf[0..1]).await?;
        let len = vu32::decode_len_vu32(buf[0]) as usize;
        if len > 1 {
            AsyncReadExt::read_exact(self, &mut buf[1..len]).await?;
        }
        Ok(decode_vu32_be(vu32::Vu32(buf, PhantomData)))
    }

    async fn read_vu32_le(&mut self) -> std::io::Result<u32> {
        let mut buf = [0u8; vu32::VU32_BUF_SIZE];
        AsyncReadExt::read_exact(self, &mut buf[0..1]).await?;
        let len = vu32::decode_len_vu32(buf[0]) as usize;
        if len > 1 {
            AsyncReadExt::read_exact(self, &mut buf[1..len]).await?;
        }
        Ok(decode_vu32_le(vu32::Vu32(buf, PhantomData)))
    }

    async fn read_vi32_be(&mut self) -> std::io::Result<i32> {
        self.read_vu32_be().await.map(vi32::zigzag_decode_i32)
    }

    async fn read_vi32_le(&mut self) -> std::io::Result<i32> {
        self.read_vu32_le().await.map(vi32::zigzag_decode_i32)
    }

    async fn read_vu64_be(&mut self) -> std::io::Result<u64> {
        let mut buf = [0u8; vu64::VU64_BUF_SIZE];
        AsyncReadExt::read_exact(self, &mut buf[0..1]).await?;
        let prefix = buf[0];
        let len = vu64::decode_len_vu64(prefix) as usize;
        if len > 1 {
            AsyncReadExt::read_exact(self, &mut buf[1..len]).await?;
        }
        let mut data_buf = [0u8; 8];
        if len > 1 {
            data_buf[..(len - 1)].copy_from_slice(&buf[1..len]);
        }
        let packed = u64::from_be_bytes(data_buf);
        Ok(decode_vu64_be(vu64::Vu64(prefix, packed, PhantomData)))
    }

    async fn read_vu64_le(&mut self) -> std::io::Result<u64> {
        let mut buf = [0u8; vu64::VU64_BUF_SIZE];
        AsyncReadExt::read_exact(self, &mut buf[0..1]).await?;
        let prefix = buf[0];
        let len = vu64::decode_len_vu64(prefix) as usize;
        if len > 1 {
            AsyncReadExt::read_exact(self, &mut buf[1..len]).await?;
        }
        let mut data_buf = [0u8; 8];
        if len > 1 {
            data_buf[..(len - 1)].copy_from_slice(&buf[1..len]);
        }
        let packed = u64::from_le_bytes(data_buf);
        Ok(decode_vu64_le(vu64::Vu64(prefix, packed, PhantomData)))
    }

    async fn read_vi64_be(&mut self) -> std::io::Result<i64> {
        self.read_vu64_be().await.map(vi64::zigzag_decode_i64)
    }

    async fn read_vi64_le(&mut self) -> std::io::Result<i64> {
        self.read_vu64_le().await.map(vi64::zigzag_decode_i64)
    }

    async fn read_vu128_be(&mut self) -> std::io::Result<u128> {
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
        Ok(decode_vu128_be(vu128::Vu128(buf, PhantomData)))
    }

    async fn read_vu128_le(&mut self) -> std::io::Result<u128> {
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
        Ok(decode_vu128_le(vu128::Vu128(buf, PhantomData)))
    }

    async fn read_vi128_be(&mut self) -> std::io::Result<i128> {
        self.read_vu128_be().await.map(vi128::zigzag_decode_i128)
    }

    async fn read_vi128_le(&mut self) -> std::io::Result<i128> {
        self.read_vu128_le().await.map(vi128::zigzag_decode_i128)
    }
}

impl<W: AsyncWrite + Unpin> AsyncWriteVlqExt for W {
    async fn write_vu32_be(&mut self, n: u32) -> std::io::Result<()> {
        AsyncWriteExt::write_all(self, encode_vu32_be(n).as_slice()).await
    }

    async fn write_vu32_le(&mut self, n: u32) -> std::io::Result<()> {
        AsyncWriteExt::write_all(self, encode_vu32_le(n).as_slice()).await
    }

    async fn write_vi32_be(&mut self, n: i32) -> std::io::Result<()> {
        self.write_vu32_be(vi32::zigzag_encode_i32(n)).await
    }

    async fn write_vi32_le(&mut self, n: i32) -> std::io::Result<()> {
        self.write_vu32_le(vi32::zigzag_encode_i32(n)).await
    }

    async fn write_vu64_be(&mut self, n: u64) -> std::io::Result<()> {
        let v = encode_vu64_be(n);
        AsyncWriteExt::write_all(self, &v.bytes()[..v.len() as usize]).await
    }

    async fn write_vu64_le(&mut self, n: u64) -> std::io::Result<()> {
        let v = encode_vu64_le(n);
        AsyncWriteExt::write_all(self, &v.bytes()[..v.len() as usize]).await
    }

    async fn write_vi64_be(&mut self, n: i64) -> std::io::Result<()> {
        self.write_vu64_be(vi64::zigzag_encode_i64(n)).await
    }

    async fn write_vi64_le(&mut self, n: i64) -> std::io::Result<()> {
        self.write_vu64_le(vi64::zigzag_encode_i64(n)).await
    }

    async fn write_vu128_be(&mut self, n: u128) -> std::io::Result<()> {
        AsyncWriteExt::write_all(self, encode_vu128_be(n).as_slice()).await
    }

    async fn write_vu128_le(&mut self, n: u128) -> std::io::Result<()> {
        AsyncWriteExt::write_all(self, encode_vu128_le(n).as_slice()).await
    }

    async fn write_vi128_be(&mut self, n: i128) -> std::io::Result<()> {
        self.write_vu128_be(vi128::zigzag_encode_i128(n)).await
    }

    async fn write_vi128_le(&mut self, n: i128) -> std::io::Result<()> {
        self.write_vu128_le(vi128::zigzag_encode_i128(n)).await
    }
}
