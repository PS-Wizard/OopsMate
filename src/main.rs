use oops_mate::{lmr::init, uci::UciEngine};
use strikes::warmup_attack_tables;

fn main() {
    let mut engine = UciEngine::new();
    // Precomputed Table Inits
    warmup_attack_tables();
    init();
    engine.run();
}
