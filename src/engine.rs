//! engine.rs — Stockfish UCI 完整版
//! 支持 multipv 5 真解析，供 dual 共享 raw

use anyhow::{Result, anyhow, Context};
use std::process::Stdio;
use tokio::process::{Command, Child};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[derive(Debug, Clone)]
pub struct RawCandidate {
    pub mv: String,
    pub ev: i32,       // cp, 10000=mate win, -10000=mate loss
    pub mate: Option<i32>,
}

pub struct Engine {
    path: String,
    child: Option<Child>,
}

impl Engine {
    pub fn new(path: &str) -> Self { Self { path: path.to_string(), child: None } }

    pub async fn probe(&mut self) -> Result<()> {
        self.spawn().await?;
        self.cmd("uci").await?;
        self.expect_contains("uciok", 8000).await?;
        self.quit().await;
        Ok(())
    }

    /// 单次取 top-5 raw（depth 18 足够，人味/精确都够）
    pub async fn analyze_raw(&mut self, fen: &str, depth: u8) -> Result<Vec<RawCandidate>> {
        self.spawn().await?;
        self.cmd("uci").await?;
        self.expect_contains("uciok", 8000).await?;
        // 开 multipv 5
        self.cmd("setoption name MultiPV value 5").await?;
        self.cmd("isready").await?;
        self.expect_contains("readyok", 5000).await?;
        self.cmd(&format!("position fen {}", fen)).await?;
        self.cmd(&format!("go depth {}", depth)).await?;
        let lines = self.collect_until_bestmove(12000).await?;
        let mut out = parse_multipv(&lines);
        if out.is_empty() {
            // 回退：单 bestmove
            out = parse_bestmove_fallback(&lines);
        }
        self.quit().await;
        if out.is_empty() { Err(anyhow!("no candidates")) } else { Ok(out) }
    }

    async fn spawn(&mut self) -> Result<()> {
        if self.child.is_some() { return Ok(()); }
        let child = Command::new(&self.path)
            .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null())
            .spawn().with_context(|| format!("spawn {}", self.path))?;
        self.child = Some(child);
        Ok(())
    }
    async fn cmd(&mut self, s: &str) -> Result<()> {
        if let Some(c) = &mut self.child {
            if let Some(stdin) = c.stdin.as_mut() {
                stdin.write_all(format!("{}\n", s).as_bytes()).await?;
                stdin.flush().await?;
            }
        }
        Ok(())
    }
    async fn expect_contains(&mut self, pat: &str, timeout_ms: u64) -> Result<()> {
        let child = self.child.as_mut().ok_or_else(|| anyhow!("no child"))?;
        let stdout = child.stdout.as_mut().ok_or_else(|| anyhow!("no stdout"))?;
        let mut reader = BufReader::new(stdout);
        let deadline = tokio::time::Instant::now() + tokio::time::Duration::from_millis(timeout_ms);
        let mut line = String::new();
        loop {
            if tokio::time::Instant::now() > deadline { return Err(anyhow!("timeout {}", pat)); }
            line.clear();
            let n = tokio::time::timeout(deadline - tokio::time::Instant::now(), reader.read_line(&mut line)).await.map_err(|_| anyhow!("timeout"))??;
            if n == 0 { return Err(anyhow!("eof {}", pat)); }
            if line.contains(pat) { return Ok(()); }
        }
    }
    async fn collect_until_bestmove(&mut self, timeout_ms: u64) -> Result<Vec<String>> {
        let child = self.child.as_mut().ok_or_else(|| anyhow!("no child"))?;
        let stdout = child.stdout.as_mut().ok_or_else(|| anyhow!("no stdout"))?;
        let mut reader = BufReader::new(stdout);
        let deadline = tokio::time::Instant::now() + tokio::time::Duration::from_millis(timeout_ms);
        let mut lines = Vec::new();
        let mut line = String::new();
        loop {
            if tokio::time::Instant::now() > deadline { break; }
            line.clear();
            let n = tokio::time::timeout(deadline - tokio::time::Instant::now(), reader.read_line(&mut line)).await.map_err(|_| anyhow!("timeout"))??;
            if n == 0 { break; }
            let l = line.trim().to_string();
            lines.push(l.clone());
            if l.starts_with("bestmove") { break; }
        }
        Ok(lines)
    }
    async fn quit(&mut self) {
        if let Some(mut c) = self.child.take() {
            let _ = c.stdin.take();
            let _ = tokio::time::timeout(tokio::time::Duration::from_millis(500), c.wait()).await;
            let _ = c.kill().await;
        }
    }
}

fn parse_multipv(lines: &[String]) -> Vec<RawCandidate> {
    // 取每个 multipv 的最后一条 info（depth 最深）
    use std::collections::HashMap;
    let mut map: HashMap<u8, RawCandidate> = HashMap::new();
    for l in lines {
        if !l.starts_with("info") || !l.contains(" multipv ") || !l.contains(" pv ") { continue; }
        // 例: info depth 18 ... multipv 2 score cp 123 pv e2e4 ...
        let parts: Vec<&str> = l.split_whitespace().collect();
        let mut mpv: Option<u8> = None;
        let mut cp: Option<i32> = None;
        let mut mate: Option<i32> = None;
        let mut pv: Option<String> = None;
        let mut i = 0;
        while i < parts.len() {
            match parts[i] {
                "multipv" => { mpv = parts.get(i+1).and_then(|s| s.parse().ok()); i+=2; },
                "score" => {
                    if parts.get(i+1) == Some(&"cp") { cp = parts.get(i+2).and_then(|s| s.parse().ok()); i+=3; }
                    else if parts.get(i+1) == Some(&"mate") { mate = parts.get(i+2).and_then(|s| s.parse().ok()); i+=3; }
                    else { i+=1; }
                },
                "pv" => { pv = parts.get(i+1).map(|s| s.to_string()); i+=2; },
                _ => i+=1,
            }
        }
        if let (Some(n), Some(mv)) = (mpv, pv) {
            let ev = if let Some(m) = mate { if m > 0 { 10000 } else { -10000 } } else { cp.unwrap_or(0) };
            map.insert(n, RawCandidate { mv, ev, mate });
        }
    }
    let mut v: Vec<(u8, RawCandidate)> = map.into_iter().collect();
    v.sort_by_key(|(k,_)| *k);
    v.into_iter().map(|(_,c)| c).collect()
}

fn parse_bestmove_fallback(lines: &[String]) -> Vec<RawCandidate> {
    for l in lines.iter().rev() {
        if l.starts_with("bestmove") {
            if let Some(mv) = l.split_whitespace().nth(1) {
                if !mv.is_empty() && mv != "(none)" {
                    return vec![RawCandidate { mv: mv.to_string(), ev: 0, mate: None }];
                }
            }
        }
    }
    vec![]
}
