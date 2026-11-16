// native_hooks::Game::native_commands (a hook point): the command names this mod owns.
fn waypoints_commands(commands: &mut Vec<&'static str>) {
    commands.push("wp");
}
