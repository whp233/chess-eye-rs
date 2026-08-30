//! error.rs — 对应 human_error.py 完整翻译
//! 纯函数零随机，四源

use std::collections::HashMap;
use shakmaty::{Chess, Role, Position};

#[derive(Debug, Clone)]
pub struct Profile {
    pub calculation: f64,
    pub evaluation: f64,
    pub planning: f64,
    pub time_pressure: f64,
}

fn clock_frac(remaining_ms: Option<i64>, speed: &str) -> f64 {
    let Some(ms) = remaining_ms else { return 1.0; };
    if ms <= 0 { return 1.0; }
    let budget: i64 = match speed {
        "bullet" => 60_000, "blitz" => 180_000, "rapid" => 600_000,
        "classical" => 1_500_000, "correspondence" => 86_400_000, _ => 600_000,
    };
    ((ms as f64) / (budget as f64)).min(1.0)
}
fn time_pressure(frac: f64) -> f64 {
    if frac >= 0.35 { 0.0 } else { ((0.35 - frac)/0.25).min(1.0) }
}
fn tactical(chess: &Chess) -> f64 {
    if chess.is_check() { return 1.0; }
    for m in chess.legal_moves() {
        if chess.is_capture(m) {
            if let Some(pc) = chess.board().piece_at(m.to()) {
                if pc != Role::Pawn { return 1.0; }
            }
        }
    }
    0.0
}
fn weakness(elo: Option<u16>) -> f64 {
    match elo {
        None => 0.4,
        Some(e) if e <= 800 => 1.0,
        Some(e) if e >= 2400 => 0.0,
        Some(e) => (2400 - e) as f64 / 1600.0,
    }
}

pub fn profile(chess: &Chess, elo: Option<u16>, remaining_ms: Option<i64>, speed: &str, state: &HashMap<String,String>) -> Profile {
    let tac = tactical(chess);
    let frac = clock_frac(remaining_ms, speed);
    let tp = time_pressure(frac);
    let w = weakness(elo);
    let legal = chess.legal_moves().len() as f64;
    let simple = if tac == 0.0 && legal <= 20.0 { 1.0 } else { 0.0 };
    let normal = if tac == 0.0 && legal > 20.0 && legal <= 40.0 { 1.0 } else { 0.0 };
    let calc = (0.8*tac + 0.5*tp + 0.2*w).min(1.0);
    let eval_ = ((0.5*normal + 0.4*simple)*(0.4+0.6*w) + 0.2*tp).min(1.0);
    let plan_age: i32 = state.get("plan_age").and_then(|s| s.parse().ok()).unwrap_or(0);
    let plan_ = (0.5*normal + 0.6*simple)*(0.5+0.1*plan_age as f64)*(0.5+0.5*w);
    let plan_ = plan_.min(1.0);
    Profile { calculation: (calc*1000.0).round()/1000.0, evaluation: (eval_*1000.0).round()/1000.0, planning: (plan_*1000.0).round()/1000.0, time_pressure: (tp*1000.0).round()/1000.0 }
}

pub fn profile_stub() -> Profile {
    Profile { calculation: 0.0, evaluation: 0.0, planning: 0.0, time_pressure: 0.0 }
}
