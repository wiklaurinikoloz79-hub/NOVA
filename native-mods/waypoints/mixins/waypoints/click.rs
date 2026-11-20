// native_hooks::Game::native_settings_click (a hook point): a click in the menu.
fn waypoints_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    if mod_id == "waypoints" {
        game.waypoints_settings_click(button);
    }
}
