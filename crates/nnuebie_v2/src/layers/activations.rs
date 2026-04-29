use crate::constants::WEIGHT_SCALE_BITS;

#[inline(always)]
pub fn clipped_relu(input: &[i32], output: &mut [u8]) {
    debug_assert!(output.len() >= input.len());

    for (dst, &value) in output.iter_mut().zip(input.iter()) {
        *dst = (value >> WEIGHT_SCALE_BITS).clamp(0, 127) as u8;
    }
}

#[inline(always)]
pub fn sqr_clipped_relu(input: &[i32], output: &mut [u8]) {
    debug_assert!(output.len() >= input.len());

    for (dst, &value) in output.iter_mut().zip(input.iter()) {
        let squared = (i64::from(value) * i64::from(value)) >> (2 * WEIGHT_SCALE_BITS + 7);
        *dst = squared.min(127) as u8;
    }
}
