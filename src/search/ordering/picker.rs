use std::mem::MaybeUninit;

use crate::memory::MoveHistory;
use crate::movegen::{
    generate_captures_with_analysis, generate_evasions_with_analysis,
    generate_quiets_with_analysis, Analysis,
};
use crate::{Move, MoveCollector, Position};

use super::score_move;

const MAX_MOVES: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TtMode {
    ValidateInStage,
    BlindTrust,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Tt,
    Captures,
    Quiets,
    Evasions,
    Done,
}

pub(crate) struct MovePicker {
    phase: Phase,
    tt_move: Move,
    tt_mode: TtMode,
    tt_yielded: bool,
    in_check: bool,
    allow_quiets: bool,
    stage_loaded: bool,
    moves: MoveCollector,
    scores: MaybeUninit<[i32; MAX_MOVES]>,
    next: usize,
}

impl MovePicker {
    #[inline(always)]
    pub(crate) fn new(
        analysis: &Analysis,
        tt_move: Option<Move>,
        tt_mode: TtMode,
        allow_quiets: bool,
    ) -> Self {
        Self {
            phase: Phase::Tt,
            tt_move: tt_move.unwrap_or(Move(0)),
            tt_mode,
            tt_yielded: false,
            in_check: analysis.in_check(),
            allow_quiets,
            stage_loaded: false,
            moves: MoveCollector::new(),
            scores: MaybeUninit::uninit(),
            next: 0,
        }
    }

    pub(crate) fn next_move(
        &mut self,
        pos: &Position,
        analysis: &Analysis,
        history: Option<&MoveHistory>,
        ply: usize,
    ) -> Option<Move> {
        loop {
            match self.phase {
                Phase::Tt => {
                    self.phase = self.first_generated_phase();
                    if let Some(mv) = self.try_tt_move(pos, analysis, history, ply) {
                        self.tt_yielded = true;
                        return Some(mv);
                    }
                }
                Phase::Captures | Phase::Quiets | Phase::Evasions => {
                    if !self.stage_loaded {
                        self.load_stage(pos, analysis, history, ply);
                    }

                    if let Some(mv) = self.pick_best() {
                        return Some(mv);
                    }

                    self.advance_phase();
                }
                Phase::Done => return None,
            }
        }
    }

    #[inline(always)]
    fn first_generated_phase(&self) -> Phase {
        if self.in_check {
            Phase::Evasions
        } else {
            Phase::Captures
        }
    }

    fn try_tt_move(
        &mut self,
        pos: &Position,
        analysis: &Analysis,
        history: Option<&MoveHistory>,
        ply: usize,
    ) -> Option<Move> {
        if self.tt_move.0 == 0 {
            return None;
        }

        match self.tt_mode {
            TtMode::BlindTrust => Some(self.tt_move),
            TtMode::ValidateInStage => {
                let validation_phase = if self.in_check {
                    Phase::Evasions
                } else if is_tactical_move(self.tt_move) {
                    Phase::Captures
                } else {
                    Phase::Quiets
                };

                if validation_phase == self.phase {
                    if !self.stage_loaded {
                        self.load_stage(pos, analysis, history, ply);
                    }
                    self.moves
                        .as_slice()
                        .contains(&self.tt_move)
                        .then_some(self.tt_move)
                } else {
                    let mut generated = MoveCollector::new();
                    match validation_phase {
                        Phase::Captures => {
                            generate_captures_with_analysis(pos, analysis, &mut generated)
                        }
                        Phase::Quiets => {
                            generate_quiets_with_analysis(pos, analysis, &mut generated)
                        }
                        Phase::Evasions => {
                            generate_evasions_with_analysis(pos, analysis, &mut generated)
                        }
                        Phase::Tt | Phase::Done => unreachable!(),
                    }
                    generated
                        .as_slice()
                        .contains(&self.tt_move)
                        .then_some(self.tt_move)
                }
            }
        }
    }

    fn load_stage(
        &mut self,
        pos: &Position,
        analysis: &Analysis,
        history: Option<&MoveHistory>,
        ply: usize,
    ) {
        self.moves.clear();
        match self.phase {
            Phase::Captures => generate_captures_with_analysis(pos, analysis, &mut self.moves),
            Phase::Quiets => generate_quiets_with_analysis(pos, analysis, &mut self.moves),
            Phase::Evasions => generate_evasions_with_analysis(pos, analysis, &mut self.moves),
            Phase::Tt | Phase::Done => unreachable!(),
        }

        self.next = 0;
        self.stage_loaded = true;

        let len = self.moves.len();
        for index in 0..len {
            let mv = self.moves.get(index);
            let score = score_move(mv, pos, None, history, ply);
            self.write_score(index, score);
        }
    }

    fn pick_best(&mut self) -> Option<Move> {
        while self.next < self.moves.len() {
            let mut best = self.next;
            for index in (self.next + 1)..self.moves.len() {
                if self.score(index) > self.score(best) {
                    best = index;
                }
            }

            self.swap_move(self.next, best);
            self.swap_scores(self.next, best);

            let mv = self.moves.get(self.next);
            self.next += 1;

            if self.tt_yielded && mv.0 == self.tt_move.0 {
                continue;
            }

            return Some(mv);
        }

        None
    }

    fn advance_phase(&mut self) {
        self.stage_loaded = false;
        self.phase = match self.phase {
            Phase::Captures => {
                if self.allow_quiets {
                    Phase::Quiets
                } else {
                    Phase::Done
                }
            }
            Phase::Quiets | Phase::Evasions => Phase::Done,
            Phase::Tt | Phase::Done => Phase::Done,
        };
    }

    #[inline(always)]
    fn write_score(&mut self, index: usize, score: i32) {
        debug_assert!(index < MAX_MOVES);
        unsafe {
            (self.scores.as_mut_ptr() as *mut i32)
                .add(index)
                .write(score);
        }
    }

    #[inline(always)]
    fn score(&self, index: usize) -> i32 {
        debug_assert!(index < self.moves.len());
        unsafe { *((self.scores.as_ptr() as *const i32).add(index)) }
    }

    #[inline(always)]
    fn swap_scores(&mut self, a: usize, b: usize) {
        debug_assert!(a < self.moves.len());
        debug_assert!(b < self.moves.len());
        if a == b {
            return;
        }

        unsafe {
            std::ptr::swap(
                (self.scores.as_mut_ptr() as *mut i32).add(a),
                (self.scores.as_mut_ptr() as *mut i32).add(b),
            );
        }
    }

    #[inline(always)]
    fn swap_move(&mut self, a: usize, b: usize) {
        self.moves.swap(a, b);
    }
}

#[inline(always)]
pub(crate) fn pick_next_move(moves: &mut [Move], scores: &mut [i32], index: usize) {
    if index >= moves.len() {
        return;
    }

    let mut best_idx = index;
    let mut best_score = unsafe { *scores.get_unchecked(index) };

    for i in (index + 1)..moves.len() {
        let score = unsafe { *scores.get_unchecked(i) };
        if score > best_score {
            best_score = score;
            best_idx = i;
        }
    }

    if best_idx != index {
        moves.swap(index, best_idx);
        scores.swap(index, best_idx);
    }
}

#[inline(always)]
fn is_tactical_move(mv: Move) -> bool {
    mv.is_capture() || mv.is_promotion()
}
