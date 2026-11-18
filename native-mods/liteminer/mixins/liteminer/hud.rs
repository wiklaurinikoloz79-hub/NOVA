// native_hooks::Game::native_hud (a hook point): the vein mining line.
fn liteminer_hud(game: &mut crate::game::Game<'_>, batch: &mut horizon_client::hud::HudBatch, corners: &mut horizon_client::extras_hud::Corners) {
    game.liteminer_draw(batch, corners);
}
