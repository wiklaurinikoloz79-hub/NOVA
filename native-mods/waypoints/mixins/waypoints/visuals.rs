// native_hooks::Game::native_visuals (a hook point): the saved places, outlined.
fn waypoints_visuals(game: &crate::game::Game<'_>, rims: &mut Vec<horizon_client::mod_api::NativeBlockRim>) {
    game.waypoints_rims(rims);
}
