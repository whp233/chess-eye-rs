//! config.rs — 镜像 Python config.json
//! { lichess_token, human:{accuracy, time_mode, accuracy_mode}, mode, elo }
//! 向下兼容：human.enabled → mode 推断

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub lichess_token: Option<String>,
    #[serde(default)]
    pub human: HumanCfg,
    /// "exact" | "human" | "dual" — 双轨核心
    #[serde(default = "default_mode")]
    pub mode: String,
    #[serde(default = "default_elo")]
    pub elo: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanCfg {
    #[serde(default = "default_accuracy")]
    pub accuracy: f64,
    #[serde(default = "default_time_mode")]
    pub time_mode: String,
    #[serde(default = "default_accuracy_mode")]
    pub accuracy_mode: String,
    /// 旧字段兼容
    #[serde(default)]
    pub enabled: Option<bool>,
}

impl Default for HumanCfg {
    fn default() -> Self {
        Self {
            accuracy: 0.85,
            time_mode: "instant".into(),
            accuracy_mode: "static".into(),
            enabled: None,
        }
    }
}

fn default_mode() -> String { "exact".into() }
fn default_elo() -> u16 { 1600 }
fn default_accuracy() -> f64 { 0.85 }
fn default_time_mode() -> String { "instant".into() }
fn default_accuracy_mode() -> String { "static".into() }

impl Default for Config {
    fn default() -> Self {
        Self {
            lichess_token: None,
            human: HumanCfg::default(),
            mode: "exact".into(),
            elo: 1600,
        }
    }
}

pub fn load_or_default() -> Config {
    let path = std::path::Path::new("config.json");
    if let Ok(s) = std::fs::read_to_string(path) {
        if let Ok(cfg) = serde_json::from_str::<Config>(&s) {
            // 兼容旧 human.enabled → mode
            let mut cfg = cfg;
            if cfg.mode == "exact" {
                if let Some(true) = cfg.human.enabled {
                    cfg.mode = "human".into();
                }
            }
            return cfg;
        }
    }
    // 也尝试从 Python 目录的 config.json 读
    let py_path = r"C:\Users\whp18\Desktop\Desktop\chess-eye\config.json";
    if let Ok(s) = std::fs::read_to_string(py_path) {
        if let Ok(cfg) = serde_json::from_str::<Config>(&s) {
            return cfg;
        }
    }
    Config::default()
}
