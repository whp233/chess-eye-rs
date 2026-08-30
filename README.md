# chess-eye-rs

Rust 重构版 ChessEye — 对照 `../chess-eye` Python 版（Spec）实现。

## 架构

```
src/main.rs      # 交互 + 主循环（1精确/2人味/3双轨）
src/lichess.rs   # Lichess Board SSE（复刻5坑：chunked/raw_decode/startpos/重放/超时）
src/engine.rs    # Stockfish UCI（analyze_raw 共享，两轨零重复）
src/config.rs    # config.json（兼容 human.enabled）
src/overlay.rs   # 浮窗（M0 占位，M2 eframe 420x340 双行）
src/human/
  accuracy.rs    # ELO_WEIGHTS / TACTICAL_TAIL / P(best)=eff
  mode.rs        # get_human_move / MISS 0.20 [40,160] / 四门槛
  error.rs       # profile 四源零随机
  time.rs        # calculate
  stats.rs       # GameStats
  book.rs        # Polyglot
```

## 约束（来自 Python 版）

- 本机 4GB/14GB 不本地编译，走 GitHub Actions `windows-latest`
- `NO_PROXY` 需含 `lichess.org`（reqwest `no_proxy()`）
- Stockfish 必须 POPCNT（i5-3470 无 AVX2）
- Human 不变量：单掷、P(best)=eff、tail 只动尾部、elo 闭区间、POV 白视角 vs 行棋视角

## 里程碑

- M0 空壳：SSE 连通 + 引擎 probe + 占位浮窗（本周）
- M1 人味移植：逐函数对照 + 61 测试 mirror
- M2 双轨浮窗：egui 双行 + 延迟揭示
- M3 单文件分发

## 本地

```bash
# 仅语法检查（不真编译，省磁盘）
cargo check

# 推送后 CI 编译
git push
```

## 配置

`config.json` 同 Python 版，新增 `mode: "exact"|"human"|"dual"`。
