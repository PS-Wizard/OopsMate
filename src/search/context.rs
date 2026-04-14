use super::ordering::MoveHistory;
use crate::{eval::EvalProvider, tpt::TranspositionTable, Position};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

const NODE_TIME_CHECK_MASK: u64 = 63;

/// Mutable counters and stop state carried through a single search.
pub(crate) struct SearchStats {
    pub(crate) nodes: u64,
    pub(crate) tt_hits: u64,
    stop_signal: Option<Arc<AtomicBool>>,
    start_time: Instant,
    hard_time_ms: Option<u64>,
}

impl SearchStats {
    pub(crate) fn new(
        stop_signal: Option<Arc<AtomicBool>>,
        start_time: Instant,
        hard_time_ms: Option<u64>,
    ) -> Self {
        Self {
            nodes: 0,
            tt_hits: 0,
            stop_signal,
            start_time,
            hard_time_ms,
        }
    }

    #[inline(always)]
    pub(crate) fn should_stop(&self) -> bool {
        if self.nodes & NODE_TIME_CHECK_MASK == 0 {
            if let Some(max_time) = self.hard_time_ms {
                if self.start_time.elapsed().as_millis() as u64 >= max_time {
                    if let Some(signal) = &self.stop_signal {
                        signal.store(true, Ordering::Relaxed);
                    }
                    return true;
                }
            }

            if let Some(signal) = &self.stop_signal {
                if signal.load(Ordering::Relaxed) {
                    return true;
                }
            }
        }

        false
    }

    #[inline(always)]
    pub(crate) fn elapsed_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }
}

pub(crate) struct SearchContext<'a, E: EvalProvider> {
    pub(crate) tt: &'a mut TranspositionTable,
    pub(crate) eval: &'a E,
    pub(crate) eval_state: Box<E::State>,
    pub(crate) history: MoveHistory,
    pub(crate) stats: SearchStats,
}

impl<'a, E: EvalProvider> SearchContext<'a, E> {
    pub(crate) fn new(
        pos: &Position,
        eval: &'a E,
        tt: &'a mut TranspositionTable,
        stop_signal: Arc<AtomicBool>,
        hard_time_ms: Option<u64>,
        start_time: Instant,
    ) -> Self {
        Self {
            tt,
            eval,
            eval_state: Box::new(eval.new_state(pos)),
            history: MoveHistory::new(),
            stats: SearchStats::new(Some(stop_signal), start_time, hard_time_ms),
        }
    }
}
