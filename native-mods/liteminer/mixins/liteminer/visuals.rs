// native_hooks::Game::native_visuals (a hook point): the outline of the blocks a break would take.
fn liteminer_visuals(game: &crate::game::Game<'_>, rims: &mut Vec<horizon_client::mod_api::NativeBlockRim>) {
    game.liteminer_rims(rims);
}
