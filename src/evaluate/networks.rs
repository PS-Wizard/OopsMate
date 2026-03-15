use nnuebie::NnueNetworks;
use std::sync::{Arc, OnceLock};

const BIG_NETWORK_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/crates/nnuebie/archive/nnue/networks/nn-1c0000000000.nnue"
);
const SMALL_NETWORK_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/crates/nnuebie/archive/nnue/networks/nn-37f18f62d772.nnue"
);

static NNUE_NETWORKS: OnceLock<Arc<NnueNetworks>> = OnceLock::new();

#[inline(always)]
pub fn networks() -> Arc<NnueNetworks> {
    NNUE_NETWORKS
        .get_or_init(|| {
            Arc::new(
                NnueNetworks::new(BIG_NETWORK_PATH, SMALL_NETWORK_PATH)
                    .expect("failed to load embedded nnue networks"),
            )
        })
        .clone()
}
