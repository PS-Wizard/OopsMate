#[cfg(not(target_arch = "x86_64"))]
compile_error!("nnue requires x86_64");

mod aligned;
mod arch;
mod compat;
mod constants;
mod context;
mod eval;
mod features;
mod finny;
mod layers;
mod layout;
mod loader;
mod network;
mod transform;
mod update;

pub(crate) use context::NnueContext;
pub(crate) use network::NnueNetworks;

use super::EvalProvider;
use crate::Position;
use std::sync::OnceLock;

static NNUE_NETWORKS: OnceLock<NnueNetworks> = OnceLock::new();

#[derive(Clone)]
pub struct NnueProvider {
    networks: &'static NnueNetworks,
}

impl NnueProvider {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            networks: networks(),
        }
    }
}

impl Default for NnueProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl EvalProvider for NnueProvider {
    type State = NnueContext;
    type Undo = ();

    #[inline(always)]
    fn new_state(&self, pos: &Position) -> Self::State {
        let mut ctx = NnueContext::new();
        self.sync(&mut ctx, pos);
        ctx
    }

    #[inline(always)]
    fn sync(&self, state: &mut Self::State, pos: &Position) {
        self.networks.reset_context(pos, state);
    }

    #[inline(always)]
    fn eval(&self, pos: &Position, state: &mut Self::State) -> i32 {
        self.networks.evaluate(pos, state).final_cp
    }

    #[inline(always)]
    fn update_on_move(&self, state: &mut Self::State, pos: &Position, mv: crate::Move) -> Self::Undo {
        state.push_move(pos, mv.into());
    }

    #[inline(always)]
    fn update_on_undo(&self, state: &mut Self::State, _undo: Self::Undo) {
        state.pop();
    }

    #[inline(always)]
    fn update_on_null_move(&self, state: &mut Self::State, _pos: &Position) {
        state.push_null_move();
    }

    #[inline(always)]
    fn update_on_undo_null(&self, state: &mut Self::State) {
        state.pop();
    }
}

fn networks() -> &'static NnueNetworks {
    NNUE_NETWORKS
        .get_or_init(|| NnueNetworks::load_default().expect("failed to load embedded nnue networks"))
}
