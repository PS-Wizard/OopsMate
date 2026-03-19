use oops_mate::{search::init_lmr, uci::UciEngine, PestoProvider};
use strikes::warmup_attack_tables;

fn main() {
    let mut engine = UciEngine::new(PestoProvider::new());
    warmup_attack_tables();
    init_lmr();
    engine.run();
}
