@echo off
chcp 65001 >nul
echo === ChessEye Rust v0.1 — 双轨版 ===
echo.

if not exist "chess-eye-rs.exe" (
  echo [错误] 找不到 chess-eye-rs.exe，请从 Releases 下最新 zip 解压
  pause
  exit /b
)
if not exist "stockfish.exe" (
  echo [错误] 找不到 stockfish.exe，请把 chess-eye 里的 stockfish.exe 拷到本目录
  pause
  exit /b
)
if not exist "book.bin" (
  echo [警告] 找不到 book.bin，人味开局将无开局库
)
if not exist "config.json" (
  echo [提示] 未找到 config.json，已从 config.json.example 复制模板
  copy /Y "config.json.example" "config.json" >nul
  echo 请用记事本打开 config.json 填入 lichess_token 后重跑
  pause
  exit /b
)

.\chess-eye-rs.exe
pause
