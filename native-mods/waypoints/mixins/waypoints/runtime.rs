// Waypoints: places the player saves with `/wp add NAME`, kept in the game's
// config folder (`waypoints.tsv`) and shown on the HUD with a direction and a
// distance, so a long trip never loses them. Injected as
// `crate::game::waypoints`; the hook points call `waypoints_draw` and
// `waypoints_command`.
//
//   /wp add NAME      saves the place the player stands at
//   /wp remove NAME   forgets it
//   /wp list          lists this world's places
//
// The places are kept per world folder (a world opened from a server, which
// has none, shares one list). The HUD shows the nearest few of the current
// dimension: `NAME ^> 312m` is a place ahead and to the right, 312 blocks off.
use super::Game;
use horizon_client::{extras::Corner, extras_hud::Corners, hud::HudBatch};
use std::{
    path::PathBuf,
    sync::{Mutex, MutexGuard},
    time::{Duration, Instant},
};

/// The file in the game's config folder.
const FILE: &str = "waypoints.tsv";
/// How many places the HUD shows at most.
const SHOWN: usize = 5;
/// The longest name kept.
const NAME_MAX: usize = 24;
/// How long an answer to a command stays on the HUD.
const ANSWER: Duration = Duration::from_secs(6);
/// How long a list stays (it is longer).
const LIST: Duration = Duration::from_secs(12);
/// What a place's arrow shows: ahead, then clockwise in eighths of a turn.
const ARROWS: [&str; 8] = ["^", "^>", ">", "v>", "v", "<v", "<", "<^"];
/// The HUD's box.
const BOX: u32 = 0x7800_0000;
const NAME_COLOR: u32 = 0x00FF_FFFF;
const ANSWER_COLOR: u32 = 0x00FF_FFAA;

struct Entry {
    world: String,
    dim: String,
    pos: [i32; 3],
    name: String,
}

struct State {
    entries: Vec<Entry>,
    loaded: bool,
    answer: Vec<String>,
    until: Option<Instant>,
}

static STATE: Mutex<State> = Mutex::new(State {
    entries: Vec::new(),
    loaded: false,
    answer: Vec::new(),
    until: None,
});

fn state() -> MutexGuard<'static, State> {
    let mut s = STATE.lock().unwrap_or_else(|e| e.into_inner());
    if !s.loaded {
        s.loaded = true;
        if let Some(text) = file().and_then(|p| std::fs::read_to_string(p).ok()) {
            s.entries = text.lines().filter_map(parse).collect();
        }
    }
    s
}

fn file() -> Option<PathBuf> {
    Game::native_config_dir().ok().map(|d| d.join(FILE))
}

/// `world TAB dimension TAB x TAB y TAB z TAB name`.
fn parse(line: &str) -> Option<Entry> {
    let mut parts = line.splitn(6, '\t');
    let world = parts.next()?.to_string();
    let dim = parts.next()?.to_string();
    let x = parts.next()?.parse().ok()?;
    let y = parts.next()?.parse().ok()?;
    let z = parts.next()?.parse().ok()?;
    let name = parts.next()?.to_string();
    if name.is_empty() {
        return None;
    }
    Some(Entry {
        world,
        dim,
        pos: [x, y, z],
        name,
    })
}

fn save(entries: &[Entry]) -> bool {
    let Some(path) = file() else {
        return false;
    };
    let mut text = String::new();
    for e in entries {
        text.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\n",
            e.world, e.dim, e.pos[0], e.pos[1], e.pos[2], e.name
        ));
    }
    std::fs::write(path, text).is_ok()
}

/// A name without control characters (the file's tabs and lines), trimmed and
/// cut to `NAME_MAX`.
fn clean(name: &str) -> String {
    let kept: String = name.chars().filter(|c| !c.is_control()).collect();
    kept.trim().chars().take(NAME_MAX).collect::<String>().trim().to_string()
}

fn short(dim: &str) -> &str {
    dim.strip_prefix("minecraft:").unwrap_or(dim)
}

/// The arrow for a place at `dx`, `dz` from the camera looking at `yaw`
/// (vanilla's: 0 looks south, -90 east).
fn arrow(dx: f64, dz: f64, yaw: f32) -> &'static str {
    let to = (-dx).atan2(dz).to_degrees();
    let mut rel = (to - f64::from(yaw)) % 360.0;
    if rel > 180.0 {
        rel -= 360.0;
    } else if rel < -180.0 {
        rel += 360.0;
    }
    let sector = ((rel / 45.0).round() as i32).rem_euclid(8);
    ARROWS[sector as usize]
}

impl Game<'_> {
    /// The key of the open world's list.
    fn waypoints_world(&self) -> String {
        self.native_world_dir()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
            .unwrap_or_else(|| "default".to_string())
    }

    /// `/wp ...` (without the slash): true when it was this mod's command.
    pub(super) fn waypoints_command(&mut self, command: &str) -> bool {
        let mut words = command.split_whitespace();
        if words.next() != Some("wp") {
            return false;
        }
        let sub = words.next().unwrap_or("").to_ascii_lowercase();
        let name = clean(&words.collect::<Vec<_>>().join(" "));
        let world = self.waypoints_world();
        let camera = self.native_camera();
        let mut answer = Vec::new();
        let mut keep = ANSWER;
        let mut s = state();
        match sub.as_str() {
            "add" | "set" => match (name.is_empty(), camera) {
                (true, _) => answer.push("Name it: /wp add NAME".to_string()),
                (false, None) => answer.push("No position yet".to_string()),
                (false, Some(cam)) => {
                    let pos = [
                        cam.position[0].floor() as i32,
                        cam.position[1].floor() as i32,
                        cam.position[2].floor() as i32,
                    ];
                    let entry = Entry {
                        world: world.clone(),
                        dim: cam.dimension.clone(),
                        pos,
                        name: name.clone(),
                    };
                    let old = s
                        .entries
                        .iter()
                        .position(|e| e.world == world && e.name.eq_ignore_ascii_case(&name));
                    match old {
                        Some(i) => s.entries[i] = entry,
                        None => s.entries.push(entry),
                    }
                    let kept = save(&s.entries);
                    answer.push(format!(
                        "Saved {} at {} {} {}{}",
                        name,
                        pos[0],
                        pos[1],
                        pos[2],
                        if kept { "" } else { " (not kept: the file failed)" }
                    ));
                }
            },
            "remove" | "rm" | "delete" | "del" => {
                let at = s
                    .entries
                    .iter()
                    .position(|e| e.world == world && e.name.eq_ignore_ascii_case(&name));
                match (name.is_empty(), at) {
                    (true, _) => answer.push("Name it: /wp remove NAME".to_string()),
                    (false, None) => answer.push(format!("No place called {}", name)),
                    (false, Some(i)) => {
                        s.entries.remove(i);
                        let kept = save(&s.entries);
                        answer.push(format!(
                            "Removed {}{}",
                            name,
                            if kept { "" } else { " (not kept: the file failed)" }
                        ));
                    }
                }
            }
            "list" | "ls" => {
                keep = LIST;
                for e in s.entries.iter().filter(|e| e.world == world) {
                    answer.push(format!(
                        "{}: {} {} {} ({})",
                        e.name,
                        e.pos[0],
                        e.pos[1],
                        e.pos[2],
                        short(&e.dim)
                    ));
                }
                if answer.is_empty() {
                    answer.push("No saved places. /wp add NAME saves one".to_string());
                }
            }
            _ => answer.push("/wp add NAME, /wp remove NAME, /wp list".to_string()),
        }
        s.answer = answer;
        s.until = Some(Instant::now() + keep);
        true
    }

    /// The HUD (`native_hud`): the last answer to a command, and the nearest
    /// saved places of this dimension with their direction and distance.
    pub(super) fn waypoints_draw(&mut self, batch: &mut HudBatch, corners: &mut Corners) {
        let Some(cam) = self.native_camera() else {
            return;
        };
        if !self.native_corner_shown(Corner::TopLeft) {
            return;
        }
        let world = self.waypoints_world();
        let mut lines: Vec<(String, u32)> = Vec::new();
        {
            let mut s = state();
            if s.until.is_some_and(|t| Instant::now() >= t) {
                s.until = None;
                s.answer.clear();
            }
            for line in &s.answer {
                lines.push((line.clone(), ANSWER_COLOR));
            }
            let mut near: Vec<(f64, String)> = s
                .entries
                .iter()
                .filter(|e| e.world == world && e.dim == cam.dimension)
                .map(|e| {
                    let dx = f64::from(e.pos[0]) + 0.5 - cam.position[0];
                    let dy = f64::from(e.pos[1]) - cam.position[1];
                    let dz = f64::from(e.pos[2]) + 0.5 - cam.position[2];
                    let dist = (dx * dx + dy * dy + dz * dz).sqrt();
                    (
                        dist,
                        format!(
                            "{} {} {}m",
                            e.name,
                            arrow(dx, dz, cam.yaw),
                            dist.round() as i64
                        ),
                    )
                })
                .collect();
            near.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
            for (_, text) in near.into_iter().take(SHOWN) {
                lines.push((text, NAME_COLOR));
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
            self.native_text(batch, text, [x + 3.0, y + 2.0 + i as f32 * 10.0], *rgb, true);
        }
    }

    /// `native_key`: `key.keyboard.k` (a touch button's key) while playing.
    pub(super) fn waypoints_key(&mut self, key: &str, pressed: bool) {
        if pressed && key == "key.keyboard.k" && self.native_playing() {
            self.waypoints_command("wp list");
        }
    }
}
