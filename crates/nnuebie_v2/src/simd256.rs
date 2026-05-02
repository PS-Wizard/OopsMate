use std::arch::x86_64::{
    __m256i, _mm256_add_epi16, _mm256_add_epi32, _mm256_castsi256_ps, _mm256_castsi256_si128,
    _mm256_cmpeq_epi32, _mm256_dpbusd_epi32, _mm256_extracti128_si256, _mm256_load_si256,
    _mm256_loadu_si256, _mm256_max_epi16, _mm256_min_epi16, _mm256_movemask_ps, _mm256_mulhi_epi16,
    _mm256_packus_epi16, _mm256_set1_epi16, _mm256_set1_epi32, _mm256_setzero_si256,
    _mm256_slli_epi16, _mm256_store_si256, _mm256_sub_epi16, _mm256_sub_epi32, _mm_add_epi32,
    _mm_cvtsi128_si32, _mm_shuffle_epi32,
};

#[repr(transparent)]
#[derive(Clone, Copy)]
pub(crate) struct I16x16(__m256i);

#[repr(transparent)]
#[derive(Clone, Copy)]
pub(crate) struct I32x8(__m256i);

#[repr(transparent)]
#[derive(Clone, Copy)]
pub(crate) struct Bytes32(__m256i);

#[inline(always)]
pub(crate) fn is_32_byte_aligned<T>(ptr: *const T) -> bool {
    (ptr as usize & 31) == 0
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn load_i16x16(ptr: *const i16, offset: usize) -> I16x16 {
    unsafe {
        // SAFETY: caller guarantees `ptr.add(offset)` is 32-byte aligned and valid for 16 i16s.
        I16x16(_mm256_load_si256(ptr.add(offset).cast::<__m256i>()))
    }
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn store_i16x16(ptr: *mut i16, offset: usize, value: I16x16) {
    unsafe {
        // SAFETY: caller guarantees `ptr.add(offset)` is 32-byte aligned and valid for 16 i16s.
        _mm256_store_si256(ptr.add(offset).cast::<__m256i>(), value.0);
    }
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn zero_i16x16() -> I16x16 {
    I16x16(_mm256_setzero_si256())
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn splat_i16x16(value: i16) -> I16x16 {
    I16x16(_mm256_set1_epi16(value))
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn add_i16x16(lhs: I16x16, rhs: I16x16) -> I16x16 {
    I16x16(_mm256_add_epi16(lhs.0, rhs.0))
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn sub_i16x16(lhs: I16x16, rhs: I16x16) -> I16x16 {
    I16x16(_mm256_sub_epi16(lhs.0, rhs.0))
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn min_i16x16(lhs: I16x16, rhs: I16x16) -> I16x16 {
    I16x16(_mm256_min_epi16(lhs.0, rhs.0))
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn max_i16x16(lhs: I16x16, rhs: I16x16) -> I16x16 {
    I16x16(_mm256_max_epi16(lhs.0, rhs.0))
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn shl_i16x16<const SHIFT: i32>(value: I16x16) -> I16x16 {
    I16x16(_mm256_slli_epi16::<SHIFT>(value.0))
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn mulhi_i16x16(lhs: I16x16, rhs: I16x16) -> I16x16 {
    I16x16(_mm256_mulhi_epi16(lhs.0, rhs.0))
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn packus_i16x16(lhs: I16x16, rhs: I16x16) -> Bytes32 {
    Bytes32(_mm256_packus_epi16(lhs.0, rhs.0))
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn load_i32x8(ptr: *const i32, offset: usize) -> I32x8 {
    unsafe {
        // SAFETY: caller guarantees `ptr.add(offset)` is 32-byte aligned and valid for 8 i32s.
        I32x8(_mm256_load_si256(ptr.add(offset).cast::<__m256i>()))
    }
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn store_i32x8(ptr: *mut i32, offset: usize, value: I32x8) {
    unsafe {
        // SAFETY: caller guarantees `ptr.add(offset)` is 32-byte aligned and valid for 8 i32s.
        _mm256_store_si256(ptr.add(offset).cast::<__m256i>(), value.0);
    }
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn zero_i32x8() -> I32x8 {
    I32x8(_mm256_setzero_si256())
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn add_i32x8(lhs: I32x8, rhs: I32x8) -> I32x8 {
    I32x8(_mm256_add_epi32(lhs.0, rhs.0))
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn sub_i32x8(lhs: I32x8, rhs: I32x8) -> I32x8 {
    I32x8(_mm256_sub_epi32(lhs.0, rhs.0))
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn horizontal_sum_i32x8(value: I32x8) -> i32 {
    let mut sum128 = _mm_add_epi32(
        _mm256_castsi256_si128(value.0),
        _mm256_extracti128_si256(value.0, 1),
    );
    sum128 = _mm_add_epi32(sum128, _mm_shuffle_epi32(sum128, 0b10_11_00_01));
    sum128 = _mm_add_epi32(sum128, _mm_shuffle_epi32(sum128, 0b01_00_11_10));
    _mm_cvtsi128_si32(sum128)
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn load_bytes32_i8(ptr: *const i8, offset: usize) -> Bytes32 {
    unsafe {
        // SAFETY: caller guarantees `ptr.add(offset)` is 32-byte aligned and valid for 32 bytes.
        Bytes32(_mm256_load_si256(ptr.add(offset).cast::<__m256i>()))
    }
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn load_bytes32_u8_aligned(ptr: *const u8, offset: usize) -> Bytes32 {
    unsafe {
        // SAFETY: caller guarantees `ptr.add(offset)` is 32-byte aligned and valid for 32 bytes.
        Bytes32(_mm256_load_si256(ptr.add(offset).cast::<__m256i>()))
    }
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn load_bytes32_u8_unaligned(ptr: *const u8, offset: usize) -> Bytes32 {
    unsafe {
        // SAFETY: caller guarantees `ptr.add(offset)` is valid for 32 bytes.
        Bytes32(_mm256_loadu_si256(ptr.add(offset).cast::<__m256i>()))
    }
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn store_bytes32_u8(ptr: *mut u8, offset: usize, value: Bytes32) {
    unsafe {
        // SAFETY: caller guarantees `ptr.add(offset)` is 32-byte aligned and valid for 32 bytes.
        _mm256_store_si256(ptr.add(offset).cast::<__m256i>(), value.0);
    }
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn broadcast_i32_bytes32(value: i32) -> Bytes32 {
    Bytes32(_mm256_set1_epi32(value))
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn zero_bytes32() -> Bytes32 {
    Bytes32(_mm256_setzero_si256())
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn zero_lane_mask_u32x8(value: Bytes32, zero: Bytes32) -> u32 {
    _mm256_movemask_ps(_mm256_castsi256_ps(_mm256_cmpeq_epi32(value.0, zero.0))) as u32
}

#[target_feature(enable = "avx2,avx512vnni,avx512vl")]
pub(crate) unsafe fn dpbusd_i32x8(acc: I32x8, input: Bytes32, weights: Bytes32) -> I32x8 {
    I32x8(_mm256_dpbusd_epi32(acc.0, input.0, weights.0))
}
