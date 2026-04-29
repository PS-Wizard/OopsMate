use crate::Move;

use super::score::{denormalize_score, normalize_score};

const AGE_PENALTY: i16 = 8;
pub const NO_STATIC_EVAL: i16 = i16::MIN;

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Bound {
    Exact = 1,
    Lower = 2,
    Upper = 3,
}

impl Bound {
    #[inline(always)]
    pub(crate) const fn from_tag(tag: u8) -> Self {
        match tag {
            1 => Self::Exact,
            2 => Self::Lower,
            3 => Self::Upper,
            _ => unreachable!(),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TtHit {
    pub best_move: Move,
    pub score: i32,
    pub static_eval: i16,
    pub depth: u8,
    pub bound: Bound,
}

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug)]
pub(crate) struct TtEntry {
    key32: u32,
    move16: u16,
    score16: i16,
    eval16: i16,
    depth8: u8,
    gen8: u8,
    bound8: u8,
    pad8: u8,
}

impl TtEntry {
    pub(crate) const EMPTY: Self = Self {
        key32: 0,
        move16: 0,
        score16: 0,
        eval16: NO_STATIC_EVAL,
        depth8: 0,
        gen8: 0,
        bound8: 0,
        pad8: 0,
    };

    #[inline(always)]
    pub(crate) const fn is_empty(self) -> bool {
        self.bound8 == 0
    }

    #[inline(always)]
    pub(crate) const fn matches_key(self, key32: u32) -> bool {
        self.bound8 != 0 && self.key32 == key32
    }

    #[inline(always)]
    pub(crate) const fn replacement_value(self, current_generation: u8) -> i16 {
        let age = current_generation.wrapping_sub(self.gen8) as i16;
        self.depth8 as i16 - age * AGE_PENALTY
    }

    #[inline(always)]
    pub(crate) fn to_hit(self, ply: u8) -> TtHit {
        TtHit {
            best_move: Move(self.move16),
            score: denormalize_score(self.score16 as i32, ply),
            static_eval: self.eval16,
            depth: self.depth8,
            bound: Bound::from_tag(self.bound8),
        }
    }

    #[inline(always)]
    pub(crate) fn from_values(
        key32: u32,
        best_move: Move,
        score: i32,
        static_eval: i16,
        depth: u8,
        generation: u8,
        bound: Bound,
        ply: u8,
    ) -> Self {
        let normalized_score = normalize_score(score, ply);
        debug_assert!(normalized_score >= i16::MIN as i32 && normalized_score <= i16::MAX as i32);

        Self {
            key32,
            move16: best_move.0,
            score16: normalized_score as i16,
            eval16: static_eval,
            depth8: depth,
            gen8: generation,
            bound8: bound as u8,
            pad8: 0,
        }
    }
}
