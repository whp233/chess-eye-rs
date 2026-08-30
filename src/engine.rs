//! engine.rs — Stockfish UCI 封装
//! 对照 Python ChessEngine.analyze_raw / analyze / eval_fen
//! 直接起 stockfish.exe 子进程走 UCI: uci / position fen <fen> / go depth 18

use anyhow::{Result, anyhow, Context};
use std::process::Stdio;
use tokio::process::{Command, Child};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[derive(Debug, Clone)]
pub struct RawCandidate {
    pub mv: String,
    pub ev: i32,       // cp, 10000 = mate win, -10000 = mate loss
    pub mate: Option<i32>,
}

pub struct Engine {
    path: String,
    child: Option<Child>,
}

impl Engine {
    pub fn new(path: &str) -> Self {
        Self { path: path.to_string(), child: None }
    }

    /// 试探启动，M0 验收用
    pub async fn probe(&mut self) -> Result<()> {
        self.spawn().await?;
        self.cmd("uci").await?;
        // 读到 uciok 即算通
        self.expect_contains("uciok", 3000).await?;
        self.quit().await;
        Ok(())
    }

    /// 单次取 top-5 raw（无 ELO 加权）— 双轨核心
    pub async fn analyze_raw(&mut self, fen: &str, depth: u8) -> Result<Vec<RawCandidate>> {
        self.spawn().await?;
        self.cmd("uci").await?;
        self.expect_contains("uciok", 3000).await?;
        self.cmd("isready").await?;
        self.expect_contains("readyok", 2000).await?;
        self.cmd(&format!("position fen {}", fen)).await?;
        self.cmd(&format!("go depth {}", depth)).await?;
        // 收集 info + bestmove
        let out = self.collect_until_bestmove(8000).await?;
        self.quit().await;
        parse_top_moves(&out)
    }

    async fn spawn(&mut self) -> Result<()> {
        if self.child.is_some() { return Ok(()); }
        let child = Command::new(&self.path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .with_context(|| format!("spawn {}", self.path))?;
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
            if tokio::time::Instant::now() > deadline {
                return Err(anyhow!("timeout waiting for {}", pat));
            }
            line.clear();
            let n = tokio::time::timeout(deadline - tokio::time::Instant::now(), reader.read_line(&mut line)).await
                .map_err(|_| anyhow!("timeout"))??;
            if n == 0 { return Err(anyhow!("eof waiting for {}", pat)); }
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
            if tokio::time::Instant::now() > deadline {
                break;
            }
            line.clear();
            let n = tokio::time::timeout(deadline - tokio::time::Instant::now(), reader.read_line(&mut line)).await
                .map_err(|_| anyhow!("timeout"))??;
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
            // 兜底 kill
            let _ = c.kill().await;
        }
    }
}

fn parse_top_moves(lines: &[String]) -> Result<Vec<RawCandidate>> {
    // 简化：从最后几行 info 里提 multipv，若引擎未开 multipv 则只有 bestmove，回退到单步
    // M1 再精细解析 score cp/mate
    let mut best = None;
    for l in lines.iter().rev() {
        if l.starts_with("bestmove") {
            let mv = l.split_whitespace().nth(1).unwrap_or("").to_string();
            if !mv.is_empty() {
                best = Some(mv);
            }
            break;
        }
    }
    if let Some(mv) = best {
        // 粗略：ev 0，mate None，足够 M0 跑通；M1 再从 info 补 ev
        Ok(vec![RawCandidate { mv, ev: 0, mate: None }])
    } else {
        Err(anyhow!("no bestmove"))
    }
}
