use crate::Move;

#[derive(Clone, Copy)]
pub(crate) struct NodeState {
    pub(crate) allow_null: bool,
    pub(crate) pv_node: bool,
    pub(crate) excluded_move: Option<Move>,
    pub(crate) ply: usize,
}

impl NodeState {
    pub(crate) const fn new(
        allow_null: bool,
        pv_node: bool,
        excluded_move: Option<Move>,
        ply: usize,
    ) -> Self {
        Self {
            allow_null,
            pv_node,
            excluded_move,
            ply,
        }
    }
}
