//! stats.rs — 对应 human_stats.py
//! GameStats: P(best), avg CPL, mistake>=50, blunder>=100, think p50/p90

#[derive(Debug, Default)]
pub struct GameStats {
    pub elo: Option<u16>,
    pub moves: usize,
    pub ranks: Vec<usize>,
    pub cpls: Vec<i32>,
    pub thinks: Vec<f64>,
}

impl GameStats {
    pub fn new() -> Self { Self::default() }
    pub fn reset(&mut self) { *self = Self::default(); }
    pub fn record(&mut self, elo: u16, rank: usize, cpl: i32, think: f64) {
        if self.elo.is_none() { self.elo = Some(elo); }
        self.moves += 1;
        self.ranks.push(rank);
        self.cpls.push(cpl);
        self.thinks.push(think);
    }
    pub fn p_best(&self) -> f64 {
        if self.moves == 0 { return 0.0; }
        self.ranks.iter().filter(|&&r| r==0).count() as f64 / self.moves as f64
    }
}
