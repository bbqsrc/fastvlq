use fastvlq::*;
use proptest::prelude::*;

proptest! {
    #[test]
    fn roundtrip_u64_be(x: u64) {
        prop_assert_eq!(u64::from(Vu64::<BE>::from(x)), x);
    }

    #[test]
    fn roundtrip_u64_le(x: u64) {
        prop_assert_eq!(u64::from(Vu64::<LE>::from(x)), x);
    }

    #[test]
    fn roundtrip_i64_be(x: i64) {
        prop_assert_eq!(i64::from(Vi64::<BE>::from(x)), x);
    }

    #[test]
    fn roundtrip_i64_le(x: i64) {
        prop_assert_eq!(i64::from(Vi64::<LE>::from(x)), x);
    }

    #[test]
    fn roundtrip_u32_be(x: u32) {
        prop_assert_eq!(u32::from(Vu32::<BE>::from(x)), x);
    }

    #[test]
    fn roundtrip_u32_le(x: u32) {
        prop_assert_eq!(u32::from(Vu32::<LE>::from(x)), x);
    }

    #[test]
    fn roundtrip_i32_be(x: i32) {
        prop_assert_eq!(i32::from(Vi32::<BE>::from(x)), x);
    }

    #[test]
    fn roundtrip_i32_le(x: i32) {
        prop_assert_eq!(i32::from(Vi32::<LE>::from(x)), x);
    }

    #[test]
    fn roundtrip_u128_be(x: u128) {
        prop_assert_eq!(u128::from(Vu128::<BE>::from(x)), x);
    }

    #[test]
    fn roundtrip_u128_le(x: u128) {
        prop_assert_eq!(u128::from(Vu128::<LE>::from(x)), x);
    }

    #[test]
    fn roundtrip_i128_be(x: i128) {
        prop_assert_eq!(i128::from(Vi128::<BE>::from(x)), x);
    }

    #[test]
    fn roundtrip_i128_le(x: i128) {
        prop_assert_eq!(i128::from(Vi128::<LE>::from(x)), x);
    }
}
