mod delta;
mod mapping;
mod networks;
mod probe;
#[cfg(test)]
mod tests;

pub use delta::{apply_move, apply_null_move, undo_move, undo_null_move};
pub use probe::{evaluate, evaluate_with_probe, new_probe, sync_probe};

pub type EvalProbe = nnuebie::NNUEProbe;
pub type EvalMoveDelta = nnuebie::MoveDelta;
