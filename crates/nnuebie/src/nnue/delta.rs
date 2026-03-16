use crate::accumulator_stack::DirtyPiece;
use crate::types::{Piece, Square};
use std::error::Error;
use std::fmt;

/// One board change inside an incremental NNUE update.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeltaChange {
    pub from: Square,
    pub to: Square,
    pub piece_from: Piece,
    pub piece_to: Piece,
}

impl DeltaChange {
    /// Builds a raw board-state change.
    pub const fn new(from: Square, to: Square, piece_from: Piece, piece_to: Piece) -> Self {
        Self {
            from,
            to,
            piece_from,
            piece_to,
        }
    }

    /// Encodes a move from `from` to `to`.
    pub const fn move_piece(from: Square, to: Square, piece_from: Piece, piece_to: Piece) -> Self {
        Self::new(from, to, piece_from, piece_to)
    }

    /// Encodes removing `piece` from `square`.
    pub const fn removal(square: Square, piece: Piece) -> Self {
        Self::new(square, square, piece, Piece::None)
    }

    /// Encodes adding `piece` onto `square`.
    pub const fn addition(square: Square, piece: Piece) -> Self {
        Self::new(square, square, Piece::None, piece)
    }

    pub(crate) const fn is_empty(self) -> bool {
        matches!(self.piece_from, Piece::None) && matches!(self.piece_to, Piece::None)
    }
}

/// Errors returned when constructing an incremental move delta.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeltaError {
    EmptyChange,
    TooManyChanges,
}

impl fmt::Display for DeltaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyChange => f.write_str("delta change must modify at least one piece"),
            Self::TooManyChanges => f.write_str("move delta exceeds the 3-change limit"),
        }
    }
}

impl Error for DeltaError {}

/// Compact representation of a chess move as up to three piece changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MoveDelta {
    changes: [DeltaChange; 3],
    len: usize,
    next_rule50: i32,
}

impl Default for MoveDelta {
    fn default() -> Self {
        Self::new(0)
    }
}

impl MoveDelta {
    /// Maximum number of piece changes supported by one delta.
    pub const MAX_CHANGES: usize = 3;

    /// Creates an empty delta with the supplied next rule-50 counter.
    pub const fn new(next_rule50: i32) -> Self {
        Self {
            changes: [DeltaChange::new(0, 0, Piece::None, Piece::None); 3],
            len: 0,
            next_rule50,
        }
    }

    /// Creates the delta used for a null move.
    pub const fn null(next_rule50: i32) -> Self {
        Self::new(next_rule50)
    }

    /// Returns the number of stored board changes.
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` when the delta contains no board changes.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the rule-50 value that should apply after the delta.
    pub const fn next_rule50(&self) -> i32 {
        self.next_rule50
    }

    /// Overwrites the post-move rule-50 value.
    pub fn set_next_rule50(&mut self, next_rule50: i32) {
        self.next_rule50 = next_rule50;
    }

    /// Appends a single change to the delta.
    pub fn push(&mut self, change: DeltaChange) -> Result<(), DeltaError> {
        if change.is_empty() {
            return Err(DeltaError::EmptyChange);
        }
        if self.len >= Self::MAX_CHANGES {
            return Err(DeltaError::TooManyChanges);
        }

        self.changes[self.len] = change;
        self.len += 1;
        Ok(())
    }

    /// Convenience wrapper around [`DeltaChange::new`].
    pub fn push_change(
        &mut self,
        from: Square,
        to: Square,
        piece_from: Piece,
        piece_to: Piece,
    ) -> Result<(), DeltaError> {
        self.push(DeltaChange::new(from, to, piece_from, piece_to))
    }

    /// Convenience wrapper around [`DeltaChange::move_piece`].
    pub fn push_move(
        &mut self,
        from: Square,
        to: Square,
        piece_from: Piece,
        piece_to: Piece,
    ) -> Result<(), DeltaError> {
        self.push(DeltaChange::move_piece(from, to, piece_from, piece_to))
    }

    /// Appends a removal change.
    pub fn push_removal(&mut self, square: Square, piece: Piece) -> Result<(), DeltaError> {
        self.push(DeltaChange::removal(square, piece))
    }

    /// Appends an addition change.
    pub fn push_addition(&mut self, square: Square, piece: Piece) -> Result<(), DeltaError> {
        self.push(DeltaChange::addition(square, piece))
    }

    /// Returns the initialized change slice.
    pub fn changes(&self) -> &[DeltaChange] {
        &self.changes[..self.len]
    }

    pub(crate) fn to_dirty_piece(self) -> DirtyPiece {
        let mut dirty = DirtyPiece::new();
        for change in self.changes() {
            dirty.add_change(
                change.from,
                change.to,
                change.piece_from.index(),
                change.piece_to.index(),
            );
        }
        dirty
    }
}
