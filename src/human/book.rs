//! book.rs — 对应 human_mode._book_move
//! Polyglot book.bin 读取，top3 主流池，accuracy 只调概率不扩池

use std::path::Path;

fn find_book() -> Option<std::path::PathBuf> {
    let candidates = [
        "book.bin",
        "./book.bin",
        "C:\\Users\\whp18\\Desktop\\Desktop\\chess-eye\\book.bin",
        "C:\\Users\\whp18\\Desktop\\Desktop\\chess-eye-rs\\book.bin",
    ];
    for c in candidates {
        let p = Path::new(c);
        if p.exists() { return Some(p.to_path_buf()); }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(d) = exe.parent() {
            let p = d.join("book.bin");
            if p.exists() { return Some(p); }
        }
    }
    None
}

pub fn book_move(_fen: &str, _accuracy: f64) -> Option<String> {
    let p = find_book()?;
    if !p.exists() { return None; }
    // M1: 用 shakmaty 打开 polyglot reader，取 entries[:3]
    None
}
