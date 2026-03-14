use crate::aligned::AlignedBuffer;

mod core;
mod simd;

type UpdateSinglePassFn = unsafe fn(&[i16], &mut [i16], &[*const i16], &[*const i16]);
type FeatureUpdateFn = unsafe fn(&mut [i16], &[i16]);
type RefreshFn = unsafe fn(&mut [i16], &[i16], &AlignedBuffer<i16>, &[usize]);

/// Holds the per-perspective accumulator data for one network size.
#[derive(Clone)]
pub struct Accumulator<const SIZE: usize> {
    pub accumulation: [AlignedBuffer<i16>; 2],
    pub psqt_accumulation: [[i32; 8]; 2],
    pub computed: [bool; 2],
    add_feature_fn: FeatureUpdateFn,
    remove_feature_fn: FeatureUpdateFn,
    update_single_pass_fn: UpdateSinglePassFn,
    refresh_fn: Option<RefreshFn>,
}

impl<const SIZE: usize> Default for Accumulator<SIZE> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accumulator_new() {
        let acc = Accumulator::<128>::new();
        assert_eq!(acc.accumulation[0].len(), 128);
        assert_eq!(acc.accumulation[1].len(), 128);
        assert_eq!(acc.computed, [false, false]);
    }
}
