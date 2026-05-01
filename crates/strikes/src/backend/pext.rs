#[inline(always)]
#[must_use]
pub fn slider_index(occupied: u64, mask: u64) -> usize {
    unsafe { pext(occupied, mask) as usize }
}

#[target_feature(enable = "bmi2")]
unsafe fn pext(value: u64, mask: u64) -> u64 {
    core::arch::x86_64::_pext_u64(value, mask)
}
