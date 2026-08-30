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
    for p in [
        std::path::Path::new("config.json").to_path_buf(),
        std::env::current_exe().ok().and_then(|e| e.parent().map(|d| d.join("config.json"))).unwrap_or_default(),
        std::path::Path::new(r"C:\Users\whp18\Desktop\Desktop\chess-eye\config.json").to_path_buf(),
        std::path::Path::new(r"C:\Users\whp18\Desktop\Desktop\chess-eye-rs\config.json").to_path_buf(),
    ] {
        if p.as_os_str().is_empty() { continue; }
        if let Ok(s) = std::fs::read_to_string(&p) {
            if let Ok(cfg) = serde_json::from_str::<Config>(&s) {
                let mut cfg = cfg;
                if cfg.mode == "exact" {
                    if let Some(true) = cfg.human.enabled {
                        cfg.mode = "human".into();
                    }
                }
                return cfg;
            }
        }
    }
    Config::default()
}
