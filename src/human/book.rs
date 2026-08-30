//! book.rs — 对应 human_mode._book_move
//! Polyglot book.bin 读取，top3 主流池，accuracy 只调概率不扩池

use std::path::Path;

pub fn book_move(_fen: &str, _accuracy: f64) -> Option<String> {
    let p = Path::new("book.bin");
    if !p.exists() { return None; }
    // M1: 用 shakmaty 打开 polyglot reader，取 entries[:3]
    None
}
