use oops_mate::{search::init_lmr, uci::UciEngine, NnueProvider};
use strikes::warmup_attack_tables;

fn main() {
    let mut engine = UciEngine::new(NnueProvider::new());
    warmup_attack_tables();
    init_lmr();
    engine.run();
}
