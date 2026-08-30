# 本地一键打包 — 解压即用 zip
# 用法：powershell -ExecutionPolicy Bypass -File package.ps1

$ErrorActionPreference = "Stop"
$ver = (Get-Content Cargo.toml | Select-String 'version = "(.*)"').Matches[0].Groups[1].Value
$exe = "target\release\chess-eye-rs.exe"
if (!(Test-Path $exe)) {
  Write-Host "[x] 未找到 $exe，先跑 cargo build --release 或等 CI 编好下载" -ForegroundColor Red
  exit 1
}
$zip = "chess-eye-rs-v$ver-windows.zip"
if (Test-Path $zip) { Remove-Item $zip -Force }

# 待打包清单（stockfish/book 若在本地就一起打进去，否则只打模板）
$files = @("target\release\chess-eye-rs.exe", "config.json.example", "双击运行.bat", "README.md", "CLAUDE.md")
if (Test-Path "stockfish.exe") { $files += "stockfish.exe" } else { Write-Host "[!] stockfish.exe 不在，将仅打包模板，手动拷入" -ForegroundColor Yellow }
if (Test-Path "book.bin") { $files += "book.bin" } else { Write-Host "[!] book.bin 不在，将仅打包模板" -ForegroundColor Yellow }

# 复制到临时 dist
$dist = "dist"
if (Test-Path $dist) { Remove-Item $dist -Recurse -Force }
New-Item -ItemType Directory -Path $dist | Out-Null
foreach ($f in $files) {
  $dst = Join-Path $dist (Split-Path $f -Leaf)
  # exe 改名去掉路径
  if ($f -like "target*") { $dst = Join-Path $dist "chess-eye-rs.exe" }
  Copy-Item $f $dst -Force
  Write-Host "[+] $f -> $dst"
}
# config.json 若不存在，从 example 生成占位
if (!(Test-Path (Join-Path $dist "config.json")) -and (Test-Path "config.json.example")) {
  Copy-Item "config.json.example" (Join-Path $dist "config.json") -Force
}

Compress-Archive -Path "$dist\*" -DestinationPath $zip -Force
Write-Host "[✓] 已生成 $zip" -ForegroundColor Green
Get-Item $zip | Select-Object FullName, Length | Format-List
