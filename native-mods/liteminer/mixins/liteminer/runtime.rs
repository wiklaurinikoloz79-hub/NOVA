// Liteminer: a port of Liteminer (https://github.com/iamkaf/liteminer, MIT
// licence, by iamkaf). Break a block and the connected blocks of the same kind
// go with it, up to a limit, without wearing the tool out. Injected as
// `crate::game::liteminer`; the hook points call `liteminer_destroy`,
// `liteminer_draw` and `liteminer_command`.
//
//   /vein             turns vein mining on or off
//   /vein shape NAME  shapeless (every connected block) or tunnel (a line the way you look)
//   /vein limit N     the most blocks one break takes (1 to 256)
//   /vein deepslate   treats an ore and its deepslate twin as one kind
//
// The state is shared by everyone on the world (a phone plays alone).
use super::Game;
use horizon_client::{
    extras::Corner, extras_hud::Corners, hud::HudBatch, mod_api::NativeBlockRim,
};
use horizon_world::registry::BlockStateId;
use std::{
    sync::{
        Mutex, MutexGuard,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

const LIMIT_MAX: usize = 256;
const ANSWER: Duration = Duration::from_secs(6);
const BOX: u32 = 0x7800_0000;
const ON_COLOR: u32 = 0x0055_FF55;
const ANSWER_COLOR: u32 = 0x00FF_FFAA;
/// The outline of the blocks a break would take (ARGB).
const RIM_COLOR: u32 = 0xFF55_FF55;
/// An aim older than this is stale (the player looks away or the frame skipped it).
const AIM_FRESH: Duration = Duration::from_millis(400);
/// How long a computed vein is reused for the same aim.
const PREVIEW_KEEP: Duration = Duration::from_millis(500);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Shape {
    Shapeless,
    Tunnel,
}

impl Shape {
    fn name(self) -> &'static str {
        match self {
            Shape::Shapeless => "shapeless",
            Shape::Tunnel => "tunnel",
        }
    }
}

struct State {
    on: bool,
    shape: Shape,
    limit: usize,
    deepslate: bool,
    answer: Option<(String, Instant)>,
}

static STATE: Mutex<State> = Mutex::new(State {
    on: false,
    shape: Shape::Shapeless,
    limit: 64,
    deepslate: true,
    answer: None,
});

/// The block aimed at and the vein found for it.
struct Aim {
    pos: [i32; 3],
    seen: Instant,
    vein: Vec<[i32; 3]>,
    made: Instant,
    valid: bool,
}

static AIM: Mutex<Option<Aim>> = Mutex::new(None);

fn aim_state() -> MutexGuard<'static, Option<Aim>> {
    AIM.lock().unwrap_or_else(|e| e.into_inner())
}

/// Set while this mod breaks the neighbours, so its own breaks are not veins.
static BUSY: AtomicBool = AtomicBool::new(false);

fn vein_state() -> MutexGuard<'static, State> {
    STATE.lock().unwrap_or_else(|e| e.into_inner())
}

/// The kind a block belongs to: its name, with an ore's deepslate twin folded
/// into the plain ore when `deepslate` is on.
fn kind(name: &str, deepslate: bool) -> String {
    if deepslate {
        if let Some(rest) = name.strip_prefix("minecraft:deepslate_") {
            if rest.ends_with("_ore") {
                return format!("minecraft:{rest}");
            }
        }
    }
    name.to_string()
}

/// The 26 blocks around a position (the 3x3x3 cube without its centre).
fn around(p: [i32; 3]) -> impl Iterator<Item = [i32; 3]> {
    (-1..=1).flat_map(move |dx| {
        (-1..=1).flat_map(move |dy| {
            (-1..=1).filter_map(move |dz| {
                if dx == 0 && dy == 0 && dz == 0 {
                    None
                } else {
                    Some([p[0] + dx, p[1] + dy, p[2] + dz])
                }
            })
        })
    })
}

/// The axis the camera looks along most, as a unit step (vanilla's yaw: 0
/// looks south (+z), -90 east (+x); pitch down is positive).
fn look_step(yaw: f32, pitch: f32) -> [i32; 3] {
    if pitch > 55.0 {
        return [0, -1, 0];
    }
    if pitch < -55.0 {
        return [0, 1, 0];
    }
    let rad = f64::from(yaw).to_radians();
    let (dx, dz) = (-rad.sin(), rad.cos());
    if dx.abs() > dz.abs() {
        [if dx > 0.0 { 1 } else { -1 }, 0, 0]
    } else {
        [0, 0, if dz > 0.0 { 1 } else { -1 }]
    }
}

impl Game<'_> {
    /// The blocks to take with the one at `origin`, nearest first.
    fn liteminer_vein(
        &self,
        origin: [i32; 3],
        name: &str,
        shape: Shape,
        limit: usize,
        deepslate: bool,
    ) -> Vec<[i32; 3]> {
        let want = kind(name, deepslate);
        let same = |game: &Self, p: [i32; 3]| {
            let id = game.world.block(p[0], p[1], p[2]);
            let facts = game.res.facts().get(id);
            kind(&facts.name, deepslate) == want
        };
        let mut found: Vec<[i32; 3]> = Vec::new();
        match shape {
            Shape::Tunnel => {
                let step = match self.native_camera() {
                    Some(cam) => look_step(cam.yaw, cam.pitch),
                    None => return found,
                };
                let mut p = origin;
                while found.len() < limit {
                    p = [p[0] + step[0], p[1] + step[1], p[2] + step[2]];
                    if !same(self, p) {
                        break;
                    }
                    found.push(p);
                }
            }
            Shape::Shapeless => {
                let mut seen = std::collections::HashSet::new();
                seen.insert(origin);
                let mut queue = std::collections::VecDeque::new();
                queue.push_back(origin);
                while let Some(at) = queue.pop_front() {
                    for next in around(at) {
                        if found.len() >= limit {
                            return found;
                        }
                        if seen.insert(next) && same(self, next) {
                            found.push(next);
                            queue.push_back(next);
                        }
                    }
                }
            }
        }
        found
    }

    /// Whether the held tool can lose one more point of durability without
    /// breaking (a tool that cannot wear, or creative, always can).
    fn liteminer_tool_ok(&self) -> bool {
        if self.creative {
            return true;
        }
        let Some(stack) = self.held() else {
            return true;
        };
        let max = self.items().get(&stack.item).map_or(0, |i| i.max_damage);
        max == 0 || stack.damage + 2 < max
    }

    /// `native_destroy_block`: the player broke `pos`, a block of `state`.
    pub(super) fn liteminer_destroy(&mut self, pos: [i32; 3], state: BlockStateId) {
        if BUSY.load(Ordering::Relaxed) {
            return;
        }
        let (on, shape, limit, deepslate) = {
            let s = vein_state();
            (s.on, s.shape, s.limit, s.deepslate)
        };
        if !on {
            return;
        }
        let name = self.res.facts().get(state).name.to_string();
        if name == "minecraft:air" || name.ends_with("_air") {
            return;
        }
        let vein = self.liteminer_vein(pos, &name, shape, limit, deepslate);
        if vein.is_empty() {
            return;
        }
        BUSY.store(true, Ordering::Relaxed);
        let mut taken = 0usize;
        for at in vein {
            if !self.liteminer_tool_ok() {
                break;
            }
            // Something may have changed it since (a drop, a neighbour update).
            let now = self.world.block(at[0], at[1], at[2]);
            if kind(&self.res.facts().get(now).name, deepslate) != kind(&name, deepslate) {
                continue;
            }
            if self.break_block_as_player(at).is_ok() {
                taken += 1;
            }
        }
        BUSY.store(false, Ordering::Relaxed);
        if taken > 0 {
            let mut s = vein_state();
            s.answer = Some((format!("Vein: +{taken} blocks"), Instant::now() + ANSWER));
        }
    }

    /// `/vein ...` (without the slash): true when it was this mod's command.
    pub(super) fn liteminer_command(&mut self, command: &str) -> bool {
        let mut words = command.split_whitespace();
        if words.next() != Some("vein") {
            return false;
        }
        let sub = words.next().unwrap_or("").to_ascii_lowercase();
        let arg = words.next().unwrap_or("").to_ascii_lowercase();
        let mut s = vein_state();
        let text = match sub.as_str() {
            "" | "toggle" => {
                s.on = !s.on;
                format!("Vein mining {}", if s.on { "on" } else { "off" })
            }
            "on" => {
                s.on = true;
                "Vein mining on".to_string()
            }
            "off" => {
                s.on = false;
                "Vein mining off".to_string()
            }
            "shape" => match arg.as_str() {
                "shapeless" => {
                    s.shape = Shape::Shapeless;
                    "Shape: shapeless".to_string()
                }
                "tunnel" => {
                    s.shape = Shape::Tunnel;
                    "Shape: tunnel".to_string()
                }
                _ => "/vein shape shapeless|tunnel".to_string(),
            },
            "limit" => match arg.parse::<usize>() {
                Ok(n) if (1..=LIMIT_MAX).contains(&n) => {
                    s.limit = n;
                    format!("Limit: {n} blocks")
                }
                _ => format!("/vein limit 1-{LIMIT_MAX}"),
            },
            "deepslate" => {
                s.deepslate = !s.deepslate;
                format!(
                    "Deepslate ores count as the plain ore: {}",
                    if s.deepslate { "yes" } else { "no" }
                )
            }
            _ => "/vein, /vein shape NAME, /vein limit N, /vein deepslate".to_string(),
        };
        s.answer = Some((text, Instant::now() + ANSWER));
        true
    }

    /// The HUD (`native_hud`): a line while vein mining is on, and the last answer.
    pub(super) fn liteminer_draw(&mut self, batch: &mut HudBatch, corners: &mut Corners) {
        if self.native_camera().is_none() || !self.native_corner_shown(Corner::TopLeft) {
            return;
        }
        let mut lines: Vec<(String, u32)> = Vec::new();
        let on;
        {
            let mut s = vein_state();
            if s.answer.as_ref().is_some_and(|(_, t)| Instant::now() >= *t) {
                s.answer = None;
            }
            if let Some((text, _)) = &s.answer {
                lines.push((text.clone(), ANSWER_COLOR));
            }
            if s.on {
                lines.push((
                    format!("Vein mining: {} (max {})", s.shape.name(), s.limit),
                    ON_COLOR,
                ));
            }
            on = s.on;
        }
        if on {
            let n = self.liteminer_preview().len();
            if n > 0 {
                lines.push((format!("Will break: {} blocks", n + 1), ON_COLOR));
            }
        }
        if lines.is_empty() {
            return;
        }
        let width = lines
            .iter()
            .map(|(t, _)| self.native_text_width(t))
            .fold(0.0_f32, f32::max)
            + 6.0;
        let height = lines.len() as f32 * 10.0 + 4.0;
        let (x, y) = corners.place(Corner::TopLeft, width, height);
        self.native_fill(batch, [x, y, width, height], BOX);
        for (i, (text, rgb)) in lines.iter().enumerate() {
            let rgb = if *rgb == ON_COLOR { ON_COLOR } else { *rgb };
            self.native_text(batch, text, [x + 3.0, y + 2.0 + i as f32 * 10.0], rgb, true);
        }
    }

    /// `native_block_highlight`: the block now aimed at.
    pub(super) fn liteminer_aim(&self, pos: [i32; 3]) {
        if !vein_state().on {
            return;
        }
        let mut aim = aim_state();
        match aim.as_mut() {
            Some(a) if a.pos == pos => a.seen = Instant::now(),
            _ => {
                *aim = Some(Aim {
                    pos,
                    seen: Instant::now(),
                    vein: Vec::new(),
                    made: Instant::now().checked_sub(PREVIEW_KEEP).unwrap_or_else(Instant::now),
                    valid: false,
                })
            }
        }
    }

    /// The blocks the aimed block would take, reused for half a second.
    fn liteminer_preview(&self) -> Vec<[i32; 3]> {
        let (on, shape, limit, deepslate) = {
            let s = vein_state();
            (s.on, s.shape, s.limit, s.deepslate)
        };
        let mut aim = aim_state();
        let Some(a) = aim.as_mut() else {
            return Vec::new();
        };
        if !on || a.seen.elapsed() > AIM_FRESH {
            return Vec::new();
        }
        if !a.valid || a.made.elapsed() > PREVIEW_KEEP {
            let id = self.world.block(a.pos[0], a.pos[1], a.pos[2]);
            let name = self.res.facts().get(id).name.to_string();
            a.vein = if name == "minecraft:air" || name.ends_with("_air") {
                Vec::new()
            } else {
                self.liteminer_vein(a.pos, &name, shape, limit, deepslate)
            };
            a.made = Instant::now();
            a.valid = true;
        }
        a.vein.clone()
    }

    /// `native_visuals`: an outline round each block a break would take.
    pub(super) fn liteminer_rims(&self, rims: &mut Vec<NativeBlockRim>) {
        for pos in self.liteminer_preview() {
            rims.push(NativeBlockRim {
                pos,
                argb: RIM_COLOR,
                width: 2.0,
            });
        }
    }

    /// `native_key`: `key.keyboard.v` (a touch button's key) while playing.
    pub(super) fn liteminer_key(&mut self, key: &str, pressed: bool) {
        if pressed && key == "key.keyboard.v" && self.native_playing() {
            self.liteminer_command("vein");
        }
    }
}
