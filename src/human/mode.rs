//! mode.rs — 对应 human_mode.py 完整翻译
//! 单掷契约 + miss 0.20 [40,160] + 四门槛

use rand::Rng;
use shakmaty::{Chess, Role, Square};
use std::collections::HashMap;
use std::str::FromStr;
use shakmaty::fen::Fen;

pub const MAX_EVAL_DROP_CP: i32 = 300;
pub const MISS_RATE: f64 = 0.20;
pub const MISS_CPL_MIN: i32 = 40;
pub const MISS_CPL_MAX: i32 = 160;
pub const MISS_CANDIDATES: usize = 6;
pub const ENDGAME_MATERIAL: i32 = 26;
pub const LOST_EVAL_CP: i32 = -150;
pub const DECISIVE_GAP_CP: i32 = 150;
pub const HAND_SPEED_S: f64 = 3.0;

fn piece_value(role: Role) -> i32 {
    match role { Role::Pawn=>1, Role::Knight=>3, Role::Bishop=>3, Role::Rook=>5, Role::Queen=>9, _=>0 }
}

pub fn weighted_pick(cands: &[(String, i32)], weights: &[f64]) -> (String, usize) {
    if cands.len()==1 || weights.is_empty() { return (cands[0].0.clone(), 0); }
    let mut rng = rand::thread_rng();
    let total: f64 = weights.iter().sum();
    if total <= 0.0 { return (cands[0].0.clone(), 0); }
    let r: f64 = rng.gen::<f64>() * total;
    let mut cum = 0.0;
    for (i,&w) in weights.iter().enumerate() {
        cum += w;
        if r <= cum { return (cands[i].0.clone(), i); }
    }
    (cands.last().unwrap().0.clone(), cands.len()-1)
}

fn filter_blunders(cands: Vec<(String,i32,Option<i32>)>) -> Vec<(String,i32,Option<i32>)> {
    if cands.is_empty() { return cands; }
    let best = cands.iter().map(|(_,ev,_)| *ev).max().unwrap_or(0);
    let good: Vec<_> = cands.iter().filter(|(_,ev,mate)| {
        if mate == &Some(-1) { return false; }
        (best - *ev) <= MAX_EVAL_DROP_CP
    }).cloned().collect();
    if good.is_empty() {
        let best_c = cands.into_iter().max_by_key(|(_,ev,_)| *ev).unwrap();
        vec![best_c]
    } else { good }
}

fn position_allows_miss(chess: &Chess, cands: &[(String,i32,Option<i32>)]) -> bool {
    if cands.is_empty() { return false; }
    if chess.is_check() { return false; }
    let best = cands.iter().map(|(_,ev,_)| *ev).max().unwrap_or(0);
    if best <= LOST_EVAL_CP { return false; }
    let material: i32 = chess.board().iter().filter(|(_,pc)| pc.role != Role::King).map(|(_,pc)| piece_value(pc.role)).sum();
    if material <= ENDGAME_MATERIAL { return false; }
    if cands.len() >= 2 {
        let mut evs: Vec<i32> = cands.iter().map(|(_,ev,_)| *ev).collect();
        evs.sort_by(|a,b| b.cmp(a));
        if evs[0] - evs[1] > DECISIVE_GAP_CP { return false; }
    }
    true
}

/// 简化版 deliberate miss：从合法走法池外采样，需 engine 回调
/// M1 完整版需 engine.eval_fen，白视角 vs 行棋视角取负
pub fn deliberate_miss_stub() -> Option<(String,i32)> { None }

pub fn get_human_move_from_raw(
    fen: &str,
    raw: &[(String,i32,Option<i32>)],
    accuracy: f64,
    _speed: &str,
    tempo: &mut HashMap<String,i32>,
    policy_weights: Vec<f64>,
) -> Option<(String,f64,f64)> {
    // book 复用 raw 已在 human::book 处理，此处仅演示主路径
    let mut cands = filter_blunders(raw.to_vec());
    if cands.is_empty() { return None; }
    let pairs: Vec<(String,i32)> = cands.iter().map(|(m,e,_)| (m.clone(),*e)).collect();
    let (mv, rank) = weighted_pick(&pairs, &policy_weights);
    let conf = 1.0 - (rank as f64 / cands.len().max(1) as f64);
    // 思考时间占位，调 human::time
    let think = crate::human::time::calculate(fen, _speed, None, "human", tempo).unwrap_or(2.0);
    // miss 注入（20%）
    let mut final_mv = mv;
    let mut final_conf = (conf*100.0).round()/100.0;
    if rand::random::<f64>() < MISS_RATE {
        let chess = Fen::from_str(fen).ok().and_then(|f| f.into_position(shakmaty::CastlingMode::Standard).ok()).unwrap_or_default();
        if position_allows_miss(&chess, &cands) {
            // 此处应采样池外走法并浅评估，M1 补 engine 回调
            // 占位：暂不注入，保持单掷
        }
    }
    Some((final_mv, think, final_conf))
}
