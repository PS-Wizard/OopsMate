use oops_mate::{pruning::init_lmr, uci::UciEngine};
use strikes::warmup_attack_tables;

fn main() {
    let mut engine = UciEngine::new();
    // Precomputed Table Inits
    warmup_attack_tables();
    init_lmr();
    engine.run();
}