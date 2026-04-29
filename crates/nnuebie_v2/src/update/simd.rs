#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub(super) fn is_32_byte_aligned<T>(ptr: *const T) -> bool {
    (ptr as usize & 31) == 0
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub(super) fn can_vectorize_i16(a: *const i16, b: *const i16, len: usize) -> bool {
    len % 16 == 0 && is_32_byte_aligned(a) && is_32_byte_aligned(b)
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub(super) fn can_vectorize_i16_3(a: *const i16, b: *const i16, c: *const i16, len: usize) -> bool {
    len % 16 == 0 && is_32_byte_aligned(a) && is_32_byte_aligned(b) && is_32_byte_aligned(c)
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub(super) fn can_vectorize_i16_4(
    a: *const i16,
    b: *const i16,
    c: *const i16,
    d: *const i16,
    len: usize,
) -> bool {
    len % 16 == 0
        && is_32_byte_aligned(a)
        && is_32_byte_aligned(b)
        && is_32_byte_aligned(c)
        && is_32_byte_aligned(d)
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub(super) fn can_vectorize_i16_5(
    a: *const i16,
    b: *const i16,
    c: *const i16,
    d: *const i16,
    e: *const i16,
    len: usize,
) -> bool {
    can_vectorize_i16_4(a, b, c, d, len) && is_32_byte_aligned(e)
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub(super) fn can_vectorize_i16_6(
    a: *const i16,
    b: *const i16,
    c: *const i16,
    d: *const i16,
    e: *const i16,
    f: *const i16,
    len: usize,
) -> bool {
    can_vectorize_i16_5(a, b, c, d, e, len) && is_32_byte_aligned(f)
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub(super) fn can_vectorize_i32(a: *const i32, b: *const i32) -> bool {
    is_32_byte_aligned(a) && is_32_byte_aligned(b)
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub(super) fn can_vectorize_i32_3(a: *const i32, b: *const i32, c: *const i32) -> bool {
    can_vectorize_i32(a, b) && is_32_byte_aligned(c)
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub(super) fn can_vectorize_i32_4(
    a: *const i32,
    b: *const i32,
    c: *const i32,
    d: *const i32,
) -> bool {
    can_vectorize_i32_3(a, b, c) && is_32_byte_aligned(d)
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub(super) fn can_vectorize_i32_5(
    a: *const i32,
    b: *const i32,
    c: *const i32,
    d: *const i32,
    e: *const i32,
) -> bool {
    can_vectorize_i32_4(a, b, c, d) && is_32_byte_aligned(e)
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub(super) fn can_vectorize_i32_6(
    a: *const i32,
    b: *const i32,
    c: *const i32,
    d: *const i32,
    e: *const i32,
    f: *const i32,
) -> bool {
    can_vectorize_i32_5(a, b, c, d, e) && is_32_byte_aligned(f)
}
