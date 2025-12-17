//! Async VLQ trait implementations for futures-io.

use futures_io::{AsyncRead, AsyncWrite};
use futures_util::{AsyncReadExt, AsyncWriteExt};

use crate::ext::{AsyncReadVintExt, AsyncWriteVintExt};
use crate::{decode_vu32, decode_vu64, decode_vu128, encode_vu32, encode_vu64, encode_vu128};
use crate::{vi32, vi64, vi128, vu32, vu64, vu128};

impl<R: AsyncRead + Unpin> AsyncReadVintExt for R {
    async fn read_vu32(&mut self) -> std::io::Result<u32> {
        let mut buf = [0u8; vu32::VU32_BUF_SIZE];
        AsyncReadExt::read_exact(self, &mut buf[0..1]).await?;
        let prefix = buf[0];
        let len = vu32::decode_len_vu32(prefix) as usize;
        if len > 1 {
            AsyncReadExt::read_exact(self, &mut buf[1..len]).await?;
        }
        let mut data_buf = [0u8; 4];
        data_buf[..(len - 1).min(4)].copy_from_slice(&buf[1..len]);
        let data = u32::from_le_bytes(data_buf);
        Ok(decode_vu32(vu32::Vu32(prefix, data)))
    }

    async fn read_vi32(&mut self) -> std::io::Result<i32> {
        self.read_vu32().await.map(vi32::zigzag_decode_i32)
    }

    async fn read_vu64(&mut self) -> std::io::Result<u64> {
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
        Ok(decode_vu64(vu64::Vu64(prefix, packed)))
    }

    async fn read_vi64(&mut self) -> std::io::Result<i64> {
        self.read_vu64().await.map(vi64::zigzag_decode_i64)
    }

    async fn read_vu128(&mut self) -> std::io::Result<u128> {
        let mut buf = [0u8; vu128::VU128_BUF_SIZE];
        AsyncReadExt::read_exact(self, &mut buf[0..1]).await?;
        let p1 = buf[0];

        if p1 == 0 {
            // Extended format (10-18 bytes) - need second byte for length
            AsyncReadExt::read_exact(self, &mut buf[1..2]).await?;
            let p2 = buf[1];
            let len = vu128::decode_len_vu128(p1, p2) as usize;
            if len > 2 {
                AsyncReadExt::read_exact(self, &mut buf[2..len]).await?;
            }
            let mut data_buf = [0u8; 16];
            if len > 2 {
                data_buf[..(len - 2)].copy_from_slice(&buf[2..len]);
            }
            let data = u128::from_le_bytes(data_buf);
            Ok(decode_vu128(vu128::Vu128(p1, p2, data)))
        } else {
            // Standard format (len 1-8) - data goes in self.2
            let len = vu128::decode_len_vu128(p1, 0) as usize;
            if len > 1 {
                AsyncReadExt::read_exact(self, &mut buf[1..len]).await?;
            }
            let mut data_buf = [0u8; 16];
            if len > 1 {
                data_buf[..(len - 1)].copy_from_slice(&buf[1..len]);
            }
            let packed = u128::from_le_bytes(data_buf);
            Ok(decode_vu128(vu128::Vu128(p1, 0, packed)))
        }
    }

    async fn read_vi128(&mut self) -> std::io::Result<i128> {
        self.read_vu128().await.map(vi128::zigzag_decode_i128)
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

    async fn write_vu128(&mut self, n: u128) -> std::io::Result<()> {
        let v = encode_vu128(n);
        AsyncWriteExt::write_all(self, &v.bytes()[..v.len() as usize]).await
    }

    async fn write_vi128(&mut self, n: i128) -> std::io::Result<()> {
        self.write_vu128(vi128::zigzag_encode_i128(n)).await
    }
}
