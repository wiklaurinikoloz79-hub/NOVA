// native_hooks::Game::native_block_highlight (a hook point): the block aimed at.
fn liteminer_highlight(game: &crate::game::Game<'_>, pos: [i32; 3]) {
    game.liteminer_aim(pos);
}
