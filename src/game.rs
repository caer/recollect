use image::imageops::FilterType;

use crate::{
    engine::tile::as_macroquad_color,
    game::{audio::SoundTrack, entity::Player},
};

pub mod audio;
pub mod entity;
pub mod map;

/// Main game loop entrypoint.
pub async fn game_loop() {
    // Clear the screen to the map background color.
    macroquad::prelude::clear_background(as_macroquad_color(map::BACKGROUND));
    macroquad::prelude::next_frame().await;

    // Start the background texture.
    let bg_track = macroquad::audio::load_sound_from_bytes(audio::FOLEY_VINYL_TEXTURE)
        .await
        .unwrap();
    macroquad::audio::play_sound(
        &bg_track,
        macroquad::audio::PlaySoundParams {
            looped: true,
            volume: 1.0,
        },
    );

    // Drum samples.
    #[rustfmt::skip]
    let mut track_1 = SoundTrack::new(audio::SAMPLE_BASELINE, [
        1,0,0,0, 1,0,0,1, 1,0,0,0, 1,0,0,0, 
        1,0,0,0, 1,0,0,1, 1,0,0,0, 1,0,0,0
    ], audio::TEMPO_BPM).await;

    #[rustfmt::skip]
    let mut track_2_lo = SoundTrack::new(audio::SAMPLE_1_LO, [
        0,0,0,0, 1,0,0,0, 0,0,0,0, 0,0,0,0, 
        0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0
    ], audio::TEMPO_BPM).await;
    #[rustfmt::skip]
    let mut track_2_hi = SoundTrack::new(audio::SAMPLE_1_HI, [
        0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0, 
        0,0,0,0, 1,0,0,0, 0,0,0,0, 0,0,0,0
    ], audio::TEMPO_BPM).await;

    #[rustfmt::skip]
    let mut track_3_lo = SoundTrack::new(audio::SAMPLE_2_LO, [
        0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0, 
        0,0,0,0, 0,0,0,0, 0,0,1,0, 0,0,0,0
    ], audio::TEMPO_BPM).await;
    #[rustfmt::skip]
    let mut track_3_hi = SoundTrack::new(audio::SAMPLE_2_HI, [
        0,0,0,0, 0,0,0,0, 0,0,1,0, 0,0,0,0, 
        0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0
    ], audio::TEMPO_BPM).await;

    // Pulse samples.
    #[rustfmt::skip]
    let mut track_4_lo = SoundTrack::new(audio::SAMPLE_3_LO, [
        1,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0
    ], audio::TEMPO_BPM).await;
    #[rustfmt::skip]
    let mut track_5_hi = SoundTrack::new(audio::SAMPLE_3_HI, [
        0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0,
        1,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0
    ], audio::TEMPO_BPM).await;

    // Load tile maps.
    let mut tileamps = vec![];
    for &map_bytes in map::TILEMAPS {
        let map_image = image::load_from_memory(map_bytes)
            .unwrap()
            .rotate270()
            .resize_exact(map::WIDTH as u32, map::HEIGHT as u32, FilterType::Nearest);
        tileamps.push(map_image);
    }

    // Load the first map.
    let map_wall_texture = crate::engine::tile::TileTexture::from_bytes(map::TILE_WALL);
    let map_floor_texture = crate::engine::tile::TileTexture::from_bytes(map::TILE_FLOOR);
    let mut map =
        crate::engine::tile::TileMap::new(map::WIDTH, map::HEIGHT, map::BACKGROUND, map::DEFAULT);

    // Configure default map settings.
    map.draw_debug_info = true;
    map.viewport_scale = 5.0;

    // Configure player sprites and state.
    let mut player = Player::new();

    loop {
        let frame_time = macroquad::prelude::get_frame_time();

        // Update audio tracks.
        track_1.update(frame_time);
        track_2_lo.update(frame_time);
        track_2_hi.update(frame_time);
        track_3_lo.update(frame_time);
        track_3_hi.update(frame_time);
        track_4_lo.update(frame_time);
        track_5_hi.update(frame_time);

        // Update player position.
        player.translate(frame_time, &mut map, &map_wall_texture);

        // Refresh map state.
        // TODO: This is an expensive hack to clear out state customizations.
        map.load_from_bitmap(
            &tileamps[0],
            map::FOREGROUND_LAYER,
            map::LayeredColorMapper {
                wall_texture: map_wall_texture.clone(),
                floor_texture: map_floor_texture.clone(),
                floor_opacity: 0.75,
            },
        )
        .unwrap();

        // Center the map viewport on the player. //
        let player_view_position =
            map.grid_to_view(player.position.x, player.position.y, map::FOREGROUND_LAYER);
        // Subtract viewport offset from the view position, since the view position
        // includes the viewport offset.
        let player_view_position = player_view_position - map.viewport_offset;
        map.viewport_offset.x = -player_view_position.x;
        map.viewport_offset.y = -player_view_position.y;
        // Shift the viewport offset to be centered on the player.
        let view_size = map.calculate_view_size();
        map.viewport_offset.x += view_size.x / 2.0;
        map.viewport_offset.y += view_size.y / 2.0;
        // Shift the viewport offset to adjust for the player sprite width.
        let tile_size = map.calculate_tile_size();
        map.viewport_offset.x -= tile_size.x / 2.0;
        map.viewport_offset.y -= tile_size.y / 2.0;

        // Render the map.
        map.draw_tiles();
        map.draw_sprite(
            &player.sprite,
            player.position.x,
            player.position.y,
            0.5,
            map::FOREGROUND_LAYER,
            player.sprite_flipped,
        );

        // Draw controls
        let screen_height = macroquad::prelude::screen_height();
        macroquad::prelude::draw_text(
            "[mouse | touch]",
            10.,
            screen_height - 60.,
            20.,
            macroquad::prelude::GRAY,
        );
        macroquad::prelude::draw_text(
            "[w a s d]: move",
            10.,
            screen_height - 40.,
            20.,
            macroquad::prelude::GRAY,
        );
        macroquad::prelude::draw_text(
            "[e]: debug info",
            10.,
            screen_height - 20.,
            20.,
            macroquad::prelude::GRAY,
        );

        // Await next frame.
        macroquad::prelude::next_frame().await
    }
}
