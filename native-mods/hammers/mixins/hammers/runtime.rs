// Hammers: the Excavator pickaxe, shovel and axe (items of hammers.hmod) break
// a 3x3 wall: the block you break and the eight round it, across the way you
// look (up or down when you look steeply), but only blocks their kind of tool
// is made for. Injected as `crate::game::hammers`.
use super::Game;
use horizon_world::registry::BlockStateId;
use std::sync::atomic::{AtomicBool, Ordering};

static BUSY: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tool {
    Pickaxe,
    Shovel,
    Axe,
}

fn tool_of(item: &str) -> Option<Tool> {
    match item {
        "hammers:excavator_pickaxe" => Some(Tool::Pickaxe),
        "hammers:excavator_shovel" => Some(Tool::Shovel),
        "hammers:excavator_axe" => Some(Tool::Axe),
        _ => None,
    }
}

/// Blocks that must never go with a neighbour.
fn protected(name: &str) -> bool {
    const NEVER: [&str; 12] = [
        "bedrock", "barrier", "command_block", "structure_block", "jigsaw", "portal",
        "reinforced_deepslate", "spawner", "end_gateway", "chest", "shulker",
    ];
    name.ends_with("_air") || name == "minecraft:air" || name == "minecraft:light" || NEVER.iter().any(|n| name.contains(n))
}

/// Is `name` a block this kind of tool is made for?
fn fits(tool: Tool, name: &str) -> bool {
    let n = name.strip_prefix("minecraft:").unwrap_or(name);
    match tool {
        Tool::Pickaxe => [
            "stone", "ore", "deepslate", "cobble", "brick", "netherrack", "basalt", "blackstone",
            "obsidian", "terracotta", "concrete", "andesite", "diorite", "granite", "tuff",
            "calcite", "end_stone", "sandstone", "prismarine", "purpur", "quartz", "dripstone",
            "ice", "magma", "amethyst",
        ]
        .iter()
        .any(|k| n.contains(k)) && !n.contains("powder"),
        Tool::Shovel => [
            "dirt", "grass_block", "sand", "gravel", "clay", "soul_soil", "soul_sand", "snow",
            "mud", "podzol", "mycelium", "concrete_powder", "farmland", "path", "rooted",
        ]
        .iter()
        .any(|k| n.contains(k)) && !n.contains("sandstone") || n.contains("concrete_powder"),
        Tool::Axe => ["_log", "_wood", "_stem", "_hyphae", "planks", "bookshelf", "crafting_table", "pumpkin", "melon"]
            .iter()
            .any(|k| n.contains(k)),
    }
}

/// The axes of the wall: the one the player looks along is left out.
fn wall(yaw: f32, pitch: f32) -> [[i32; 3]; 2] {
    if pitch > 55.0 || pitch < -55.0 {
        return [[1, 0, 0], [0, 0, 1]];
    }
    let rad = f64::from(yaw).to_radians();
    let (dx, dz) = (-rad.sin(), rad.cos());
    if dx.abs() > dz.abs() {
        [[0, 1, 0], [0, 0, 1]]
    } else {
        [[1, 0, 0], [0, 1, 0]]
    }
}

impl Game<'_> {
    /// `native_destroy_block`: the player broke `pos`.
    pub(super) fn hammers_destroy(&mut self, pos: [i32; 3], _state: BlockStateId) {
        if BUSY.load(Ordering::Relaxed) {
            return;
        }
        let Some(tool) = self.held().and_then(|s| tool_of(&s.item)) else {
            return;
        };
        let Some(cam) = self.native_camera() else {
            return;
        };
        let [a, b] = wall(cam.yaw, cam.pitch);
        BUSY.store(true, Ordering::Relaxed);
        for i in -1..=1_i32 {
            for j in -1..=1_i32 {
                if i == 0 && j == 0 {
                    continue;
                }
                if !self.creative {
                    let worn_out = self.held().is_none_or(|s| {
                        let max = self.items().get(&s.item).map_or(0, |d| d.max_damage);
                        max != 0 && s.damage + 2 >= max
                    });
                    if worn_out {
                        BUSY.store(false, Ordering::Relaxed);
                        return;
                    }
                }
                let at = [
                    pos[0] + a[0] * i + b[0] * j,
                    pos[1] + a[1] * i + b[1] * j,
                    pos[2] + a[2] * i + b[2] * j,
                ];
                let now = self.world.block(at[0], at[1], at[2]);
                let name = self.res.facts().get(now).name.to_string();
                if protected(&name) || !fits(tool, &name) {
                    continue;
                }
                let _ = self.break_block_as_player(at);
            }
        }
        BUSY.store(false, Ordering::Relaxed);
    }
}
