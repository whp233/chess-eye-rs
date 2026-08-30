# CLAUDE.md — chess-eye-rs

Rust 版 ChessEye，Python 版 `../chess-eye` 是 Spec。

## 约束

- C 盘仅 6.4G 剩余，不本地编大项目，靠 GitHub Actions `windows-latest` 编
- `reqwest` 必须 `no_proxy()`，否则 Lichess 走 127.0.0.1:10809 404
- Stockfish 用 POPCNT（i5-3470 无 AVX2），`stockfish.exe` 放 exe 同目录
- Human 不变量（见 `src/human/*.rs` 注释）：单掷、P(best)=eff、tail 只动尾部、elo 闭区间、POV 白视角
- 改前无需备份（git 已管），但 Python 原版改前仍需 `cp -r chess-eye chess-eye_backup_<ts>`

## 启动

```
cargo run --release
# 交互：mode[1/2/3] → gameId → w/b → elo
```

## 双轨核心

`engine.analyze_raw(fen)` 一次取 top5，两轨共享：
- POWER = raw[0] 纯 Best
- HUMAN = raw → human::accuracy::policy → weighted_pick → miss 20% [40,160]

## 验证

- `cargo check` 本地
- `cargo test` 镜像 Python 61 用例（M1 补）
- CI 绿后下载 `chess-eye-rs.exe` 单文件即用
