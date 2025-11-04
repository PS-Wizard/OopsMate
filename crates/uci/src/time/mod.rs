use evaluation::search::iterative_deepening::SearchLimits;

/// Struct to parse the time control from UCI
pub struct TimeControl {
    pub wtime: Option<u64>,
    pub btime: Option<u64>,
    pub winc: u64,
    pub binc: u64,
    pub movestogo: Option<u32>,
    pub movetime: Option<u64>,
    pub depth: Option<u8>,
    pub infinite: bool,
}

impl TimeControl {
    /// Returns a new time control with empty values
    pub fn new() -> Self {
        Self {
            wtime: None,
            btime: None,
            winc: 0,
            binc: 0,
            movestogo: None,
            movetime: None,
            depth: None,
            infinite: false,
        }
    }

    /// Convert UCI time control to search limits
    pub fn to_search_limits(&self, is_white: bool) -> SearchLimits {
        // Explicit depth
        if let Some(d) = self.depth {
            return SearchLimits::from_depth(d);
        }

        // Infinite search
        if self.infinite {
            return SearchLimits::infinite();
        }

        // Fixed move time
        if let Some(mt) = self.movetime {
            let hard = mt + (mt / 10); // 10% buffer for hard limit
            return SearchLimits::from_time(mt, hard);
        }

        // Calculate time from remaining clock
        let our_time = if is_white {
            self.wtime.unwrap_or(30000)
        } else {
            self.btime.unwrap_or(30000)
        };

        let increment = if is_white { self.winc } else { self.binc };

        let moves_to_go = self.movestogo.unwrap_or(30) as u64;

        // Soft time: time we aim to use for this move
        // (remaining_time / moves_to_go) + (increment * 0.75)
        let soft_time = (our_time / moves_to_go) + (increment * 3 / 4);

        // Hard limit: absolute maximum time we can use
        // Makin sure we don't use more than 1/3 of remaining time in one move
        let hard_limit = soft_time.min(our_time / 3) + increment;

        // Add some buffer to soft time for time loss in communication
        let soft_buffered = soft_time.saturating_sub(50).max(10);
        let hard_buffered = hard_limit.saturating_sub(20).max(20);

        SearchLimits::from_time(soft_buffered, hard_buffered)
    }
}
