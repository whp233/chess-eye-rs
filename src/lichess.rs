//! lichess.rs — Lichess Board API Stream (SSE, ndjson)
//! 对照 Python lichess_board.py 逐坑翻译：
//! 1. chunked 用 reqwest 自动解码（无需手动 read1）
//! 2. JSON 可能含换行 → 按行 + raw_decode 思路，用 serde_json Value 流式
//! 3. initialFen=="startpos" → 标准开局
//! 4. gameFull/gameState 只有 moves 串 → shakmaty 重放
//! 5. 中文名 3 字节切片 → Rust String 天然 UTF-8 安全，无需 re-encode  trick

use anyhow::{Result, anyhow};
use serde_json::Value;
use shakmaty::{Chess, Position, fen::Fen, uci::UciMove};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct BoardState {
    pub fen: String,
    pub ply: u32,
    pub move_number: u32,
    pub side_to_move: String,
    pub status: String,
    pub white: String,
    pub black: String,
    pub speed: String,
    pub wtime_ms: Option<i64>,
    pub btime_ms: Option<i64>,
}

pub struct LichessBoard {
    token: String,
    game_id: String,
    initial_fen: Option<String>,
    players: (String, String),
    speed: String,
    board: Option<BoardState>,
}

impl LichessBoard {
    pub fn new(token: String, game_id: String) -> Self {
        Self {
            token,
            game_id,
            initial_fen: None,
            players: ("White".into(), "Black".into()),
            speed: "rapid".into(),
            board: None,
        }
    }

    /// M0: 单次拉取 gameFull 的阻塞版（便于验收），M1 再做持续 stream
    pub async fn connect_and_fetch(&mut self) -> Result<BoardState> {
        let url = format!("https://lichess.org/api/board/game/stream/{}", self.game_id);
        let client = reqwest::Client::builder()
            .no_proxy()
            .build()?;
        let resp = client
            .get(&url)
            .bearer_auth(&self.token)
            .header("Accept", "application/x-ndjson")
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(anyhow!("lichess HTTP {}", resp.status()));
        }
        // 非流式：读第一块 ndjson（gameFull）
        let text = resp.text().await?;
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() { continue; }
            if let Ok(v) = serde_json::from_str::<Value>(line) {
                self.handle_event(&v);
                if self.board.is_some() {
                    break;
                }
            }
        }
        self.board.clone().ok_or_else(|| anyhow!("no board state parsed"))
    }

    fn handle_event(&mut self, v: &Value) {
        let t = v.get("type").and_then(|x| x.as_str()).unwrap_or("");
        match t {
            "gameFull" => {
                self.speed = v.get("speed").and_then(|x| x.as_str()).unwrap_or("rapid").to_string();
                self.initial_fen = Some(v.get("initialFen").and_then(|x| x.as_str()).unwrap_or("startpos").to_string());
                self.players.0 = v.get("white").and_then(|w| w.get("name")).and_then(|x| x.as_str()).unwrap_or("White").to_string();
                self.players.1 = v.get("black").and_then(|w| w.get("name")).and_then(|x| x.as_str()).unwrap_or("Black").to_string();
                if let Some(st) = v.get("state") {
                    let moves = st.get("moves").and_then(|x| x.as_str()).unwrap_or("");
                    let status = st.get("status").and_then(|x| x.as_str()).unwrap_or("started").to_string();
                    let wtime = st.get("wtime").and_then(|x| x.as_i64());
                    let btime = st.get("btime").and_then(|x| x.as_i64());
                    self.replay(moves, status, wtime, btime);
                }
            }
            "gameState" => {
                let moves = v.get("moves").and_then(|x| x.as_str()).unwrap_or("");
                let status = v.get("status").and_then(|x| x.as_str()).unwrap_or("started").to_string();
                let wtime = v.get("wtime").and_then(|x| x.as_i64());
                let btime = v.get("btime").and_then(|x| x.as_i64());
                self.replay(moves, status, wtime, btime);
            }
            _ => {}
        }
    }

    fn replay(&mut self, moves_str: &str, status: String, wtime: Option<i64>, btime: Option<i64>) {
        let fen_str = self.initial_fen.clone().unwrap_or_else(|| "startpos".into());
        let mut pos: Chess = if fen_str == "startpos" {
            Chess::default()
        } else {
            Fen::from_str(&fen_str).ok().and_then(|f| f.into_position(shakmaty::CastlingMode::Standard).ok()).unwrap_or_default()
        };
        for mv in moves_str.split_whitespace() {
            if let Ok(uci) = UciMove::from_str(mv) {
                if let Ok(m) = uci.to_move(&pos) {
                    pos.play_unchecked(&m);
                }
            }
        }
        let ply = (pos.fullmoves().get() - 1) * 2 + if pos.turn().is_white() { 0 } else { 1 };
        let fen = Fen::from_position(pos.clone(), shakmaty::EnPassantMode::Legal).to_string();
        let side = if pos.turn().is_white() { "White" } else { "Black" };
        let mn = if ply == 0 { 1 } else { (ply / 2) + 1 };
        self.board = Some(BoardState {
            fen,
            ply,
            move_number: mn,
            side_to_move: side.into(),
            status,
            white: self.players.0.clone(),
            black: self.players.1.clone(),
            speed: self.speed.clone(),
            wtime_ms: wtime,
            btime_ms: btime,
        });
    }

    pub fn current(&self) -> Option<BoardState> {
        self.board.clone()
    }
}
