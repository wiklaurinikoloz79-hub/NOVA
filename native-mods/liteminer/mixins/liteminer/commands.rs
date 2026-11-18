// native_hooks::Game::native_commands (a hook point): so chat completes `/vein`.
fn liteminer_commands(commands: &mut Vec<&'static str>) {
    commands.push("vein");
}
