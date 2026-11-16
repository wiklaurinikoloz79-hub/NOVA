// native_hooks::Game::native_command (a hook point): `/wp ...` in chat.
fn waypoints_command(game: &mut crate::game::Game<'_>, command: &str, ran: &mut bool) {
    if game.waypoints_command(command) {
        *ran = true;
    }
}
