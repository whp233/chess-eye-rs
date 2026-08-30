//! accuracy.rs — 对应 human_accuracy.py 完整翻译
//! 有效性：P(best)=eff, tail 塑形只动 (1-eff)，单掷唯一 random 在调用方

use std::collections::HashMap;
use shakmaty::{Chess, Position};

use crate::human::error;

// 闭区间权威表 (lo < elo <= hi)
pub const ELO_WEIGHTS: &[((u16, u16), [f64; 5])] = &[
    ((0, 800),    [0.25, 0.22, 0.20, 0.18, 0.15]),
    ((800, 1200), [0.35, 0.25, 0.18, 0.12, 0.10]),
    ((1200, 1600),[0.50, 0.25, 0.12, 0.08, 0.05]),
    ((1600, 2000),[0.65, 0.22, 0.10, 0.03, 0.00]),
    ((2000, 2400),[0.85, 0.12, 0.03, 0.00, 0.00]),
    ((2400, 9999),[0.95, 0.05, 0.00, 0.00, 0.00]),
];
pub const ELO_BASE_ACC: &[((u16, u16), f64)] = &[
    ((0, 800), 0.70),
    ((800, 1200), 0.78),
    ((1200, 1600),0.85),
    ((1600, 2000),0.90),
    ((2000, 9999),0.95),
];
pub const TACTICAL_TAIL: [f64; 4] = [0.15, 0.25, 0.30, 0.30];

pub fn elo_weights(elo: u16) -> [f64; 5] {
    for &((lo, hi), w) in ELO_WEIGHTS {
        if lo < elo && elo <= hi { return w; }
    }
    [0.65, 0.22, 0.10, 0.03, 0.00]
}
pub fn elo_base_acc(elo: u16) -> f64 {
    for &((lo, hi), a) in ELO_BASE_ACC {
        if lo < elo && elo <= hi { return a; }
    }
    0.85
}

fn tactical(chess: &Chess) -> bool {
    if chess.is_check() { return true; }
    for m in chess.legal_moves() {
        if chess.is_capture(m) {
            // 仅非兵被吃算战术（镜像 Python）
            if let Some(pc) = chess.board().piece_at(m.to()) {
                if pc != shakmaty::Role::Pawn { return true; }
            }
        }
    }
    false
}

/// _effective_accuracy: accuracy + 位置/时钟/jitter → clamp 0.50..0.99
pub fn effective_accuracy(accuracy: f64, chess: &Chess, remaining_ms: Option<i64>, speed: &str) -> f64 {
    let mut adj: f64 = 0.0;
    let tac = tactical(chess);
    let legal = chess.legal_moves().len() as i32;
    if tac {
        adj -= 0.15;
    } else if legal <= 20 {
        adj += 0.0;
    }
    if chess.ply() <= 16 { // fullmove_number <=8  → ply <=16
        adj += 0.03;
    }
    if let Some(ms) = remaining_ms {
        if ms > 0 {
            let frac = ms as f64 / 600_000.0;
            if frac > 0.8 { adj += 0.05; }
            else if frac < 0.10 { adj -= 0.15; }
            else if frac < 0.30 { adj -= 0.08; }
        }
    }
    // jitter ±0.02 唯一随机源（调用方需单次）
    let jitter: f64 = {
        use rand::Rng;
        rand::thread_rng().gen_range(-0.02..0.02)
    };
    adj += jitter;
    (accuracy + adj).clamp(0.50, 0.99)
}

/// _weights_for_pool — 保持 P(best)=eff 硬契约
pub fn weights_for_pool(eff: f64, elo: Option<u16>, cands: &[(String, i32)], errors: Option<&crate::human::error::Profile>, state: &HashMap<String, String>, chess: &Chess) -> Vec<f64> {
    let n = cands.len();
    if n <= 1 { return vec![1.0; n]; }
    if elo.is_none() {
        // 几何衰减兼容
        let w: Vec<f64> = (0..n).map(|i| eff.powi(i as i32)).collect();
        let total: f64 = w.iter().sum();
        return w.into_iter().map(|x| x/total).collect();
    }
    let elo_v = elo.unwrap();
    let tactical_flag = errors.map(|e| e.calculation > 0.6).unwrap_or(false); // 简化：用 calculation 近似
    let base: Vec<f64> = if tactical_flag {
        TACTICAL_TAIL[..n-1].to_vec()
    } else {
        let w = elo_weights(elo_v);
        w[1..n].to_vec()
    };
    let s: f64 = base.iter().sum::<f64>().max(1.0);
    // M1 再接 _shape_tail；此处先直通
    let shaped = shape_tail(&base, errors, chess, cands, state);
    let ts: f64 = shaped.iter().sum();
    let shaped = if ts > 0.0 && (ts - s).abs() > 1e-12 {
        shaped.into_iter().map(|x| x * s / ts).collect()
    } else { base };
    let mut w = Vec::with_capacity(n);
    w.push(eff);
    for x in shaped { w.push((1.0 - eff) * x / s); }
    let total: f64 = w.iter().sum();
    w.into_iter().map(|x| x/total).collect()
}

fn shape_tail(tail: &[f64], errors: Option<&crate::human::error::Profile>, _chess: &Chess, _cands: &[(String,i32)], _state: &HashMap<String,String>) -> Vec<f64> {
    if tail.is_empty() { return tail.to_vec(); }
    let Some(e) = errors else { return tail.to_vec(); };
    if e.calculation == 0.0 && e.evaluation == 0.0 && e.planning == 0.0 && e.time_pressure == 0.0 {
        return tail.to_vec();
    }
    let n = tail.len();
    let mut bias = vec![1.0; n];
    if e.calculation != 0.0 {
        for i in 0..n {
            bias[i] *= if i <= 1 { 1.0 + 0.55*e.calculation } else { (1.0 - 0.45*e.calculation).max(0.20) };
        }
    }
    if e.evaluation != 0.0 && n > 1 {
        for i in 0..n {
            let ramp = i as f64 / (n as f64 - 1.0);
            bias[i] *= 1.0 - 0.40*e.evaluation + 0.80*e.evaluation*ramp;
        }
    }
    if e.time_pressure != 0.0 {
        for i in 0..n {
            bias[i] *= if i==0 { 1.0 + 0.75*e.time_pressure } else { (1.0 - 0.50*e.time_pressure).max(0.20) };
        }
    }
    // planning 需 _plan_match_vector，M1 补
    tail.iter().enumerate().map(|(i,&x)| x * bias[i].clamp(0.05, 3.0)).collect()
}

/// 供 main 用的 policy（返回 eff + weights）
pub fn policy(fen: &str, cands: &[(String,i32)], elo: Option<u16>, target_accuracy: f64, remaining_ms: Option<i64>, speed: &str, state: &HashMap<String,String>) -> (f64, Vec<f64>) {
    let chess = fen_to_chess(fen);
    let mut eff = effective_accuracy(target_accuracy, &chess, remaining_ms, speed);
    if let Some(e) = elo { eff *= elo_base_acc(e) / 0.85; }
    eff = eff.clamp(0.50, 0.99);
    let errors = crate::human::error::profile(&chess, elo, remaining_ms, speed, state);
    let w = weights_for_pool(eff, elo, cands, Some(&errors), state, &chess);
    (eff, w)
}

fn fen_to_chess(fen: &str) -> Chess {
    use std::str::FromStr;
    use shakmaty::fen::Fen;
    Fen::from_str(fen).ok()
        .and_then(|f| f.into_position(shakmaty::CastlingMode::Standard).ok())
        .unwrap_or_default()
}
