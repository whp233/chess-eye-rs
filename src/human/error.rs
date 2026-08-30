//! error.rs — 对应 human_error.py
//! profile() 纯函数零随机，四源 calculation/evaluation/planning/time_pressure

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Profile {
    pub calculation: f64,
    pub evaluation: f64,
    pub planning: f64,
    pub time_pressure: f64,
}

pub fn profile_stub() -> Profile {
    Profile { calculation: 0.0, evaluation: 0.0, planning: 0.0, time_pressure: 0.0 }
}

pub fn profile(_fen: &str, _elo: Option<u16>, _remaining_ms: Option<i64>, _speed: &str, _state: &HashMap<String, String>) -> Profile {
    // M1 按 Python 逐行翻译：_tactical, _weakness, _time_pressure 等，零随机
    profile_stub()
}
