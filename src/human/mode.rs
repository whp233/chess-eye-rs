//! mode.rs — 对应 human_mode.py
//! get_human_move / get_human_move_from_raw 唯一随机 _weighted_pick
//! MISS_RATE 0.20 [40,160] + _position_allows_miss 四门槛 + decisive 150

use rand::Rng;

pub const MAX_EVAL_DROP_CP: i32 = 300;
pub const MISS_RATE: f64 = 0.20;
pub const MISS_CPL_MIN: i32 = 40;
pub const MISS_CPL_MAX: i32 = 160;
pub const MISS_CANDIDATES: usize = 6;
pub const ENDGAME_MATERIAL: i32 = 26;
pub const LOST_EVAL_CP: i32 = -150;
pub const DECISIVE_GAP_CP: i32 = 150;
pub const HAND_SPEED_S: f64 = 3.0;

pub fn weighted_pick(cands: &[(String, i32)], weights: &[f64]) -> (String, usize) {
    if cands.len() == 1 || weights.is_empty() { return (cands[0].0.clone(), 0); }
    let mut rng = rand::thread_rng();
    let total: f64 = weights.iter().sum();
    if total <= 0.0 { return (cands[0].0.clone(), 0); }
    let mut r: f64 = rng.gen::<f64>() * total;
    let mut cum = 0.0;
    for (i, &w) in weights.iter().enumerate() {
        cum += w;
        if r <= cum { return (cands[i].0.clone(), i); }
    }
    (cands.last().unwrap().0.clone(), cands.len()-1)
}

/// M1: 实现 _book_move / _candidate_list / _filter_blunders / _deliberate_miss / get_human_move
pub fn get_human_move_stub() {}
