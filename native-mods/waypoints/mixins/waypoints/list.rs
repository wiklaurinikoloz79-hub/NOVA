// native_hooks::Game::native_mods (a hook point): the waypoints' row in Options > Mods.
fn waypoints_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    mods.push(crate::game::native_hooks::NativeMod {
        id: "waypoints".into(),
        title: "Waypoints".into(),
        description: "Saved places with a direction, a distance and an outline in the world.".into(),
        keys: Vec::new(),
    });
}
