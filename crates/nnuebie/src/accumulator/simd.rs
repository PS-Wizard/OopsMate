#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub(super) unsafe fn update_psqt_avx2(acc: &mut [i32; 8], psqt_slice: &[i32], add: bool) {
    let acc_ptr = acc.as_mut_ptr();
    let w_ptr = psqt_slice.as_ptr();
    let weights = _mm256_loadu_si256(w_ptr as *const __m256i);
    let current = _mm256_loadu_si256(acc_ptr as *const __m256i);
    let updated = if add {
        _mm256_add_epi32(current, weights)
    } else {
        _mm256_sub_epi32(current, weights)
    };
    _mm256_storeu_si256(acc_ptr as *mut __m256i, updated);
}

#[cfg(any(not(target_arch = "x86_64"), not(feature = "simd_avx2")))]
pub(super) fn update_psqt_scalar(acc: &mut [i32; 8], psqt_slice: &[i32], add: bool) {
    for (slot, &weight) in acc.iter_mut().zip(psqt_slice.iter()) {
        if add {
            *slot += weight;
        } else {
            *slot -= weight;
        }
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub(super) unsafe fn add_feature_avx2(acc: &mut [i16], weights: &[i16]) {
    let mut i = 0;
    let acc_ptr = acc.as_mut_ptr();
    let w_ptr = weights.as_ptr();
    let count = acc.len();

    while i + 64 <= count {
        let w0 = _mm256_load_si256(w_ptr.add(i) as *const _);
        let w1 = _mm256_load_si256(w_ptr.add(i + 16) as *const _);
        let w2 = _mm256_load_si256(w_ptr.add(i + 32) as *const _);
        let w3 = _mm256_load_si256(w_ptr.add(i + 48) as *const _);

        let a0 = _mm256_load_si256(acc_ptr.add(i) as *const _);
        let a1 = _mm256_load_si256(acc_ptr.add(i + 16) as *const _);
        let a2 = _mm256_load_si256(acc_ptr.add(i + 32) as *const _);
        let a3 = _mm256_load_si256(acc_ptr.add(i + 48) as *const _);

        _mm256_store_si256(acc_ptr.add(i) as *mut _, _mm256_add_epi16(a0, w0));
        _mm256_store_si256(acc_ptr.add(i + 16) as *mut _, _mm256_add_epi16(a1, w1));
        _mm256_store_si256(acc_ptr.add(i + 32) as *mut _, _mm256_add_epi16(a2, w2));
        _mm256_store_si256(acc_ptr.add(i + 48) as *mut _, _mm256_add_epi16(a3, w3));

        i += 64;
    }

    while i + 16 <= count {
        let w = _mm256_load_si256(w_ptr.add(i) as *const _);
        let a = _mm256_load_si256(acc_ptr.add(i) as *const _);
        _mm256_store_si256(acc_ptr.add(i) as *mut _, _mm256_add_epi16(a, w));
        i += 16;
    }

    for j in i..count {
        *acc_ptr.add(j) += *w_ptr.add(j);
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub(super) unsafe fn remove_feature_avx2(acc: &mut [i16], weights: &[i16]) {
    let mut i = 0;
    let acc_ptr = acc.as_mut_ptr();
    let w_ptr = weights.as_ptr();
    let count = acc.len();

    while i + 64 <= count {
        let w0 = _mm256_load_si256(w_ptr.add(i) as *const _);
        let w1 = _mm256_load_si256(w_ptr.add(i + 16) as *const _);
        let w2 = _mm256_load_si256(w_ptr.add(i + 32) as *const _);
        let w3 = _mm256_load_si256(w_ptr.add(i + 48) as *const _);

        let a0 = _mm256_load_si256(acc_ptr.add(i) as *const _);
        let a1 = _mm256_load_si256(acc_ptr.add(i + 16) as *const _);
        let a2 = _mm256_load_si256(acc_ptr.add(i + 32) as *const _);
        let a3 = _mm256_load_si256(acc_ptr.add(i + 48) as *const _);

        _mm256_store_si256(acc_ptr.add(i) as *mut _, _mm256_sub_epi16(a0, w0));
        _mm256_store_si256(acc_ptr.add(i + 16) as *mut _, _mm256_sub_epi16(a1, w1));
        _mm256_store_si256(acc_ptr.add(i + 32) as *mut _, _mm256_sub_epi16(a2, w2));
        _mm256_store_si256(acc_ptr.add(i + 48) as *mut _, _mm256_sub_epi16(a3, w3));

        i += 64;
    }

    while i + 16 <= count {
        let w = _mm256_load_si256(w_ptr.add(i) as *const _);
        let a = _mm256_load_si256(acc_ptr.add(i) as *const _);
        _mm256_store_si256(acc_ptr.add(i) as *mut _, _mm256_sub_epi16(a, w));
        i += 16;
    }

    for j in i..count {
        *acc_ptr.add(j) -= *w_ptr.add(j);
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub(super) unsafe fn update_accumulators_single_pass_avx2(
    prev_acc: &[i16],
    curr_acc: &mut [i16],
    added_ptrs: &[*const i16],
    removed_ptrs: &[*const i16],
) {
    debug_assert!(added_ptrs.len() <= 3);
    debug_assert!(removed_ptrs.len() <= 3);

    let added_count = added_ptrs.len();
    let removed_count = removed_ptrs.len();

    if added_count == 0 && removed_count == 0 {
        curr_acc.copy_from_slice(prev_acc);
        return;
    }

    let mut i = 0;
    let prev_ptr = prev_acc.as_ptr();
    let curr_ptr = curr_acc.as_mut_ptr();
    let count = prev_acc.len();

    if added_count == 1 && removed_count == 1 {
        let w_add = added_ptrs[0];
        let w_rem = removed_ptrs[0];

        while i + 64 <= count {
            let a0 = _mm256_load_si256(prev_ptr.add(i) as *const _);
            let a1 = _mm256_load_si256(prev_ptr.add(i + 16) as *const _);
            let a2 = _mm256_load_si256(prev_ptr.add(i + 32) as *const _);
            let a3 = _mm256_load_si256(prev_ptr.add(i + 48) as *const _);

            let ra0 = _mm256_load_si256(w_rem.add(i) as *const _);
            let ra1 = _mm256_load_si256(w_rem.add(i + 16) as *const _);
            let ra2 = _mm256_load_si256(w_rem.add(i + 32) as *const _);
            let ra3 = _mm256_load_si256(w_rem.add(i + 48) as *const _);

            let aa0 = _mm256_load_si256(w_add.add(i) as *const _);
            let aa1 = _mm256_load_si256(w_add.add(i + 16) as *const _);
            let aa2 = _mm256_load_si256(w_add.add(i + 32) as *const _);
            let aa3 = _mm256_load_si256(w_add.add(i + 48) as *const _);

            _mm256_store_si256(
                curr_ptr.add(i) as *mut _,
                _mm256_add_epi16(_mm256_sub_epi16(a0, ra0), aa0),
            );
            _mm256_store_si256(
                curr_ptr.add(i + 16) as *mut _,
                _mm256_add_epi16(_mm256_sub_epi16(a1, ra1), aa1),
            );
            _mm256_store_si256(
                curr_ptr.add(i + 32) as *mut _,
                _mm256_add_epi16(_mm256_sub_epi16(a2, ra2), aa2),
            );
            _mm256_store_si256(
                curr_ptr.add(i + 48) as *mut _,
                _mm256_add_epi16(_mm256_sub_epi16(a3, ra3), aa3),
            );

            i += 64;
        }
    } else if added_count == 1 && removed_count == 0 {
        let w_add = added_ptrs[0];

        while i + 64 <= count {
            let a0 = _mm256_load_si256(prev_ptr.add(i) as *const _);
            let a1 = _mm256_load_si256(prev_ptr.add(i + 16) as *const _);
            let a2 = _mm256_load_si256(prev_ptr.add(i + 32) as *const _);
            let a3 = _mm256_load_si256(prev_ptr.add(i + 48) as *const _);

            let aa0 = _mm256_load_si256(w_add.add(i) as *const _);
            let aa1 = _mm256_load_si256(w_add.add(i + 16) as *const _);
            let aa2 = _mm256_load_si256(w_add.add(i + 32) as *const _);
            let aa3 = _mm256_load_si256(w_add.add(i + 48) as *const _);

            _mm256_store_si256(curr_ptr.add(i) as *mut _, _mm256_add_epi16(a0, aa0));
            _mm256_store_si256(curr_ptr.add(i + 16) as *mut _, _mm256_add_epi16(a1, aa1));
            _mm256_store_si256(curr_ptr.add(i + 32) as *mut _, _mm256_add_epi16(a2, aa2));
            _mm256_store_si256(curr_ptr.add(i + 48) as *mut _, _mm256_add_epi16(a3, aa3));

            i += 64;
        }
    } else if added_count == 0 && removed_count == 1 {
        let w_rem = removed_ptrs[0];

        while i + 64 <= count {
            let a0 = _mm256_load_si256(prev_ptr.add(i) as *const _);
            let a1 = _mm256_load_si256(prev_ptr.add(i + 16) as *const _);
            let a2 = _mm256_load_si256(prev_ptr.add(i + 32) as *const _);
            let a3 = _mm256_load_si256(prev_ptr.add(i + 48) as *const _);

            let ra0 = _mm256_load_si256(w_rem.add(i) as *const _);
            let ra1 = _mm256_load_si256(w_rem.add(i + 16) as *const _);
            let ra2 = _mm256_load_si256(w_rem.add(i + 32) as *const _);
            let ra3 = _mm256_load_si256(w_rem.add(i + 48) as *const _);

            _mm256_store_si256(curr_ptr.add(i) as *mut _, _mm256_sub_epi16(a0, ra0));
            _mm256_store_si256(curr_ptr.add(i + 16) as *mut _, _mm256_sub_epi16(a1, ra1));
            _mm256_store_si256(curr_ptr.add(i + 32) as *mut _, _mm256_sub_epi16(a2, ra2));
            _mm256_store_si256(curr_ptr.add(i + 48) as *mut _, _mm256_sub_epi16(a3, ra3));

            i += 64;
        }
    } else if added_count == 1 && removed_count == 2 {
        let w_add = added_ptrs[0];
        let w_rem0 = removed_ptrs[0];
        let w_rem1 = removed_ptrs[1];

        while i + 64 <= count {
            let a0 = _mm256_load_si256(prev_ptr.add(i) as *const _);
            let a1 = _mm256_load_si256(prev_ptr.add(i + 16) as *const _);
            let a2 = _mm256_load_si256(prev_ptr.add(i + 32) as *const _);
            let a3 = _mm256_load_si256(prev_ptr.add(i + 48) as *const _);

            let rr00 = _mm256_load_si256(w_rem0.add(i) as *const _);
            let rr01 = _mm256_load_si256(w_rem0.add(i + 16) as *const _);
            let rr02 = _mm256_load_si256(w_rem0.add(i + 32) as *const _);
            let rr03 = _mm256_load_si256(w_rem0.add(i + 48) as *const _);

            let rr10 = _mm256_load_si256(w_rem1.add(i) as *const _);
            let rr11 = _mm256_load_si256(w_rem1.add(i + 16) as *const _);
            let rr12 = _mm256_load_si256(w_rem1.add(i + 32) as *const _);
            let rr13 = _mm256_load_si256(w_rem1.add(i + 48) as *const _);

            let aa0 = _mm256_load_si256(w_add.add(i) as *const _);
            let aa1 = _mm256_load_si256(w_add.add(i + 16) as *const _);
            let aa2 = _mm256_load_si256(w_add.add(i + 32) as *const _);
            let aa3 = _mm256_load_si256(w_add.add(i + 48) as *const _);

            _mm256_store_si256(
                curr_ptr.add(i) as *mut _,
                _mm256_add_epi16(_mm256_sub_epi16(_mm256_sub_epi16(a0, rr00), rr10), aa0),
            );
            _mm256_store_si256(
                curr_ptr.add(i + 16) as *mut _,
                _mm256_add_epi16(_mm256_sub_epi16(_mm256_sub_epi16(a1, rr01), rr11), aa1),
            );
            _mm256_store_si256(
                curr_ptr.add(i + 32) as *mut _,
                _mm256_add_epi16(_mm256_sub_epi16(_mm256_sub_epi16(a2, rr02), rr12), aa2),
            );
            _mm256_store_si256(
                curr_ptr.add(i + 48) as *mut _,
                _mm256_add_epi16(_mm256_sub_epi16(_mm256_sub_epi16(a3, rr03), rr13), aa3),
            );

            i += 64;
        }
    } else if added_count == 2 && removed_count == 1 {
        let w_add0 = added_ptrs[0];
        let w_add1 = added_ptrs[1];
        let w_rem = removed_ptrs[0];

        while i + 64 <= count {
            let a0 = _mm256_load_si256(prev_ptr.add(i) as *const _);
            let a1 = _mm256_load_si256(prev_ptr.add(i + 16) as *const _);
            let a2 = _mm256_load_si256(prev_ptr.add(i + 32) as *const _);
            let a3 = _mm256_load_si256(prev_ptr.add(i + 48) as *const _);

            let rr0 = _mm256_load_si256(w_rem.add(i) as *const _);
            let rr1 = _mm256_load_si256(w_rem.add(i + 16) as *const _);
            let rr2 = _mm256_load_si256(w_rem.add(i + 32) as *const _);
            let rr3 = _mm256_load_si256(w_rem.add(i + 48) as *const _);

            let aa00 = _mm256_load_si256(w_add0.add(i) as *const _);
            let aa01 = _mm256_load_si256(w_add0.add(i + 16) as *const _);
            let aa02 = _mm256_load_si256(w_add0.add(i + 32) as *const _);
            let aa03 = _mm256_load_si256(w_add0.add(i + 48) as *const _);

            let aa10 = _mm256_load_si256(w_add1.add(i) as *const _);
            let aa11 = _mm256_load_si256(w_add1.add(i + 16) as *const _);
            let aa12 = _mm256_load_si256(w_add1.add(i + 32) as *const _);
            let aa13 = _mm256_load_si256(w_add1.add(i + 48) as *const _);

            _mm256_store_si256(
                curr_ptr.add(i) as *mut _,
                _mm256_add_epi16(_mm256_add_epi16(_mm256_sub_epi16(a0, rr0), aa00), aa10),
            );
            _mm256_store_si256(
                curr_ptr.add(i + 16) as *mut _,
                _mm256_add_epi16(_mm256_add_epi16(_mm256_sub_epi16(a1, rr1), aa01), aa11),
            );
            _mm256_store_si256(
                curr_ptr.add(i + 32) as *mut _,
                _mm256_add_epi16(_mm256_add_epi16(_mm256_sub_epi16(a2, rr2), aa02), aa12),
            );
            _mm256_store_si256(
                curr_ptr.add(i + 48) as *mut _,
                _mm256_add_epi16(_mm256_add_epi16(_mm256_sub_epi16(a3, rr3), aa03), aa13),
            );

            i += 64;
        }
    } else {
        while i + 64 <= count {
            let mut a0 = _mm256_load_si256(prev_ptr.add(i) as *const _);
            let mut a1 = _mm256_load_si256(prev_ptr.add(i + 16) as *const _);
            let mut a2 = _mm256_load_si256(prev_ptr.add(i + 32) as *const _);
            let mut a3 = _mm256_load_si256(prev_ptr.add(i + 48) as *const _);

            for &ptr in removed_ptrs.iter().take(removed_count) {
                let w0 = _mm256_load_si256(ptr.add(i) as *const _);
                let w1 = _mm256_load_si256(ptr.add(i + 16) as *const _);
                let w2 = _mm256_load_si256(ptr.add(i + 32) as *const _);
                let w3 = _mm256_load_si256(ptr.add(i + 48) as *const _);
                a0 = _mm256_sub_epi16(a0, w0);
                a1 = _mm256_sub_epi16(a1, w1);
                a2 = _mm256_sub_epi16(a2, w2);
                a3 = _mm256_sub_epi16(a3, w3);
            }

            for &ptr in added_ptrs.iter().take(added_count) {
                let w0 = _mm256_load_si256(ptr.add(i) as *const _);
                let w1 = _mm256_load_si256(ptr.add(i + 16) as *const _);
                let w2 = _mm256_load_si256(ptr.add(i + 32) as *const _);
                let w3 = _mm256_load_si256(ptr.add(i + 48) as *const _);
                a0 = _mm256_add_epi16(a0, w0);
                a1 = _mm256_add_epi16(a1, w1);
                a2 = _mm256_add_epi16(a2, w2);
                a3 = _mm256_add_epi16(a3, w3);
            }

            _mm256_store_si256(curr_ptr.add(i) as *mut _, a0);
            _mm256_store_si256(curr_ptr.add(i + 16) as *mut _, a1);
            _mm256_store_si256(curr_ptr.add(i + 32) as *mut _, a2);
            _mm256_store_si256(curr_ptr.add(i + 48) as *mut _, a3);

            i += 64;
        }
    }

    while i + 16 <= count {
        let mut acc = _mm256_load_si256(prev_ptr.add(i) as *const _);

        for &ptr in removed_ptrs.iter().take(removed_count) {
            acc = _mm256_sub_epi16(acc, _mm256_load_si256(ptr.add(i) as *const _));
        }
        for &ptr in added_ptrs.iter().take(added_count) {
            acc = _mm256_add_epi16(acc, _mm256_load_si256(ptr.add(i) as *const _));
        }

        _mm256_store_si256(curr_ptr.add(i) as *mut _, acc);
        i += 16;
    }

    for j in i..count {
        let mut value = *prev_ptr.add(j);
        for &ptr in removed_ptrs.iter().take(removed_count) {
            value = value.wrapping_sub(*ptr.add(j));
        }
        for &ptr in added_ptrs.iter().take(added_count) {
            value = value.wrapping_add(*ptr.add(j));
        }
        *curr_ptr.add(j) = value;
    }
}

#[cfg(any(not(target_arch = "x86_64"), not(feature = "simd_avx2")))]
pub(super) unsafe fn add_feature_scalar(acc: &mut [i16], weights: &[i16]) {
    for (slot, weight) in acc.iter_mut().zip(weights.iter()) {
        *slot += *weight;
    }
}

#[cfg(any(not(target_arch = "x86_64"), not(feature = "simd_avx2")))]
pub(super) unsafe fn remove_feature_scalar(acc: &mut [i16], weights: &[i16]) {
    for (slot, weight) in acc.iter_mut().zip(weights.iter()) {
        *slot -= *weight;
    }
}

#[cfg(any(not(target_arch = "x86_64"), not(feature = "simd_avx2")))]
pub(super) unsafe fn update_accumulators_single_pass_scalar(
    prev_acc: &[i16],
    curr_acc: &mut [i16],
    added_ptrs: &[*const i16],
    removed_ptrs: &[*const i16],
) {
    for (index, value) in prev_acc.iter().copied().enumerate() {
        let mut value = value;
        for &ptr in removed_ptrs {
            value = value.wrapping_sub(*ptr.add(index));
        }
        for &ptr in added_ptrs {
            value = value.wrapping_add(*ptr.add(index));
        }
        curr_acc[index] = value;
    }
}
