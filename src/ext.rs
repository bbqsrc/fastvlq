//! Async VLQ extension traits.

/// Extension trait for reading VLQ-encoded integers from an async reader.
pub trait AsyncReadVlqExt {
    /// Read a big-endian variable-length `u32` asynchronously.
    fn read_vu32_be(&mut self) -> impl core::future::Future<Output = std::io::Result<u32>>;
    /// Read a little-endian variable-length `u32` asynchronously.
    fn read_vu32_le(&mut self) -> impl core::future::Future<Output = std::io::Result<u32>>;
    /// Read a big-endian variable-length `i32` asynchronously.
    fn read_vi32_be(&mut self) -> impl core::future::Future<Output = std::io::Result<i32>>;
    /// Read a little-endian variable-length `i32` asynchronously.
    fn read_vi32_le(&mut self) -> impl core::future::Future<Output = std::io::Result<i32>>;
    /// Read a big-endian variable-length `u64` asynchronously.
    fn read_vu64_be(&mut self) -> impl core::future::Future<Output = std::io::Result<u64>>;
    /// Read a little-endian variable-length `u64` asynchronously.
    fn read_vu64_le(&mut self) -> impl core::future::Future<Output = std::io::Result<u64>>;
    /// Read a big-endian variable-length `i64` asynchronously.
    fn read_vi64_be(&mut self) -> impl core::future::Future<Output = std::io::Result<i64>>;
    /// Read a little-endian variable-length `i64` asynchronously.
    fn read_vi64_le(&mut self) -> impl core::future::Future<Output = std::io::Result<i64>>;
    /// Read a big-endian variable-length `u128` asynchronously.
    fn read_vu128_be(&mut self) -> impl core::future::Future<Output = std::io::Result<u128>>;
    /// Read a little-endian variable-length `u128` asynchronously.
    fn read_vu128_le(&mut self) -> impl core::future::Future<Output = std::io::Result<u128>>;
    /// Read a big-endian variable-length `i128` asynchronously.
    fn read_vi128_be(&mut self) -> impl core::future::Future<Output = std::io::Result<i128>>;
    /// Read a little-endian variable-length `i128` asynchronously.
    fn read_vi128_le(&mut self) -> impl core::future::Future<Output = std::io::Result<i128>>;
}

/// Extension trait for writing VLQ-encoded integers to an async writer.
pub trait AsyncWriteVlqExt {
    /// Write a big-endian variable-length `u32` asynchronously.
    fn write_vu32_be(&mut self, n: u32) -> impl core::future::Future<Output = std::io::Result<()>>;
    /// Write a little-endian variable-length `u32` asynchronously.
    fn write_vu32_le(&mut self, n: u32) -> impl core::future::Future<Output = std::io::Result<()>>;
    /// Write a big-endian variable-length `i32` asynchronously.
    fn write_vi32_be(&mut self, n: i32) -> impl core::future::Future<Output = std::io::Result<()>>;
    /// Write a little-endian variable-length `i32` asynchronously.
    fn write_vi32_le(&mut self, n: i32) -> impl core::future::Future<Output = std::io::Result<()>>;
    /// Write a big-endian variable-length `u64` asynchronously.
    fn write_vu64_be(&mut self, n: u64) -> impl core::future::Future<Output = std::io::Result<()>>;
    /// Write a little-endian variable-length `u64` asynchronously.
    fn write_vu64_le(&mut self, n: u64) -> impl core::future::Future<Output = std::io::Result<()>>;
    /// Write a big-endian variable-length `i64` asynchronously.
    fn write_vi64_be(&mut self, n: i64) -> impl core::future::Future<Output = std::io::Result<()>>;
    /// Write a little-endian variable-length `i64` asynchronously.
    fn write_vi64_le(&mut self, n: i64) -> impl core::future::Future<Output = std::io::Result<()>>;
    /// Write a big-endian variable-length `u128` asynchronously.
    fn write_vu128_be(
        &mut self,
        n: u128,
    ) -> impl core::future::Future<Output = std::io::Result<()>>;
    /// Write a little-endian variable-length `u128` asynchronously.
    fn write_vu128_le(
        &mut self,
        n: u128,
    ) -> impl core::future::Future<Output = std::io::Result<()>>;
    /// Write a big-endian variable-length `i128` asynchronously.
    fn write_vi128_be(
        &mut self,
        n: i128,
    ) -> impl core::future::Future<Output = std::io::Result<()>>;
    /// Write a little-endian variable-length `i128` asynchronously.
    fn write_vi128_le(
        &mut self,
        n: i128,
    ) -> impl core::future::Future<Output = std::io::Result<()>>;
}
