// native_hooks::Game::native_hud (a hook point): the saved places near the player.
fn waypoints_hud(
    game: &mut crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    corners: &mut horizon_client::extras_hud::Corners,
) {
    game.waypoints_draw(batch, corners);
}
