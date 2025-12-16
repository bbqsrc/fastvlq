//! Async VLQ extension traits.

/// Extension trait for reading VLQ-encoded integers from an async reader.
pub trait AsyncReadVlqExt {
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

/// Extension trait for writing VLQ-encoded integers to an async writer.
pub trait AsyncWriteVlqExt {
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
