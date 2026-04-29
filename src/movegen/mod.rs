//! Legal move generation.
//!
//! The move generator computes one legality analysis per node, then emits moves
//! by stage so search can avoid work it does not need.

mod analysis;
mod attacks;
mod generate;
mod king;
mod leapers;
mod pawns;
pub mod see;
mod sliders;
mod stage;

pub use analysis::{analyze, Analysis};
pub use attacks::{is_square_attacked, is_square_attacked_with_blockers};
pub use generate::{
    generate_all, generate_all_with_analysis, generate_captures, generate_captures_with_analysis,
    generate_evasions, generate_evasions_with_analysis, generate_quiets,
    generate_quiets_with_analysis,
};
pub use stage::GenerationStage;
