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

    /// Calculate search depth based on time control
    pub fn calculate_depth(&self, is_white: bool) -> u8 {
        // If depth is explicitly set, use it
        if let Some(d) = self.depth {
            return d;
        }

        // If infinite search
        if self.infinite {
            return 6; // Or some max depth
        }

        // If movetime is set
        if let Some(mt) = self.movetime {
            return self.depth_from_time(mt);
        }

        // Use remaining time
        let our_time = if is_white {
            self.wtime.unwrap_or(30000)
        } else {
            self.btime.unwrap_or(30000)
        };

        let increment = if is_white { self.winc } else { self.binc };

        let time_for_move = (our_time / 30) + (increment / 2);
        self.depth_from_time(time_for_move)
    }

    fn depth_from_time(&self, time_ms: u64) -> u8 {
        match time_ms {
            t if t > 10000 => 6,
            t if t > 5000 => 5,
            t if t > 1000 => 4,
            _ => 3,
        }
    }
}
