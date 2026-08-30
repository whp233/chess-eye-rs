//! accuracy.rs — 对应 human_accuracy.py
//! ELO_WEIGHTS (闭区间 lo<elo<=hi) / ELO_BASE_ACC / TACTICAL_TAIL(0.15,0.25,0.30,0.30)
//! _weights_for_pool 保持 P(best)=eff 的 rescale 守卫 |ts-s|>1e-12

use crate::human::error;

// 权威表：Human Mode 用闭区间，chesseye 精确模式另有一份开区间副本，勿统一
pub const ELO_WEIGHTS: &[((u16, u16), [f64; 5])] = &[
    ((0, 800),    [0.25, 0.22, 0.20, 0.18, 0.15]),
    ((800, 1200), [0.35, 0.25, 0.18, 0.12, 0.10]),
    ((1200, 1600),[0.50, 0.25, 0.12, 0.08, 0.05]),
    ((1600, 2000),[0.65, 0.22, 0.10, 0.03, 0.00]),
    ((2000, 2400),[0.85, 0.12, 0.03, 0.00, 0.00]),
    ((2400, 9999),[0.95, 0.05, 0.00, 0.00, 0.00]),
];

pub const TACTICAL_TAIL: [f64; 4] = [0.15, 0.25, 0.30, 0.30];

pub fn elo_weights(elo: u16) -> [f64; 5] {
    for &((lo, hi), w) in ELO_WEIGHTS {
        if lo < elo && elo <= hi { return w; }
    }
    [0.65, 0.22, 0.10, 0.03, 0.00]
}

/// 占位：M1 实现 _effective_accuracy + _weights_for_pool + _shape_tail + policy()
/// 契约：weights[0]==eff, tail 重缩放回 s, 单掷唯一 random 在调用方
pub fn policy_stub() {
    let _ = error::profile_stub();
}
