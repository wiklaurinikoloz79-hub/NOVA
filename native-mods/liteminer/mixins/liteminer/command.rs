// native_hooks::Game::native_command (a hook point): `/vein ...` in chat.
fn liteminer_command(game: &mut crate::game::Game<'_>, command: &str, ran: &mut bool) {
    if game.liteminer_command(command) {
        *ran = true;
    }
}
