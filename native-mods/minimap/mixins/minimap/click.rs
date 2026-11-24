// A click on the mod's settings page: change the setting and keep it. The two
// last buttons mark the base where the player stands, or forget it.
fn official_minimap_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    use crate::game::official_minimap as m;
    if mod_id != m::ID {
        return;
    }
    let count = m::settings(|key| game.native_setting(m::ID, key)).labels().len();
    if button == count {
        if let Some(cam) = game.native_camera() {
            let world = game
                .native_world_dir()
                .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
                .unwrap_or_else(|| "default".to_string());
            let value = format!(
                "{}|{}|{}|{}|{}",
                world,
                cam.dimension,
                cam.position[0].floor() + 0.5,
                (cam.position[1] - f64::from(cam.eye_height)).floor(),
                cam.position[2].floor() + 0.5
            );
            game.native_set_setting(m::ID, "base", &value);
        }
        return;
    }
    if button == count + 1 {
        game.native_set_setting(m::ID, "base", "");
        return;
    }
    let mut settings = m::settings(|key| game.native_setting(m::ID, key));
    settings.click(button);
    m::set_settings(settings);
    for (key, value) in settings.stored() {
        game.native_set_setting(m::ID, key, &value);
    }
}
