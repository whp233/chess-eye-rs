# 手动推送指南

本地已就绪 4 个提交（main 分支）：

```
04efb7a M0: skeleton
679b118 M1: human accuracy/error/time
b2f72e8 engine: multipv parse
ee5b0e7 human/mode
```

## 一键推送（需你在 GitHub 网页先建空仓）

1. 在 https://github.com/new 建空仓 `whp233/chess-eye-rs`（不要勾 README）
2. 回到本机执行：

```powershell
cd "C:\Users\whp18\Desktop\Desktop\chess-eye-rs"
# 如果之前已 add origin，先删：
git remote remove origin 2>$null
git remote add origin https://github.com/whp233/chess-eye-rs.git
git push -u origin main
```

3. 去 Actions 页看 CI 是否绿：https://github.com/whp233/chess-eye-rs/actions

## 若 CI 红了

把红的 log 贴给我，我本地改完再 `git commit && git push`

## 本地仅检查（不编译省磁盘）

```powershell
# 需先装 rustup（约 1GB），C 盘仅剩 6.4G，请确保有空间：
# 下载 https://win.rustup.rs/x86_64 → rustup-init.exe → 默认安装
cargo check
```

## Python 原版验证（Rust 不影响它）

```powershell
"C:\Users\whp18\Desktop\Desktop\chess-eye\.venv\Scripts\python.exe" "C:\Users\whp18\Desktop\Desktop\chess-eye\tests\run_all.py"
# 应 61 绿
```
