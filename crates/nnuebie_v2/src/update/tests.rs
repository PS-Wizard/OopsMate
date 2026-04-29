use super::{
    accum_add_into_both, accum_add_sub, accum_add1_sub2_into, accum_add2_sub1_into_both,
    psqt_add2_sub2_into, psqt_sub_into_both,
};
use crate::aligned::CacheAligned;

#[test]
fn vector_accum_update_matches_scalar_reference() {
    let base = CacheAligned::new(std::array::from_fn(|i| (i as i16) * 3 - 77));
    let add: CacheAligned<[i16; 128]> =
        CacheAligned::new(std::array::from_fn(|i| (i as i16) * 5 - 11));
    let sub0: CacheAligned<[i16; 128]> =
        CacheAligned::new(std::array::from_fn(|i| (i as i16) * 7 - 29));
    let sub1: CacheAligned<[i16; 128]> = CacheAligned::new(std::array::from_fn(|i| 101 - i as i16));
    let mut out = CacheAligned::new([0i16; 128]);
    let mut scalar = [0i16; 128];

    accum_add1_sub2_into(&base, &add[..], &sub0[..], &sub1[..], &mut out);
    for i in 0..128 {
        scalar[i] = base[i]
            .wrapping_add(add[i])
            .wrapping_sub(sub0[i])
            .wrapping_sub(sub1[i]);
    }
    assert_eq!(*out, scalar);

    let mut inplace = base.clone();
    let mut inplace_scalar = base.0;
    accum_add_sub(&mut inplace, &add[..], &sub0[..]);
    for i in 0..128 {
        inplace_scalar[i] = inplace_scalar[i].wrapping_add(add[i]).wrapping_sub(sub0[i]);
    }
    assert_eq!(*inplace, inplace_scalar);
}

#[test]
fn vector_psqt_update_matches_scalar_reference() {
    let base = CacheAligned::new([1, 2, 3, 4, 5, 6, 7, 8]);
    let add0 = CacheAligned::new([3, 1, -2, 5, 8, -1, 4, 7]);
    let add1 = CacheAligned::new([2, 6, 1, -3, 4, 0, 5, -2]);
    let sub0 = CacheAligned::new([-1, 2, 4, 3, -5, 7, 1, 9]);
    let sub1 = CacheAligned::new([8, -4, 3, 2, 1, 5, -6, 0]);
    let mut out = CacheAligned::new([0i32; 8]);
    let mut scalar = [0i32; 8];

    psqt_add2_sub2_into(&base, &add0[..], &add1[..], &sub0[..], &sub1[..], &mut out);
    for i in 0..8 {
        scalar[i] = base[i] + add0[i] + add1[i] - sub0[i] - sub1[i];
    }
    assert_eq!(*out, scalar);

    let mut both_base = base.clone();
    let mut both_out = CacheAligned::new([0i32; 8]);
    psqt_sub_into_both(&mut both_base, &sub0[..], &mut both_out);
    for i in 0..8 {
        scalar[i] = base[i] - sub0[i];
    }
    assert_eq!(*both_base, scalar);
    assert_eq!(*both_out, scalar);
}

#[test]
fn vector_dual_accum_update_matches_scalar_reference() {
    let mut base = CacheAligned::new(std::array::from_fn(|i| (i as i16) * 2 - 33));
    let add0: CacheAligned<[i16; 128]> =
        CacheAligned::new(std::array::from_fn(|i| (i as i16) * 3 - 17));
    let add1: CacheAligned<[i16; 128]> = CacheAligned::new(std::array::from_fn(|i| 91 - i as i16));
    let sub: CacheAligned<[i16; 128]> =
        CacheAligned::new(std::array::from_fn(|i| (i as i16) * 5 - 7));
    let mut out = CacheAligned::new([0i16; 128]);
    let mut scalar = [0i16; 128];

    accum_add2_sub1_into_both(&mut base, &add0[..], &add1[..], &sub[..], &mut out);
    for i in 0..128 {
        scalar[i] = ((i as i16) * 2 - 33)
            .wrapping_add(add0[i])
            .wrapping_add(add1[i])
            .wrapping_sub(sub[i]);
    }
    assert_eq!(*base, scalar);
    assert_eq!(*out, scalar);

    let mut add_base = CacheAligned::new(std::array::from_fn(|i| i as i16 - 64));
    accum_add_into_both(&mut add_base, &add0[..], &mut out);
    for i in 0..128 {
        scalar[i] = (i as i16 - 64).wrapping_add(add0[i]);
    }
    assert_eq!(*add_base, scalar);
    assert_eq!(*out, scalar);
}
