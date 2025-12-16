#[cfg(not(any(
    all(target_arch = "aarch64", feature = "asm"),
    all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm")
)))]
macro_rules! offset {
    (1) => {
        0
    };
    (2) => {
        1 << 7
    };
    (3) => {
        offset!(2) as u32 + (1 << 14)
    };
    (4) => {
        offset!(3) as u32 + (1 << 21)
    };
    (5) => {
        offset!(4) as u64 + (1 << 28)
    };
    (6) => {
        offset!(5) + (1 << 35)
    };
    (7) => {
        offset!(6) + (1 << 42)
    };
    (8) => {
        offset!(7) + (1 << 49)
    };
    (9) => {
        offset!(8) + (1 << 56)
    };
    // Extended offsets for u128 (lengths 10-17)
    // Capacity: 10=70, 11=77, 12=84, 13=91, 14=98, 15=105, 16=112 bits
    (10) => {
        offset!(9) as u128 + (1u128 << 64)
    };
    (11) => {
        offset!(10) + (1u128 << 70)
    };
    (12) => {
        offset!(11) + (1u128 << 77)
    };
    (13) => {
        offset!(12) + (1u128 << 84)
    };
    (14) => {
        offset!(13) + (1u128 << 91)
    };
    (15) => {
        offset!(14) + (1u128 << 98)
    };
    (16) => {
        offset!(15) + (1u128 << 105)
    };
    (17) => {
        offset!(16) + (1u128 << 112)
    };
}
