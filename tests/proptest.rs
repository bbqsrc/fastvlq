use fastvint::*;
use proptest::prelude::*;

proptest! {
    #[test]
    fn roundtrip_u64(x: u64) {
        prop_assert_eq!(u64::from(Vu64::from(x)), x);
    }

    #[test]
    fn roundtrip_i64(x: i64) {
        prop_assert_eq!(i64::from(Vi64::from(x)), x);
    }

    #[test]
    fn roundtrip_u32(x: u32) {
        prop_assert_eq!(u32::from(Vu32::from(x)), x);
    }

    #[test]
    fn roundtrip_i32(x: i32) {
        prop_assert_eq!(i32::from(Vi32::from(x)), x);
    }

    #[test]
    fn roundtrip_u128(x: u128) {
        prop_assert_eq!(u128::from(Vu128::from(x)), x);
    }

    #[test]
    fn roundtrip_i128(x: i128) {
        prop_assert_eq!(i128::from(Vi128::from(x)), x);
    }
}
