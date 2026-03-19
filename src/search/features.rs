pub(crate) const PVS: bool = cfg!(feature = "pvs");
pub(crate) const ASPIRATION_WINDOWS: bool = cfg!(feature = "aspiration-windows");
pub(crate) const IID: bool = cfg!(feature = "iid");
pub(crate) const SINGULAR_EXTENSIONS: bool = cfg!(feature = "singular-extensions");
pub(crate) const CHECK_EXTENSIONS: bool = cfg!(feature = "check-extensions");

pub(crate) const NULL_MOVE: bool = cfg!(feature = "null-move");
pub(crate) const LMR: bool = cfg!(feature = "lmr");
pub(crate) const FUTILITY: bool = cfg!(feature = "futility");
pub(crate) const REVERSE_FUTILITY: bool = cfg!(feature = "reverse-futility");
pub(crate) const RAZORING: bool = cfg!(feature = "razoring");
pub(crate) const PROBCUT: bool = cfg!(feature = "probcut");

pub(crate) const TT_CUTOFFS: bool = cfg!(feature = "tt-cutoffs");
pub(crate) const TT_MOVE_ORDERING: bool = cfg!(feature = "tt-move-ordering");
pub(crate) const KILLER_MOVES: bool = cfg!(feature = "killer-moves");
pub(crate) const HISTORY_HEURISTIC: bool = cfg!(feature = "history-heuristic");
pub(crate) const SEE: bool = cfg!(feature = "see");
