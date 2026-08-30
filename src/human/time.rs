//! time.rs — 对应 human_time.py
//! calculate(fen, speed, remaining_ms, mode, tempo) 按 complexity tier + clock_factor

use std::collections::HashMap;

pub fn calculate(_fen: &str, _speed: &str, _remaining_ms: Option<i64>, mode: &str, _tempo: &HashMap<String, i32>) -> Option<f64> {
    if mode != "human" { return None; }
    // M1: tier simple/normal/tactical + uniform jitter + clock_factor + 10% hesitation
    Some(2.5)
}
