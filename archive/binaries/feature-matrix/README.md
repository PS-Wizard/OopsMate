# Feature Matrix Builds

This folder stores release binaries for two experiment sets across both eval backends:

- `baseline-add`: baseline build plus one added technique or dependency-aware bundle
- `full-subtract`: full build plus one removed technique or dependency-aware TT bundle removal

Eval backends:

- `nnue`
- `pesto`

Dependency-aware variants:

- `add-iid` means `tt-cutoffs,iid`
- `add-singular-extensions` means `tt-cutoffs,singular-extensions`
- `add-tt-move-ordering` means `tt-cutoffs,tt-move-ordering`
- `sub-tt-stack` removes `tt-cutoffs`, `iid`, `singular-extensions`, and `tt-move-ordering` together
