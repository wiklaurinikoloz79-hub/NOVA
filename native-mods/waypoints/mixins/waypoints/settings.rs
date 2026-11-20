// native_hooks::Game::native_settings (a hook point): the waypoints menu.
fn waypoints_page(game: &mut crate::game::Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    if mod_id == "waypoints" {
        buttons.extend(game.waypoints_settings());
    }
}
