use crate::Move;

#[inline(always)]
pub(crate) fn pick_next_move(moves: &mut [Move], scores: &mut [i32], index: usize) {
    if index >= moves.len() {
        return;
    }

    let mut best_idx = index;
    let mut best_score = unsafe { *scores.get_unchecked(index) };

    for i in (index + 1)..moves.len() {
        let score = unsafe { *scores.get_unchecked(i) };
        if score > best_score {
            best_score = score;
            best_idx = i;
        }
    }

    if best_idx != index {
        moves.swap(index, best_idx);
        scores.swap(index, best_idx);
    }
}
