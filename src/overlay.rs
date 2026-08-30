//! overlay.rs — 浮窗占位
//! M0: 控制台打印双轨占位；M2 换 eframe 置顶透明可拖

pub fn run_placeholder(mode: String, elo: u16) {
    println!("[overlay] placeholder — mode={} elo={}", mode, elo);
    println!("[overlay] M0 控制台双轨预览：");
    println!("  ⚡ POWER  e2->e4  +0.45  [2200]");
    println!("  👤 HUMAN  e2->e3  +0.12  conf 82%  think 2.1s");
    println!("[overlay] M2 将替换为 eframe 置顶浮窗 420x340，可拖动，POWER 秒出、HUMAN 延迟");
}

// M2 时取消注释：
// pub fn run_eframe(...) { ... }
