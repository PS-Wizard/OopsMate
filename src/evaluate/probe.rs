use super::{
    mapping::{collect_pieces, map_color, material_count},
    networks::networks,
    EvalProbe,
};
use crate::Position;
use nnuebie::uci::to_centipawns;
use std::cell::RefCell;

thread_local! {
    static THREAD_LOCAL_PROBE: RefCell<Option<EvalProbe>> = const { RefCell::new(None) };
}

#[inline(always)]
/// Creates a fresh probe synchronized with `pos`.
pub fn new_probe(pos: &Position) -> EvalProbe {
    let mut probe = EvalProbe::from_networks(networks());
    sync_probe(&mut probe, pos);
    probe
}

#[inline(always)]
/// Synchronizes an existing probe with the current board state.
pub fn sync_probe(probe: &mut EvalProbe, pos: &Position) {
    let pieces = collect_pieces(pos);
    probe.set_position(&pieces, pos.halfmove as i32);
}

#[inline(always)]
/// Evaluates `pos` using an already synchronized probe.
pub fn evaluate_with_probe(pos: &Position, probe: &mut EvalProbe) -> i32 {
    let internal = probe.evaluate(map_color(pos.side_to_move));
    to_centipawns(internal, material_count(pos))
}

/// Evaluates `pos` using a thread-local probe.
pub fn evaluate(pos: &Position) -> i32 {
    THREAD_LOCAL_PROBE.with(|slot| {
        let mut slot = slot.borrow_mut();
        let probe = slot.get_or_insert_with(|| EvalProbe::from_networks(networks()));
        sync_probe(probe, pos);
        evaluate_with_probe(pos, probe)
    })
}
