// The mod's settings page: the WebAssembly mod's buttons, then the base's.
fn official_minimap_page(game: &mut crate::game::Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    use crate::game::official_minimap as m;
    if mod_id != m::ID {
        return;
    }
    let game = &*game;
    buttons.extend(m::settings(|key| game.native_setting(m::ID, key)).labels());
    let base = game
        .native_setting(m::ID, "base")
        .and_then(|b| m::parse_base(&b));
    buttons.push(match base {
        Some((_, _, p)) => format!("Mark base here (now {} {} {})", p[0] as i64, p[1] as i64, p[2] as i64),
        None => "Mark base here".to_string(),
    });
    buttons.push("Remove base".to_string());
}
