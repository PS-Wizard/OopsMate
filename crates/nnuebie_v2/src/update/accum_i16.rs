use super::simd::{
    can_vectorize_i16, can_vectorize_i16_3, can_vectorize_i16_4, can_vectorize_i16_5,
    can_vectorize_i16_6,
};
use crate::simd256::{add_i16x16, load_i16x16, store_i16x16, sub_i16x16};

#[inline(always)]
pub(crate) fn accum_add<const N: usize>(acc: &mut [i16; N], add: &[i16]) {
    debug_assert_eq!(add.len(), N);
    debug_assert!(can_vectorize_i16(acc.as_ptr(), add.as_ptr(), N));
    #[cfg(target_arch = "x86_64")]
    unsafe {
        accum_add_avx2(acc, add);
    }
}

#[inline(always)]
pub(crate) fn accum_sub<const N: usize>(acc: &mut [i16; N], sub: &[i16]) {
    debug_assert_eq!(sub.len(), N);
    debug_assert!(can_vectorize_i16(acc.as_ptr(), sub.as_ptr(), N));
    #[cfg(target_arch = "x86_64")]
    unsafe {
        accum_sub_avx2(acc, sub);
    }
}

#[inline(always)]
pub(crate) fn accum_add_sub<const N: usize>(acc: &mut [i16; N], add: &[i16], sub: &[i16]) {
    debug_assert_eq!(add.len(), N);
    debug_assert_eq!(sub.len(), N);
    debug_assert!(can_vectorize_i16_3(
        acc.as_ptr(),
        add.as_ptr(),
        sub.as_ptr(),
        N
    ));
    #[cfg(target_arch = "x86_64")]
    unsafe {
        accum_add_sub_avx2(acc, add, sub);
    }
}

#[inline(always)]
pub(crate) fn accum_add_into_both<const N: usize>(
    base: &mut [i16; N],
    add: &[i16],
    out: &mut [i16; N],
) {
    debug_assert_eq!(add.len(), N);
    debug_assert!(can_vectorize_i16_3(
        base.as_ptr(),
        add.as_ptr(),
        out.as_ptr(),
        N
    ));
    #[cfg(target_arch = "x86_64")]
    unsafe {
        accum_add_into_both_avx2(base, add, out);
    }
}

#[inline(always)]
pub(crate) fn accum_sub_into_both<const N: usize>(
    base: &mut [i16; N],
    sub: &[i16],
    out: &mut [i16; N],
) {
    debug_assert_eq!(sub.len(), N);
    debug_assert!(can_vectorize_i16_3(
        base.as_ptr(),
        sub.as_ptr(),
        out.as_ptr(),
        N
    ));
    #[cfg(target_arch = "x86_64")]
    unsafe {
        accum_sub_into_both_avx2(base, sub, out);
    }
}

#[inline(always)]
pub(crate) fn accum_add1_sub1_into<const N: usize>(
    base: &[i16; N],
    add: &[i16],
    sub: &[i16],
    out: &mut [i16; N],
) {
    debug_assert_eq!(add.len(), N);
    debug_assert_eq!(sub.len(), N);
    debug_assert!(can_vectorize_i16_4(
        base.as_ptr(),
        add.as_ptr(),
        sub.as_ptr(),
        out.as_ptr(),
        N
    ));
    #[cfg(target_arch = "x86_64")]
    unsafe {
        accum_add1_sub1_into_avx2(base, add, sub, out);
    }
}

#[inline(always)]
pub(crate) fn accum_add1_sub1_into_both<const N: usize>(
    base: &mut [i16; N],
    add: &[i16],
    sub: &[i16],
    out: &mut [i16; N],
) {
    debug_assert_eq!(add.len(), N);
    debug_assert_eq!(sub.len(), N);
    debug_assert!(can_vectorize_i16_4(
        base.as_ptr(),
        add.as_ptr(),
        sub.as_ptr(),
        out.as_ptr(),
        N
    ));
    #[cfg(target_arch = "x86_64")]
    unsafe {
        accum_add1_sub1_into_both_avx2(base, add, sub, out);
    }
}

#[inline(always)]
pub(crate) fn accum_add1_sub2_into<const N: usize>(
    base: &[i16; N],
    add: &[i16],
    sub0: &[i16],
    sub1: &[i16],
    out: &mut [i16; N],
) {
    debug_assert_eq!(add.len(), N);
    debug_assert_eq!(sub0.len(), N);
    debug_assert_eq!(sub1.len(), N);
    debug_assert!(can_vectorize_i16_5(
        base.as_ptr(),
        add.as_ptr(),
        sub0.as_ptr(),
        sub1.as_ptr(),
        out.as_ptr(),
        N,
    ));
    #[cfg(target_arch = "x86_64")]
    unsafe {
        accum_add1_sub2_into_avx2(base, add, sub0, sub1, out);
    }
}

#[inline(always)]
pub(crate) fn accum_add1_sub2_into_both<const N: usize>(
    base: &mut [i16; N],
    add: &[i16],
    sub0: &[i16],
    sub1: &[i16],
    out: &mut [i16; N],
) {
    debug_assert_eq!(add.len(), N);
    debug_assert_eq!(sub0.len(), N);
    debug_assert_eq!(sub1.len(), N);
    debug_assert!(can_vectorize_i16_5(
        base.as_ptr(),
        add.as_ptr(),
        sub0.as_ptr(),
        sub1.as_ptr(),
        out.as_ptr(),
        N,
    ));
    #[cfg(target_arch = "x86_64")]
    unsafe {
        accum_add1_sub2_into_both_avx2(base, add, sub0, sub1, out);
    }
}

#[inline(always)]
pub(crate) fn accum_add2_sub1_into<const N: usize>(
    base: &[i16; N],
    add0: &[i16],
    add1: &[i16],
    sub: &[i16],
    out: &mut [i16; N],
) {
    debug_assert_eq!(add0.len(), N);
    debug_assert_eq!(add1.len(), N);
    debug_assert_eq!(sub.len(), N);
    debug_assert!(can_vectorize_i16_5(
        base.as_ptr(),
        add0.as_ptr(),
        add1.as_ptr(),
        sub.as_ptr(),
        out.as_ptr(),
        N,
    ));
    #[cfg(target_arch = "x86_64")]
    unsafe {
        accum_add2_sub1_into_avx2(base, add0, add1, sub, out);
    }
}

#[inline(always)]
pub(crate) fn accum_add2_sub1_into_both<const N: usize>(
    base: &mut [i16; N],
    add0: &[i16],
    add1: &[i16],
    sub: &[i16],
    out: &mut [i16; N],
) {
    debug_assert_eq!(add0.len(), N);
    debug_assert_eq!(add1.len(), N);
    debug_assert_eq!(sub.len(), N);
    debug_assert!(can_vectorize_i16_5(
        base.as_ptr(),
        add0.as_ptr(),
        add1.as_ptr(),
        sub.as_ptr(),
        out.as_ptr(),
        N,
    ));
    #[cfg(target_arch = "x86_64")]
    unsafe {
        accum_add2_sub1_into_both_avx2(base, add0, add1, sub, out);
    }
}

#[inline(always)]
pub(crate) fn accum_add2_sub2_into<const N: usize>(
    base: &[i16; N],
    add0: &[i16],
    add1: &[i16],
    sub0: &[i16],
    sub1: &[i16],
    out: &mut [i16; N],
) {
    debug_assert_eq!(add0.len(), N);
    debug_assert_eq!(add1.len(), N);
    debug_assert_eq!(sub0.len(), N);
    debug_assert_eq!(sub1.len(), N);
    debug_assert!(can_vectorize_i16_6(
        base.as_ptr(),
        add0.as_ptr(),
        add1.as_ptr(),
        sub0.as_ptr(),
        sub1.as_ptr(),
        out.as_ptr(),
        N,
    ));
    #[cfg(target_arch = "x86_64")]
    unsafe {
        accum_add2_sub2_into_avx2(base, add0, add1, sub0, sub1, out);
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn accum_add_avx2<const N: usize>(acc: &mut [i16; N], add: &[i16]) {
    for idx in (0..N).step_by(16) {
        unsafe {
            // SAFETY: callers validate length divisibility and 32-byte alignment for all slices.
            let sum = add_i16x16(
                load_i16x16(acc.as_ptr(), idx),
                load_i16x16(add.as_ptr(), idx),
            );
            store_i16x16(acc.as_mut_ptr(), idx, sum);
        }
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn accum_sub_avx2<const N: usize>(acc: &mut [i16; N], sub: &[i16]) {
    for idx in (0..N).step_by(16) {
        unsafe {
            // SAFETY: callers validate length divisibility and 32-byte alignment for all slices.
            let sum = sub_i16x16(
                load_i16x16(acc.as_ptr(), idx),
                load_i16x16(sub.as_ptr(), idx),
            );
            store_i16x16(acc.as_mut_ptr(), idx, sum);
        }
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn accum_add_sub_avx2<const N: usize>(acc: &mut [i16; N], add: &[i16], sub: &[i16]) {
    for idx in (0..N).step_by(16) {
        unsafe {
            // SAFETY: callers validate length divisibility and 32-byte alignment for all slices.
            let sum = add_i16x16(
                load_i16x16(acc.as_ptr(), idx),
                sub_i16x16(
                    load_i16x16(add.as_ptr(), idx),
                    load_i16x16(sub.as_ptr(), idx),
                ),
            );
            store_i16x16(acc.as_mut_ptr(), idx, sum);
        }
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn accum_add_into_both_avx2<const N: usize>(
    base: &mut [i16; N],
    add: &[i16],
    out: &mut [i16; N],
) {
    for idx in (0..N).step_by(16) {
        unsafe {
            // SAFETY: callers validate length divisibility and 32-byte alignment for all slices.
            let value = add_i16x16(
                load_i16x16(base.as_ptr(), idx),
                load_i16x16(add.as_ptr(), idx),
            );
            store_i16x16(base.as_mut_ptr(), idx, value);
            store_i16x16(out.as_mut_ptr(), idx, value);
        }
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn accum_sub_into_both_avx2<const N: usize>(
    base: &mut [i16; N],
    sub: &[i16],
    out: &mut [i16; N],
) {
    for idx in (0..N).step_by(16) {
        unsafe {
            // SAFETY: callers validate length divisibility and 32-byte alignment for all slices.
            let value = sub_i16x16(
                load_i16x16(base.as_ptr(), idx),
                load_i16x16(sub.as_ptr(), idx),
            );
            store_i16x16(base.as_mut_ptr(), idx, value);
            store_i16x16(out.as_mut_ptr(), idx, value);
        }
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn accum_add1_sub1_into_avx2<const N: usize>(
    base: &[i16; N],
    add: &[i16],
    sub: &[i16],
    out: &mut [i16; N],
) {
    for idx in (0..N).step_by(16) {
        unsafe {
            // SAFETY: callers validate length divisibility and 32-byte alignment for all slices.
            let value = add_i16x16(
                load_i16x16(base.as_ptr(), idx),
                sub_i16x16(
                    load_i16x16(add.as_ptr(), idx),
                    load_i16x16(sub.as_ptr(), idx),
                ),
            );
            store_i16x16(out.as_mut_ptr(), idx, value);
        }
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn accum_add1_sub1_into_both_avx2<const N: usize>(
    base: &mut [i16; N],
    add: &[i16],
    sub: &[i16],
    out: &mut [i16; N],
) {
    for idx in (0..N).step_by(16) {
        unsafe {
            // SAFETY: callers validate length divisibility and 32-byte alignment for all slices.
            let value = add_i16x16(
                load_i16x16(base.as_ptr(), idx),
                sub_i16x16(
                    load_i16x16(add.as_ptr(), idx),
                    load_i16x16(sub.as_ptr(), idx),
                ),
            );
            store_i16x16(base.as_mut_ptr(), idx, value);
            store_i16x16(out.as_mut_ptr(), idx, value);
        }
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn accum_add1_sub2_into_avx2<const N: usize>(
    base: &[i16; N],
    add: &[i16],
    sub0: &[i16],
    sub1: &[i16],
    out: &mut [i16; N],
) {
    for idx in (0..N).step_by(16) {
        unsafe {
            // SAFETY: callers validate length divisibility and 32-byte alignment for all slices.
            let value = sub_i16x16(
                add_i16x16(
                    load_i16x16(base.as_ptr(), idx),
                    load_i16x16(add.as_ptr(), idx),
                ),
                add_i16x16(
                    load_i16x16(sub0.as_ptr(), idx),
                    load_i16x16(sub1.as_ptr(), idx),
                ),
            );
            store_i16x16(out.as_mut_ptr(), idx, value);
        }
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn accum_add1_sub2_into_both_avx2<const N: usize>(
    base: &mut [i16; N],
    add: &[i16],
    sub0: &[i16],
    sub1: &[i16],
    out: &mut [i16; N],
) {
    for idx in (0..N).step_by(16) {
        unsafe {
            // SAFETY: callers validate length divisibility and 32-byte alignment for all slices.
            let value = sub_i16x16(
                add_i16x16(
                    load_i16x16(base.as_ptr(), idx),
                    load_i16x16(add.as_ptr(), idx),
                ),
                add_i16x16(
                    load_i16x16(sub0.as_ptr(), idx),
                    load_i16x16(sub1.as_ptr(), idx),
                ),
            );
            store_i16x16(base.as_mut_ptr(), idx, value);
            store_i16x16(out.as_mut_ptr(), idx, value);
        }
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn accum_add2_sub1_into_avx2<const N: usize>(
    base: &[i16; N],
    add0: &[i16],
    add1: &[i16],
    sub: &[i16],
    out: &mut [i16; N],
) {
    for idx in (0..N).step_by(16) {
        unsafe {
            // SAFETY: callers validate length divisibility and 32-byte alignment for all slices.
            let value = add_i16x16(
                add_i16x16(
                    load_i16x16(base.as_ptr(), idx),
                    load_i16x16(add0.as_ptr(), idx),
                ),
                sub_i16x16(
                    load_i16x16(add1.as_ptr(), idx),
                    load_i16x16(sub.as_ptr(), idx),
                ),
            );
            store_i16x16(out.as_mut_ptr(), idx, value);
        }
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn accum_add2_sub1_into_both_avx2<const N: usize>(
    base: &mut [i16; N],
    add0: &[i16],
    add1: &[i16],
    sub: &[i16],
    out: &mut [i16; N],
) {
    for idx in (0..N).step_by(16) {
        unsafe {
            // SAFETY: callers validate length divisibility and 32-byte alignment for all slices.
            let value = add_i16x16(
                add_i16x16(
                    load_i16x16(base.as_ptr(), idx),
                    load_i16x16(add0.as_ptr(), idx),
                ),
                sub_i16x16(
                    load_i16x16(add1.as_ptr(), idx),
                    load_i16x16(sub.as_ptr(), idx),
                ),
            );
            store_i16x16(base.as_mut_ptr(), idx, value);
            store_i16x16(out.as_mut_ptr(), idx, value);
        }
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn accum_add2_sub2_into_avx2<const N: usize>(
    base: &[i16; N],
    add0: &[i16],
    add1: &[i16],
    sub0: &[i16],
    sub1: &[i16],
    out: &mut [i16; N],
) {
    for idx in (0..N).step_by(16) {
        unsafe {
            // SAFETY: callers validate length divisibility and 32-byte alignment for all slices.
            let value = add_i16x16(
                load_i16x16(base.as_ptr(), idx),
                sub_i16x16(
                    add_i16x16(
                        load_i16x16(add0.as_ptr(), idx),
                        load_i16x16(add1.as_ptr(), idx),
                    ),
                    add_i16x16(
                        load_i16x16(sub0.as_ptr(), idx),
                        load_i16x16(sub1.as_ptr(), idx),
                    ),
                ),
            );
            store_i16x16(out.as_mut_ptr(), idx, value);
        }
    }
}
