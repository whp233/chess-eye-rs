//! ChessEye Rust — entry
//! M0: Lichess SSE + Stockfish UCI + placeholder overlay
//! Python 版是 Spec，Rust 版对照翻译，Human Layer v3/v4.2 不变量原样保留

mod config;
mod engine;
mod lichess;
mod overlay;
mod human;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== ChessEye Rust v0.1.0 (M0) ===");
    println!("Python Spec at: C:\\Users\\whp18\\Desktop\\Desktop\\chess-eye\\");

    let cfg = config::load_or_default();
    println!("config: mode={} elo={} accuracy={}", cfg.mode, cfg.elo, cfg.human.accuracy);

    // 交互：复刻 Python 的三选一（1 精确 / 2 人味 / 3 双轨）
    let mode = prompt_mode(&cfg.mode);
    let game_id = prompt_game_id();
    let color = prompt_color();
    let elo: u16 = prompt_elo(cfg.elo);

    println!("mode={} game={} color={} elo={}", mode, game_id, color, elo);

    // 引擎自检（会尝试启动 stockfish.exe，不存在则提示）
    let mut eng = engine::Engine::new("stockfish.exe");
    match eng.probe().await {
        Ok(_) => println!("[engine] Stockfish ready (POPCNT)"),
        Err(e) => eprintln!("[engine] not ready: {} (先把 stockfish.exe 放到 exe 同目录)", e),
    }

    // Lichess 流（需要 token）
    if let Some(token) = cfg.lichess_token.clone() {
        if !token.is_empty() {
            println!("[lichess] would stream game {} (M0 仅打印 FEN，M1 接真流)", game_id);
            // M0 验收：光连通不阻塞
            // let mut board = lichess::LichessBoard::new(token, game_id);
            // board.connect().await?;
        } else {
            println!("[lichess] no token — set lichess_token in config.json");
        }
    }

    // 浮窗占位（M2 再换 eframe）
    overlay::run_placeholder(mode, elo);

    Ok(())
}

fn prompt_mode(default: &str) -> String {
    use std::io::{self, Write};
    print!("模式 [1]精确 [2]人味 [3]双轨 (默认 {}): ", default);
    let _ = io::stdout().flush();
    let mut s = String::new();
    let _ = io::stdin().read_line(&mut s);
    let t = s.trim();
    if t.is_empty() { default.to_string() } else { t.to_string() }
}

fn prompt_game_id() -> String {
    use std::io::{self, Write};
    print!("Lichess URL/ID (e.g. https://lichess.org/bbpTULX2): ");
    let _ = io::stdout().flush();
    let mut s = String::new();
    let _ = io::stdin().read_line(&mut s);
    let t = s.trim();
    extract_game_id(t).unwrap_or_else(|| t.to_string())
}

fn prompt_color() -> String {
    use std::io::{self, Write};
    print!("执白还是黑? (w/b) [w]: ");
    let _ = io::stdout().flush();
    let mut s = String::new();
    let _ = io::stdin().read_line(&mut s);
    let t = s.trim().to_lowercase();
    if t == "b" { "b".into() } else { "w".into() }
}

fn prompt_elo(default: u16) -> u16 {
    use std::io::{self, Write};
    print!("ELO [{}]: ", default);
    let _ = io::stdout().flush();
    let mut s = String::new();
    let _ = io::stdin().read_line(&mut s);
    s.trim().parse().unwrap_or(default)
}

fn extract_game_id(s: &str) -> Option<String> {
    if s.is_empty() { return None; }
    // 复刻 chesseye.py:extract_game_id — 截 lichess.org/xxxx 最多 8 字符
    if let Some(idx) = s.find("lichess.org/") {
        let rest = &s[idx + "lichess.org/".len()..];
        let id: String = rest.chars().take_while(|c| c.is_alphanumeric()).take(8).collect();
        if id.len() >= 4 { return Some(id); }
    }
    let id: String = s.chars().take_while(|c| c.is_alphanumeric()).take(8).collect();
    if id.len() >= 4 { Some(id) } else { None }
}
