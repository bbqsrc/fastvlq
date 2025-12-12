macro_rules! prefix {
    (1) => {
        0b1000_0000
    };
    (2) => {
        0b0100_0000
    };
    (3) => {
        0b0010_0000
    };
    (4) => {
        0b0001_0000
    };
    (5) => {
        0b0000_1000
    };
    (6) => {
        0b0000_0100
    };
    (7) => {
        0b0000_0010
    };
    (8) => {
        0b0000_0001
    };
    (9) => {
        0b0000_0000
    };
    (1, $target:expr) => {
        $target | 0b1000_0000
    };
    (2, $target:expr) => {
        (0b0011_1111 & $target) | 0b0100_0000
    };
    (3, $target:expr) => {
        (0b0001_1111 & $target) | 0b0010_0000
    };
    (4, $target:expr) => {
        (0b0000_1111 & $target) | 0b0001_0000
    };
    (5, $target:expr) => {
        (0b0000_0111 & $target) | 0b0000_1000
    };
    (6, $target:expr) => {
        (0b0000_0011 & $target) | 0b0000_0100
    };
    (7, $target:expr) => {
        (0b0000_0001 & $target) | 0b0000_0010
    };
    (8, $target:expr) => {
        0b0000_0001
    };
    (9, $target:expr) => {
        0b0000_0000
    };
}

macro_rules! unprefix {
    (1, $target:expr) => {
        $target & 0b0111_1111
    };
    (2, $target:expr) => {
        $target & 0b0011_1111
    };
    (3, $target:expr) => {
        $target & 0b0001_1111
    };
    (4, $target:expr) => {
        $target & 0b0000_1111
    };
    (5, $target:expr) => {
        $target & 0b0000_0111
    };
    (6, $target:expr) => {
        $target & 0b0000_0011
    };
    (7, $target:expr) => {
        $target & 0b0000_0001
    };
    (8, $target:expr) => {
        0b0000_0000
    };
    (9, $target:expr) => {
        0b0000_0000
    };
}

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
    (10) => {
        offset!(9) as u128 + (1u128 << 64)
    };
    (11) => {
        offset!(10) + (1u128 << 71)
    };
    (12) => {
        offset!(11) + (1u128 << 78)
    };
    (13) => {
        offset!(12) + (1u128 << 85)
    };
    (14) => {
        offset!(13) + (1u128 << 92)
    };
    (15) => {
        offset!(14) + (1u128 << 99)
    };
    (16) => {
        offset!(15) + (1u128 << 106)
    };
    (17) => {
        offset!(16) + (1u128 << 113)
    };
}

macro_rules! encode_offset {
    (2, $n:tt) => {
        $n as u16 - offset!(2)
    };
    (3, $n:tt) => {
        ($n as u32 - offset!(3)) << 8
    };
    (4, $n:tt) => {
        ($n as u32 - offset!(4))
    };
    (5, $n:tt) => {
        ($n as u64 - offset!(5)) << (8 * 3)
    };
    (6, $n:tt) => {
        ($n as u64 - offset!(6)) << (8 * 2)
    };
    (7, $n:tt) => {
        ($n as u64 - offset!(7)) << 8
    };
    (8, $n:tt) => {
        ($n as u64 - offset!(8))
    };
    (9, $n:tt) => {
        ($n as u64 - offset!(9))
    };
}

macro_rules! copy_from_slice_offset {
    (source = $source:ident, dest = $dest:ident, offset = $offset:tt) => {
        let mut i = 0;
        while i < $offset {
            $dest[i] = $source[i];
            i += 1;
        }
    };
}
