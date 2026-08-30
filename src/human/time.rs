//! time.rs — 对应 human_time.py 完整翻译

use std::collections::HashMap;
use shakmaty::{Chess, Role};

fn base_budget(speed: &str) -> i64 {
    match speed {
        "bullet" => 60, "blitz" => 180, "rapid" => 600, "classical" => 1500, "correspondence" => 86400, _ => 600,
    }
}
fn complexity_tier(chess: &Chess) -> &'static str {
    let legal = chess.legal_moves().len();
    if legal == 0 { return "simple"; }
    if chess.is_check() { return "tactical"; }
    for m in chess.legal_moves() {
        if chess.is_capture(m) {
            if let Some(pc) = chess.board().piece_at(m.to()) {
                if pc != Role::Pawn { return "tactical"; }
            }
        }
    }
    if legal <= 20 { "simple" } else { "normal" }
}
fn clock_factor(remaining_ms: Option<i64>, speed: &str, tempo: &HashMap<String,i32>) -> f64 {
    let Some(ms) = remaining_ms else { return 1.0; };
    if ms <= 0 { return 1.0; }
    let budget = base_budget(speed) * 1000;
    let frac = ms as f64 / budget as f64;
    let fast_streak = tempo.get("fast_streak").copied().unwrap_or(0);
    let boost: f64 = if fast_streak >= 2 { 0.85 } else { 1.0 };
    if frac < 0.15 { 0.15*boost } else if frac < 0.30 { 0.4*boost } else if frac > 0.60 { 1.0*boost } else { 0.7*boost }
}

pub fn calculate(fen: &str, speed: &str, remaining_ms: Option<i64>, mode: &str, tempo: &HashMap<String,i32>) -> Option<f64> {
    if mode != "human" { return None; }
    let chess = {
        use std::str::FromStr; use shakmaty::fen::Fen;
        Fen::from_str(fen).ok().and_then(|f| f.into_position(shakmaty::CastlingMode::Standard).ok()).unwrap_or_default()
    };
    let tier = complexity_tier(&chess);
    let (lo, hi) = match tier { "simple" => (1.0,3.0), "normal" => (5.0,15.0), "tactical" => (15.0,40.0), _ => (5.0,15.0) };
    let mut think: f64 = {
        use rand::Rng; rand::thread_rng().gen_range(lo..hi) * clock_factor(remaining_ms, speed, tempo)
    };
    if rand::random::<f64>() < 0.10 {
        think += rand::random::<f64>() * 3.0 + 2.0;
    }
    Some((think.clamp(0.8, 40.0)*10.0).round()/10.0)
}
