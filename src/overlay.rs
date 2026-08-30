//! overlay.rs — eframe 双行置顶浮窗 (M2)
//! 复刻 Python TextOverlay: 420x340, #1a1a2e 背景, 可拖动, POWER/HUMAN 双行

use eframe::egui;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, Default)]
pub struct OverlayState {
    pub power: String,      // e.g. "e2->e4 +0.45"
    pub human: String,      // e.g. "e2->e3 2.3s 82%"
    pub fen: String,
    pub status: String,
}

pub type SharedState = Arc<Mutex<OverlayState>>;

pub fn create_shared() -> SharedState {
    Arc::new(Mutex::new(OverlayState {
        power: "等待...".into(),
        human: "等待...".into(),
        fen: "".into(),
        status: "ChessEye Rust".into(),
    }))
}

pub struct OverlayApp {
    state: SharedState,
}

impl OverlayApp {
    pub fn new(state: SharedState) -> Self { Self { state } }
}

impl eframe::App for OverlayApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 每 100ms 刷新，复刻 Python 的 overlay.refresh()
        ctx.request_repaint_after(std::time::Duration::from_millis(100));

        let st = self.state.lock().unwrap().clone();

        // 样式：深色背景
        let bg = egui::Color32::from_rgb(0x1a, 0x1a, 0x2e);
        egui::CentralPanel::default().frame(egui::Frame::none().fill(bg).inner_margin(8.0)).show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("ChessEye").color(egui::Color32::from_rgb(0xe9, 0x45, 0x60)).size(14.0).strong());
                ui.separator();
                ui.add_space(4.0);

                // POWER 行
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("⚡ POWER").color(egui::Color32::from_rgb(0x16, 0xc7, 0x9a)).size(16.0).strong());
                    ui.label(egui::RichText::new(st.power.clone()).color(egui::Color32::WHITE).size(18.0).strong());
                });
                // HUMAN 行
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("👤 HUMAN").color(egui::Color32::from_rgb(0xff, 0xcc, 0x00)).size(16.0).strong());
                    ui.label(egui::RichText::new(st.human.clone()).color(egui::Color32::from_rgb(0xff, 0xcc, 0x00)).size(18.0).strong());
                });

                ui.add_space(8.0);
                ui.label(egui::RichText::new(format!("FEN: {}", if st.fen.len() > 50 { format!("{}...", &st.fen[..50]) } else { st.fen.clone() })).color(egui::Color32::from_rgb(0x88, 0x88, 0x88)).size(9.0));
                ui.label(egui::RichText::new(st.status.clone()).color(egui::Color32::from_rgb(0x88, 0x88, 0x88)).size(9.0));
            });
        });
    }
}

pub fn run_overlay(state: SharedState) {
    let viewport = egui::ViewportBuilder::default()
        .with_title("ChessEye")
        .with_inner_size([420.0, 340.0])
        .with_position([1920.0 - 460.0, 1080.0 - 380.0]) // 右下角，复刻 Python 的 sw-40, sh-60
        .with_always_on_top()
        .with_transparent(true)
        .with_decorations(false) // 无边框，可拖动靠 egui 的 drag
        .with_resizable(false);

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    let _ = eframe::run_native("ChessEye", options, Box::new(|_cc| Box::new(OverlayApp::new(state))));
}

// 兼容旧 placeholder，供 main  fallback
pub fn run_placeholder(_mode: String, _elo: u16) {
    println!("[overlay] placeholder — 已被 eframe 替代");
}
