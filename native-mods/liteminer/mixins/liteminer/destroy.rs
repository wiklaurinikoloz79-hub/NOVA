// native_hooks::Game::native_destroy_block (a hook point): the broken block.
fn liteminer_destroy(game: &mut crate::game::Game<'_>, pos: [i32; 3], state: horizon_world::registry::BlockStateId) {
    game.liteminer_destroy(pos, state);
}
