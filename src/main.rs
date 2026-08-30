//! ChessEye Rust — entry
//! M0: Lichess SSE + Stockfish UCI + placeholder overlay
//! Python 版是 Spec，Rust 版对照翻译，Human Layer v3/v4.2 不变量原样保留

mod config;
mod engine;
mod lichess;
mod overlay;
mod human;

use anyhow::Result;

fn find_stockfish() -> String {
    // 按优先级找：当前目录 → exe 所在目录 → 经典 Python 目录 → 绝对路径
    let candidates = [
        "stockfish.exe",
        "stockfish",
        "./stockfish.exe",
        "C:\\Users\\whp18\\Desktop\\Desktop\\chess-eye\\stockfish.exe",
        "C:\\Users\\whp18\\Desktop\\Desktop\\chess-eye-rs\\stockfish.exe",
    ];
    // 再加 exe 所在目录
    let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf()));
    let mut all = candidates.to_vec();
    if let Some(d) = exe_dir {
        all.push(d.join("stockfish.exe").to_string_lossy().to_string().leak());
        all.push(d.join("stockfish").to_string_lossy().to_string().leak());
    }
    for c in all {
        if std::path::Path::new(c).exists() {
            return c.to_string();
        }
    }
    "stockfish.exe".to_string()
}

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

    // 引擎自检（多路径查找，解决 Downloads 解压后找不到引擎的问题）
    let stockfish_path = find_stockfish();
    println!("[engine] 尝试路径: {}", stockfish_path);
    {
        let mut eng = engine::Engine::new(&stockfish_path);
        match eng.probe().await {
            Ok(_) => println!("[engine] Stockfish ready (POPCNT)"),
            Err(e) => eprintln!("[engine] not ready: {} (先把 stockfish.exe 放到 exe 同目录)", e),
        }
    }

    // Lichess 流
    let token_opt = cfg.lichess_token.clone().filter(|t| !t.trim().is_empty() && !t.contains("xxx"));
    let mut board = token_opt.clone().map(|t| lichess::LichessBoard::new(t, game_id.clone()));
    if board.is_none() {
        eprintln!("[lichess] 未配置有效 token — 请在 config.json 填 lichess_token (board:play 权限)，当前仅演示空跑");
    } else {
        println!("[lichess] 已配置 token，准备每秒轮询 {}", game_id);
    }
    let dual = mode == "3" || mode == "dual" || mode == "3.双轨";
    let human_mode = mode == "2" || mode == "human" || dual;
    let exact_mode = mode == "1" || mode == "exact" || dual;
    println!("[mode] dual={} human={} exact={} (3=双轨 2=人味 1=精确)", dual, human_mode, exact_mode);
    println!("[loop] 每秒轮询，Ctrl+C 退出");
    println!();

    let mut tempo: std::collections::HashMap<String,i32> = std::collections::HashMap::new();
    tempo.insert("fast_streak".into(), 0);

    // 浮窗共享状态（eframe 在主线程，轮询在后台 tokio 任务）
    let overlay_state = overlay::create_shared();
    let overlay_for_bg = overlay_state.clone();
    // 后台轮询任务
    let bg_handle = {
        let mut board = board;
        let mut last_fen_bg = String::new();
        let mut tempo_bg = tempo;
        let overlay_bg = overlay_for_bg.clone();
        let mode_bg = mode.clone();
        let color_bg = color.clone();
        tokio::spawn(async move {
            let mut scan: u64 = 0;
            let dual_bg = mode_bg == "3" || mode_bg == "dual" || mode_bg == "3.双轨";
            let human_bg = mode_bg == "2" || mode_bg == "human" || dual_bg;
            let exact_bg = mode_bg == "1" || mode_bg == "exact" || dual_bg;
            loop {
                scan += 1;
                let mut fen_opt: Option<String> = None;
                let mut speed_opt: Option<String> = None;
                if let Some(b) = board.as_mut() {
                    match b.connect_and_fetch().await {
                        Ok(st) => {
                            if st.fen != last_fen_bg {
                                println!("[#{}] {} vs {} | {} to move | ply {} | {}", scan, st.white, st.black, st.side_to_move, st.ply, st.fen);
                                {
                                    let mut s = overlay_bg.lock().unwrap();
                                    s.fen = st.fen.clone();
                                    s.status = format!("{} vs {} | {}", st.white, st.black, st.side_to_move);
                                }
                                last_fen_bg = st.fen.clone();
                                fen_opt = Some(st.fen.clone());
                                speed_opt = Some(st.speed.clone());
                            }
                        }
                        Err(e) => {
                            if scan % 5 == 1 { eprintln!("[#{}] lichess fetch: {}", scan, e); }
                        }
                    }
                } else if last_fen_bg.is_empty() {
                    last_fen_bg = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string();
                    fen_opt = Some(last_fen_bg.clone());
                    speed_opt = Some("rapid".to_string());
                    println!("[#{}] demo FEN (无 token): {}", scan, last_fen_bg);
                    {
                        let mut s = overlay_bg.lock().unwrap();
                        s.fen = last_fen_bg.clone();
                        s.status = "demo (无 token)".into();
                    }
                }
                if let Some(fen) = fen_opt {
                    let is_white = fen.split(' ').nth(1).unwrap_or("w") == "w";
                    let my_is_white = color_bg == "w";
                    if is_white == my_is_white {
                        // 复用 engine（需在后台重新创建，简化：每次新建）
                        let mut eng_bg = engine::Engine::new(&find_stockfish());
                        if let Ok(raw) = eng_bg.analyze_raw(&fen, 16).await {
                            if !raw.is_empty() {
                                let mut s = overlay_bg.lock().unwrap();
                                if exact_bg {
                                    let best = &raw[0];
                                    let txt = format!("{}  ev {} ", best.mv, best.ev);
                                    println!("  [POWER] {}", txt);
                                    s.power = txt;
                                }
                                if human_bg {
                                    let cands: Vec<(String,i32)> = raw.iter().map(|c| (c.mv.clone(), c.ev)).collect();
                                    let state: std::collections::HashMap<String,String> = std::collections::HashMap::new();
                                    let (_eff, weights) = human::accuracy::policy(&fen, &cands, Some(elo), 0.85, None, speed_opt.as_deref().unwrap_or("rapid"), &state);
                                    if let Some((mv, think, conf)) = human::mode::get_human_move_from_raw(&fen, &raw.iter().map(|c| (c.mv.clone(), c.ev, c.mate)).collect::<Vec<_>>(), 0.85, speed_opt.as_deref().unwrap_or("rapid"), &mut tempo_bg, weights) {
                                        let txt = format!("{}  {:.1}s  {:.0}%", mv, think, conf*100.0);
                                        println!("  [HUMAN] {}", txt);
                                        s.human = txt;
                                    }
                                }
                            }
                        }
                    }
                }
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        })
    };

    // 主线程跑 eframe 浮窗（阻塞直到窗口关闭）
    overlay::run_overlay(overlay_state);
    // 窗口关了，停后台
    bg_handle.abort();
    println!("[exit] 浮窗关闭，退出");
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
