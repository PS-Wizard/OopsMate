#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GenerationStage {
    All = 0,
    Captures = 1,
    Quiets = 2,
    Evasions = 3,
}

#[inline(always)]
pub(crate) const fn include_quiets<const STAGE: u8>() -> bool {
    STAGE != GenerationStage::Captures as u8
}

#[inline(always)]
pub(crate) const fn include_captures<const STAGE: u8>() -> bool {
    STAGE != GenerationStage::Quiets as u8
}

#[inline(always)]
pub(crate) const fn is_evasions<const STAGE: u8>() -> bool {
    STAGE == GenerationStage::Evasions as u8
}
