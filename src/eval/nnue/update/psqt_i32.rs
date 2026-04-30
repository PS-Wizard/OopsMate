use super::simd::{
    can_vectorize_i32, can_vectorize_i32_3, can_vectorize_i32_4, can_vectorize_i32_5,
    can_vectorize_i32_6,
};
use crate::eval::nnue::constants::PSQT_BUCKETS;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{
    __m256i, _mm256_add_epi32, _mm256_load_si256, _mm256_store_si256, _mm256_sub_epi32,
};

#[inline(always)]
pub(crate) fn psqt_add(psqt: &mut [i32; PSQT_BUCKETS], add: &[i32]) {
    debug_assert_eq!(add.len(), PSQT_BUCKETS);
    debug_assert!(can_vectorize_i32(psqt.as_ptr(), add.as_ptr()));
    #[cfg(target_arch = "x86_64")]
    unsafe {
        psqt_add_avx2(psqt, add);
    }
}

#[inline(always)]
pub(crate) fn psqt_sub(psqt: &mut [i32; PSQT_BUCKETS], sub: &[i32]) {
    debug_assert_eq!(sub.len(), PSQT_BUCKETS);
    debug_assert!(can_vectorize_i32(psqt.as_ptr(), sub.as_ptr()));
    #[cfg(target_arch = "x86_64")]
    unsafe {
        psqt_sub_avx2(psqt, sub);
    }
}

#[inline(always)]
pub(crate) fn psqt_add_sub(psqt: &mut [i32; PSQT_BUCKETS], add: &[i32], sub: &[i32]) {
    debug_assert_eq!(add.len(), PSQT_BUCKETS);
    debug_assert_eq!(sub.len(), PSQT_BUCKETS);
    debug_assert!(can_vectorize_i32_3(psqt.as_ptr(), add.as_ptr(), sub.as_ptr()));
    #[cfg(target_arch = "x86_64")]
    unsafe {
        psqt_add_sub_avx2(psqt, add, sub);
    }
}

#[inline(always)]
pub(crate) fn psqt_add_into_both(
    base: &mut [i32; PSQT_BUCKETS],
    add: &[i32],
    out: &mut [i32; PSQT_BUCKETS],
) {
    debug_assert_eq!(add.len(), PSQT_BUCKETS);
    debug_assert!(can_vectorize_i32_3(base.as_ptr(), add.as_ptr(), out.as_ptr()));
    #[cfg(target_arch = "x86_64")]
    unsafe {
        psqt_add_into_both_avx2(base, add, out);
    }
}

#[inline(always)]
pub(crate) fn psqt_sub_into_both(
    base: &mut [i32; PSQT_BUCKETS],
    sub: &[i32],
    out: &mut [i32; PSQT_BUCKETS],
) {
    debug_assert_eq!(sub.len(), PSQT_BUCKETS);
    debug_assert!(can_vectorize_i32_3(base.as_ptr(), sub.as_ptr(), out.as_ptr()));
    #[cfg(target_arch = "x86_64")]
    unsafe {
        psqt_sub_into_both_avx2(base, sub, out);
    }
}

#[inline(always)]
pub(crate) fn psqt_add1_sub1_into(
    base: &[i32; PSQT_BUCKETS],
    add: &[i32],
    sub: &[i32],
    out: &mut [i32; PSQT_BUCKETS],
) {
    debug_assert_eq!(add.len(), PSQT_BUCKETS);
    debug_assert_eq!(sub.len(), PSQT_BUCKETS);
    debug_assert!(can_vectorize_i32_4(base.as_ptr(), add.as_ptr(), sub.as_ptr(), out.as_ptr()));
    #[cfg(target_arch = "x86_64")]
    unsafe {
        psqt_add1_sub1_into_avx2(base, add, sub, out);
    }
}

#[inline(always)]
pub(crate) fn psqt_add1_sub1_into_both(
    base: &mut [i32; PSQT_BUCKETS],
    add: &[i32],
    sub: &[i32],
    out: &mut [i32; PSQT_BUCKETS],
) {
    debug_assert_eq!(add.len(), PSQT_BUCKETS);
    debug_assert_eq!(sub.len(), PSQT_BUCKETS);
    debug_assert!(can_vectorize_i32_4(base.as_ptr(), add.as_ptr(), sub.as_ptr(), out.as_ptr()));
    #[cfg(target_arch = "x86_64")]
    unsafe {
        psqt_add1_sub1_into_both_avx2(base, add, sub, out);
    }
}

#[inline(always)]
pub(crate) fn psqt_add1_sub2_into(
    base: &[i32; PSQT_BUCKETS],
    add: &[i32],
    sub0: &[i32],
    sub1: &[i32],
    out: &mut [i32; PSQT_BUCKETS],
) {
    debug_assert_eq!(add.len(), PSQT_BUCKETS);
    debug_assert_eq!(sub0.len(), PSQT_BUCKETS);
    debug_assert_eq!(sub1.len(), PSQT_BUCKETS);
    debug_assert!(can_vectorize_i32_5(
        base.as_ptr(),
        add.as_ptr(),
        sub0.as_ptr(),
        sub1.as_ptr(),
        out.as_ptr(),
    ));
    #[cfg(target_arch = "x86_64")]
    unsafe {
        psqt_add1_sub2_into_avx2(base, add, sub0, sub1, out);
    }
}

#[inline(always)]
pub(crate) fn psqt_add1_sub2_into_both(
    base: &mut [i32; PSQT_BUCKETS],
    add: &[i32],
    sub0: &[i32],
    sub1: &[i32],
    out: &mut [i32; PSQT_BUCKETS],
) {
    debug_assert_eq!(add.len(), PSQT_BUCKETS);
    debug_assert_eq!(sub0.len(), PSQT_BUCKETS);
    debug_assert_eq!(sub1.len(), PSQT_BUCKETS);
    debug_assert!(can_vectorize_i32_5(
        base.as_ptr(),
        add.as_ptr(),
        sub0.as_ptr(),
        sub1.as_ptr(),
        out.as_ptr(),
    ));
    #[cfg(target_arch = "x86_64")]
    unsafe {
        psqt_add1_sub2_into_both_avx2(base, add, sub0, sub1, out);
    }
}

#[inline(always)]
pub(crate) fn psqt_add2_sub1_into(
    base: &[i32; PSQT_BUCKETS],
    add0: &[i32],
    add1: &[i32],
    sub: &[i32],
    out: &mut [i32; PSQT_BUCKETS],
) {
    debug_assert_eq!(add0.len(), PSQT_BUCKETS);
    debug_assert_eq!(add1.len(), PSQT_BUCKETS);
    debug_assert_eq!(sub.len(), PSQT_BUCKETS);
    debug_assert!(can_vectorize_i32_5(
        base.as_ptr(),
        add0.as_ptr(),
        add1.as_ptr(),
        sub.as_ptr(),
        out.as_ptr(),
    ));
    #[cfg(target_arch = "x86_64")]
    unsafe {
        psqt_add2_sub1_into_avx2(base, add0, add1, sub, out);
    }
}

#[inline(always)]
pub(crate) fn psqt_add2_sub1_into_both(
    base: &mut [i32; PSQT_BUCKETS],
    add0: &[i32],
    add1: &[i32],
    sub: &[i32],
    out: &mut [i32; PSQT_BUCKETS],
) {
    debug_assert_eq!(add0.len(), PSQT_BUCKETS);
    debug_assert_eq!(add1.len(), PSQT_BUCKETS);
    debug_assert_eq!(sub.len(), PSQT_BUCKETS);
    debug_assert!(can_vectorize_i32_5(
        base.as_ptr(),
        add0.as_ptr(),
        add1.as_ptr(),
        sub.as_ptr(),
        out.as_ptr(),
    ));
    #[cfg(target_arch = "x86_64")]
    unsafe {
        psqt_add2_sub1_into_both_avx2(base, add0, add1, sub, out);
    }
}

#[inline(always)]
pub(crate) fn psqt_add2_sub2_into(
    base: &[i32; PSQT_BUCKETS],
    add0: &[i32],
    add1: &[i32],
    sub0: &[i32],
    sub1: &[i32],
    out: &mut [i32; PSQT_BUCKETS],
) {
    debug_assert_eq!(add0.len(), PSQT_BUCKETS);
    debug_assert_eq!(add1.len(), PSQT_BUCKETS);
    debug_assert_eq!(sub0.len(), PSQT_BUCKETS);
    debug_assert_eq!(sub1.len(), PSQT_BUCKETS);
    debug_assert!(can_vectorize_i32_6(
        base.as_ptr(),
        add0.as_ptr(),
        add1.as_ptr(),
        sub0.as_ptr(),
        sub1.as_ptr(),
        out.as_ptr(),
    ));
    #[cfg(target_arch = "x86_64")]
    unsafe {
        psqt_add2_sub2_into_avx2(base, add0, add1, sub0, sub1, out);
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn psqt_add_avx2(psqt: &mut [i32; PSQT_BUCKETS], add: &[i32]) {
    unsafe {
        let value = _mm256_add_epi32(
            _mm256_load_si256(psqt.as_ptr().cast::<__m256i>()),
            _mm256_load_si256(add.as_ptr().cast::<__m256i>()),
        );
        _mm256_store_si256(psqt.as_mut_ptr().cast::<__m256i>(), value);
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn psqt_sub_avx2(psqt: &mut [i32; PSQT_BUCKETS], sub: &[i32]) {
    unsafe {
        let value = _mm256_sub_epi32(
            _mm256_load_si256(psqt.as_ptr().cast::<__m256i>()),
            _mm256_load_si256(sub.as_ptr().cast::<__m256i>()),
        );
        _mm256_store_si256(psqt.as_mut_ptr().cast::<__m256i>(), value);
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn psqt_add_sub_avx2(psqt: &mut [i32; PSQT_BUCKETS], add: &[i32], sub: &[i32]) {
    unsafe {
        let value = _mm256_add_epi32(
            _mm256_load_si256(psqt.as_ptr().cast::<__m256i>()),
            _mm256_sub_epi32(
                _mm256_load_si256(add.as_ptr().cast::<__m256i>()),
                _mm256_load_si256(sub.as_ptr().cast::<__m256i>()),
            ),
        );
        _mm256_store_si256(psqt.as_mut_ptr().cast::<__m256i>(), value);
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn psqt_add_into_both_avx2(
    base: &mut [i32; PSQT_BUCKETS],
    add: &[i32],
    out: &mut [i32; PSQT_BUCKETS],
) {
    unsafe {
        let value = _mm256_add_epi32(
            _mm256_load_si256(base.as_ptr().cast::<__m256i>()),
            _mm256_load_si256(add.as_ptr().cast::<__m256i>()),
        );
        _mm256_store_si256(base.as_mut_ptr().cast::<__m256i>(), value);
        _mm256_store_si256(out.as_mut_ptr().cast::<__m256i>(), value);
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn psqt_sub_into_both_avx2(
    base: &mut [i32; PSQT_BUCKETS],
    sub: &[i32],
    out: &mut [i32; PSQT_BUCKETS],
) {
    unsafe {
        let value = _mm256_sub_epi32(
            _mm256_load_si256(base.as_ptr().cast::<__m256i>()),
            _mm256_load_si256(sub.as_ptr().cast::<__m256i>()),
        );
        _mm256_store_si256(base.as_mut_ptr().cast::<__m256i>(), value);
        _mm256_store_si256(out.as_mut_ptr().cast::<__m256i>(), value);
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn psqt_add1_sub1_into_avx2(
    base: &[i32; PSQT_BUCKETS],
    add: &[i32],
    sub: &[i32],
    out: &mut [i32; PSQT_BUCKETS],
) {
    unsafe {
        let value = _mm256_add_epi32(
            _mm256_load_si256(base.as_ptr().cast::<__m256i>()),
            _mm256_sub_epi32(
                _mm256_load_si256(add.as_ptr().cast::<__m256i>()),
                _mm256_load_si256(sub.as_ptr().cast::<__m256i>()),
            ),
        );
        _mm256_store_si256(out.as_mut_ptr().cast::<__m256i>(), value);
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn psqt_add1_sub1_into_both_avx2(
    base: &mut [i32; PSQT_BUCKETS],
    add: &[i32],
    sub: &[i32],
    out: &mut [i32; PSQT_BUCKETS],
) {
    unsafe {
        let value = _mm256_add_epi32(
            _mm256_load_si256(base.as_ptr().cast::<__m256i>()),
            _mm256_sub_epi32(
                _mm256_load_si256(add.as_ptr().cast::<__m256i>()),
                _mm256_load_si256(sub.as_ptr().cast::<__m256i>()),
            ),
        );
        _mm256_store_si256(base.as_mut_ptr().cast::<__m256i>(), value);
        _mm256_store_si256(out.as_mut_ptr().cast::<__m256i>(), value);
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn psqt_add1_sub2_into_avx2(
    base: &[i32; PSQT_BUCKETS],
    add: &[i32],
    sub0: &[i32],
    sub1: &[i32],
    out: &mut [i32; PSQT_BUCKETS],
) {
    unsafe {
        let value = _mm256_sub_epi32(
            _mm256_add_epi32(
                _mm256_load_si256(base.as_ptr().cast::<__m256i>()),
                _mm256_load_si256(add.as_ptr().cast::<__m256i>()),
            ),
            _mm256_add_epi32(
                _mm256_load_si256(sub0.as_ptr().cast::<__m256i>()),
                _mm256_load_si256(sub1.as_ptr().cast::<__m256i>()),
            ),
        );
        _mm256_store_si256(out.as_mut_ptr().cast::<__m256i>(), value);
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn psqt_add1_sub2_into_both_avx2(
    base: &mut [i32; PSQT_BUCKETS],
    add: &[i32],
    sub0: &[i32],
    sub1: &[i32],
    out: &mut [i32; PSQT_BUCKETS],
) {
    unsafe {
        let value = _mm256_sub_epi32(
            _mm256_add_epi32(
                _mm256_load_si256(base.as_ptr().cast::<__m256i>()),
                _mm256_load_si256(add.as_ptr().cast::<__m256i>()),
            ),
            _mm256_add_epi32(
                _mm256_load_si256(sub0.as_ptr().cast::<__m256i>()),
                _mm256_load_si256(sub1.as_ptr().cast::<__m256i>()),
            ),
        );
        _mm256_store_si256(base.as_mut_ptr().cast::<__m256i>(), value);
        _mm256_store_si256(out.as_mut_ptr().cast::<__m256i>(), value);
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn psqt_add2_sub1_into_avx2(
    base: &[i32; PSQT_BUCKETS],
    add0: &[i32],
    add1: &[i32],
    sub: &[i32],
    out: &mut [i32; PSQT_BUCKETS],
) {
    unsafe {
        let value = _mm256_add_epi32(
            _mm256_add_epi32(
                _mm256_load_si256(base.as_ptr().cast::<__m256i>()),
                _mm256_load_si256(add0.as_ptr().cast::<__m256i>()),
            ),
            _mm256_sub_epi32(
                _mm256_load_si256(add1.as_ptr().cast::<__m256i>()),
                _mm256_load_si256(sub.as_ptr().cast::<__m256i>()),
            ),
        );
        _mm256_store_si256(out.as_mut_ptr().cast::<__m256i>(), value);
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn psqt_add2_sub1_into_both_avx2(
    base: &mut [i32; PSQT_BUCKETS],
    add0: &[i32],
    add1: &[i32],
    sub: &[i32],
    out: &mut [i32; PSQT_BUCKETS],
) {
    unsafe {
        let value = _mm256_add_epi32(
            _mm256_add_epi32(
                _mm256_load_si256(base.as_ptr().cast::<__m256i>()),
                _mm256_load_si256(add0.as_ptr().cast::<__m256i>()),
            ),
            _mm256_sub_epi32(
                _mm256_load_si256(add1.as_ptr().cast::<__m256i>()),
                _mm256_load_si256(sub.as_ptr().cast::<__m256i>()),
            ),
        );
        _mm256_store_si256(base.as_mut_ptr().cast::<__m256i>(), value);
        _mm256_store_si256(out.as_mut_ptr().cast::<__m256i>(), value);
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn psqt_add2_sub2_into_avx2(
    base: &[i32; PSQT_BUCKETS],
    add0: &[i32],
    add1: &[i32],
    sub0: &[i32],
    sub1: &[i32],
    out: &mut [i32; PSQT_BUCKETS],
) {
    unsafe {
        let value = _mm256_add_epi32(
            _mm256_load_si256(base.as_ptr().cast::<__m256i>()),
            _mm256_sub_epi32(
                _mm256_add_epi32(
                    _mm256_load_si256(add0.as_ptr().cast::<__m256i>()),
                    _mm256_load_si256(add1.as_ptr().cast::<__m256i>()),
                ),
                _mm256_add_epi32(
                    _mm256_load_si256(sub0.as_ptr().cast::<__m256i>()),
                    _mm256_load_si256(sub1.as_ptr().cast::<__m256i>()),
                ),
            ),
        );
        _mm256_store_si256(out.as_mut_ptr().cast::<__m256i>(), value);
    }
}
