// native_hooks::Game::native_key (a hook point): a key, for the touch button.
fn liteminer_key(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    game.liteminer_key(key, pressed);
}
