use super::analysis::{analyze, Analysis};
use super::king;
use super::leapers;
use super::pawns;
use super::sliders;
use super::stage::{is_evasions, GenerationStage};
use crate::{MoveCollector, Position};

#[inline(always)]
pub fn generate_all(pos: &Position, collector: &mut MoveCollector) {
    generate::<{ GenerationStage::All as u8 }>(pos, collector);
}

#[inline(always)]
pub fn generate_captures(pos: &Position, collector: &mut MoveCollector) {
    generate::<{ GenerationStage::Captures as u8 }>(pos, collector);
}

#[inline(always)]
pub fn generate_quiets(pos: &Position, collector: &mut MoveCollector) {
    generate::<{ GenerationStage::Quiets as u8 }>(pos, collector);
}

#[inline(always)]
pub fn generate_evasions(pos: &Position, collector: &mut MoveCollector) {
    generate::<{ GenerationStage::Evasions as u8 }>(pos, collector);
}

#[inline(always)]
fn generate<const STAGE: u8>(pos: &Position, collector: &mut MoveCollector) {
    let analysis = analyze(pos);
    generate_with_analysis::<STAGE>(pos, &analysis, collector);
}

#[inline(always)]
pub fn generate_all_with_analysis(pos: &Position, analysis: &Analysis, collector: &mut MoveCollector) {
    generate_with_analysis::<{ GenerationStage::All as u8 }>(pos, analysis, collector);
}

#[inline(always)]
pub fn generate_captures_with_analysis(
    pos: &Position,
    analysis: &Analysis,
    collector: &mut MoveCollector,
) {
    generate_with_analysis::<{ GenerationStage::Captures as u8 }>(pos, analysis, collector);
}

#[inline(always)]
pub fn generate_quiets_with_analysis(pos: &Position, analysis: &Analysis, collector: &mut MoveCollector) {
    generate_with_analysis::<{ GenerationStage::Quiets as u8 }>(pos, analysis, collector);
}

#[inline(always)]
pub fn generate_evasions_with_analysis(
    pos: &Position,
    analysis: &Analysis,
    collector: &mut MoveCollector,
) {
    generate_with_analysis::<{ GenerationStage::Evasions as u8 }>(pos, analysis, collector);
}

#[inline(always)]
fn generate_with_analysis<const STAGE: u8>(
    pos: &Position,
    analysis: &Analysis,
    collector: &mut MoveCollector,
) {
    collector.clear();

    if is_evasions::<STAGE>() && !analysis.in_check() {
        return;
    }

    king::generate::<STAGE>(pos, analysis, collector);
    if analysis.double_check() {
        return;
    }

    pawns::generate::<STAGE>(pos, analysis, collector);
    leapers::generate::<STAGE>(pos, analysis, collector);
    sliders::generate::<STAGE>(pos, analysis, collector);
}

impl Position {
    #[inline(always)]
    pub fn generate_moves(&self, collector: &mut MoveCollector) {
        generate_all(self, collector);
    }

    #[inline(always)]
    pub fn generate_captures(&self, collector: &mut MoveCollector) {
        generate_captures(self, collector);
    }
}
