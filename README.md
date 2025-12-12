# fastvlq

[![Actions Status](https://github.com/bbqsrc/fastvlq/workflows/CI/badge.svg)](https://github.com/bbqsrc/fastvlq/actions)
[![Documentation](https://docs.rs/fastvlq/badge.svg)](https://docs.rs/fastvlq)

A fast variant of [variable-length quantity](https://en.wikipedia.org/wiki/Variable-length_quantity) encoding with a focus on speed and `no_std` support.

The algorithm uses leading zeros in the first byte to determine how many bytes are required for decoding, allowing the length to be known immediately without parsing the entire value.

## Supported Types

| Type | Max Bytes |
|------|-----------|
| `Vu32` | 5 |
| `Vi32` | 5 |
| `Vu64` | 9 |
| `Vi64` | 9 |
| `Vu128` | 18 |
| `Vi128` | 18 |

Signed types (`Vi*`) use zigzag encoding for efficient storage of small absolute values.

## Vu64 Compression

| Bytes | Min | Max |
|-------|-----|-----|
| 1 | 0 | 127 (0x7F) |
| 2 | 128 (0x80) | 16,511 (0x407F) |
| 3 | 16,512 (0x4080) | 2,113,663 (0x20407F) |
| 4 | 2,113,664 (0x204080) | 270,549,119 (0x1020407F) |
| 5 | 270,549,120 (0x10204080) | 34,630,287,487 (0x81020407F) |
| 6 | 34,630,287,488 (0x810204080) | 4,432,676,798,591 (0x4081020407F) |
| 7 | 4,432,676,798,592 (0x40810204080) | 567,382,630,219,903 (0x204081020407F) |
| 8 | 567,382,630,219,904 (0x2040810204080) | 72,624,976,668,147,839 (0x10204081020407F) |
| 9 | 72,624,976,668,147,840 (0x102040810204080) | 18,446,744,073,709,551,615 (0xFFFFFFFFFFFFFFFF) |

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
fastvlq = "2"
```

## Features

- `std` (default) - Enables `Read`/`Write` extension traits
- `async` - Enables async `Read`/`Write` extension traits via `futures-io`

## Where is this used?

* [box](https://github.com/bbqsrc/box) - a modern replacement for the zip file format

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
