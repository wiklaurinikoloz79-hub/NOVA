// The minimap Horizon draws for the mod, unless another mod's came first.
fn minimap_view(
    game: &crate::game::Game<'_>,
    minimap: &mut Option<crate::game::native_hooks::NativeMinimap>,
) {
    use crate::game::official_minimap as m;
    use horizon_client::extras::Corner;
    use horizon_client::extras_hud::minimap_coordinates::COORDINATES;
    COORDINATES.with(|c| *c.borrow_mut() = None);
    *m::BASE_DOT.lock().unwrap_or_else(|e| e.into_inner()) = None;
    horizon_client::extras_hud::minimap_coordinates::OUTLINE.with(|s| s.set((0, false)));
    // Turned off in Options > Mods: nothing until the next build leaves it out.
    if minimap.is_some() || !game.native_on(m::ID) {
        return;
    }
    let settings = m::settings(|key| game.native_setting(m::ID, key));
    COORDINATES.with(|c| {
        *c.borrow_mut() = if settings.show_coordinates {
            game.native_player().map(|p| {
                format!(
                    "{} {} {}",
                    p.x.floor() as i64,
                    p.y.floor() as i64,
                    p.z.floor() as i64
                )
            })
        } else {
            None
        }
    });
    horizon_client::extras_hud::minimap_coordinates::OUTLINE
        .with(|s| s.set((settings.outline, settings.round)));
    let (size, blocks_per_pixel) = settings.view();
    // The marked base, as a dot on the map (the page: Mark base here).
    if let (Some(base), Some(cam)) = (game.native_setting(m::ID, "base"), game.native_camera()) {
        if let Some((_, dim, pos)) = m::parse_base(&base) {
            if dim == cam.dimension {
                *m::BASE_DOT.lock().unwrap_or_else(|e| e.into_inner()) = Some(m::base_dot(
                    cam.position,
                    pos,
                    cam.yaw,
                    settings.rotate,
                    settings.round,
                    blocks_per_pixel,
                    size / 2.0,
                ));
            }
        }
    }
    *minimap = Some(crate::game::native_hooks::NativeMinimap {
        corner: [
            Corner::TopLeft,
            Corner::TopRight,
            Corner::BottomLeft,
            Corner::BottomRight,
        ][settings.corner],
        size,
        blocks_per_pixel,
        rotate: settings.rotate,
        round: settings.round,
    });
}
