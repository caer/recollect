use std::collections::BTreeSet;

use image::imageops::FilterType;

use crate::{
    engine::tile::as_macroquad_color,
    game::{audio::SoundTrack, entity::Player},
};

pub mod audio;
pub mod entity;
pub mod fog;
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
    let mut track_4_hi = SoundTrack::new(audio::SAMPLE_3_HI, [
        0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0,
        1,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0
    ], audio::TEMPO_BPM).await;

    // Which track should unmute next?
    let mut next_track = 0;

    // Load tile maps.
    let mut tilemaps = vec![];
    for &map_bytes in map::TILEMAPS {
        let map_image = image::load_from_memory(map_bytes)
            .unwrap()
            .rotate270()
            .resize_exact(map::WIDTH as u32, map::HEIGHT as u32, FilterType::Nearest);
        tilemaps.push(map_image);
    }

    // Configure player sprites and state.
    let mut player = Player::new();
    let mut player_pulses: Vec<fog::Pulse> = vec![];

    // Configure the map.
    let map_wall_texture = crate::engine::tile::TileTexture::from_bytes(map::TILE_WALL);
    let map_floor_texture = crate::engine::tile::TileTexture::from_bytes(map::TILE_FLOOR);
    let mut map = map::GameMap::new(map_wall_texture.clone(), map_floor_texture.clone());

    // Load the first map.
    let spawn_point = map.load_map(&tilemaps[0]);
    player.position = spawn_point;
    let mut next_map_index = 1;

    loop {
        let frame_time = macroquad::prelude::get_frame_time();

        // Monitor which tracks have played.
        let mut track_2_played = false;
        let mut track_4_played = false;

        // Update audio tracks.
        track_1.update(frame_time);
        track_2_played = track_2_played || track_2_lo.update(frame_time);
        track_2_played = track_2_played || track_2_hi.update(frame_time);
        track_3_lo.update(frame_time);
        track_3_hi.update(frame_time);
        track_4_played = track_4_played || track_4_lo.update(frame_time);
        track_4_played = track_4_played || track_4_hi.update(frame_time);

        // Update player position.
        player.translate(frame_time, &mut map.map, &map_wall_texture);

        // Center the map viewport on the player. //
        let player_view_position =
            map.map
                .grid_to_view(player.position.x, player.position.y, map::FOREGROUND_LAYER);
        // Subtract viewport offset from the view position, since the view position
        // includes the viewport offset.
        let player_view_position = player_view_position - map.map.viewport_offset;
        map.map.viewport_offset.x = -player_view_position.x;
        map.map.viewport_offset.y = -player_view_position.y;
        // Shift the viewport offset to be centered on the player.
        let view_size = map.map.calculate_view_size();
        map.map.viewport_offset.x += view_size.x / 2.0;
        map.map.viewport_offset.y += view_size.y / 2.0;
        // Shift the viewport offset to adjust for the player sprite width.
        let tile_size = map.map.calculate_tile_size();
        map.map.viewport_offset.x -= tile_size.x / 2.0;
        map.map.viewport_offset.y -= tile_size.y / 2.0;

        // If the player is on an objective tile, fill all adjacent tiles
        // to clear the objectives.
        if map.map.tile_has_original_color(
            player.position.x as usize,
            player.position.y as usize,
            map::FOREGROUND_LAYER,
            map::ACCENT_1,
        ) {
            // Clear the objective tiles.
            map.objectives_remaining -= map.map.flood_fill_tiles_original_color(
                player.position.x as usize,
                player.position.y as usize,
                map::FOREGROUND_LAYER,
                map::ACCENT_1,
                map::ACCENT_2,
            );

            // Play a new track.
            match next_track {
                0 => track_1.muted = false,
                1 => {
                    track_2_lo.muted = false;
                    track_2_hi.muted = false;
                }
                2 => {
                    track_3_lo.muted = false;
                    track_3_hi.muted = false;
                }
                3 => {
                    track_4_lo.muted = false;
                    track_4_hi.muted = false;
                }
                _ => {}
            }
            next_track += 1;
        }

        // Emit pulses from the player position when tracks play. //
        if track_2_played {
            player_pulses.push(fog::Pulse::new(
                glam::Vec2::new(player.position.x + 0.5, player.position.y + 0.5),
                fog::MEDIUM_MAX_PULSE_RADIUS as f32,
            ));
        }
        if track_4_played {
            player_pulses.push(fog::Pulse::new(
                glam::Vec2::new(player.position.x + 0.5, player.position.y + 0.5),
                fog::LARGE_MAX_PULSE_RADIUS as f32,
            ));
        }

        // Update existing pulses.
        let time = macroquad::prelude::get_time();
        player_pulses.retain_mut(|pulse| pulse.update(time));

        // Collect all pulsed tiles.
        let mut pulsed_tiles = BTreeSet::new();
        for pulse in &player_pulses {
            let affected_tiles = pulse.affected_tiles(&mut map.map, &map_wall_texture);
            pulsed_tiles.extend(affected_tiles.into_iter());
        }

        // Apply fog of war to the entire map. //
        for x in 0..map::WIDTH {
            for y in 0..map::HEIGHT {
                if let Some(tile_state) = map.map.get_tile_state(x, y, map::FOREGROUND_LAYER) {
                    // Skip wall tiles.
                    if tile_state.texture.as_ref() == Some(&map_wall_texture) {
                        continue;
                    }

                    // TODO: Make default vision radius dynamic.
                    const VISION_RADIUS: f32 = 6.0;

                    // Use more severe fog opacity for tiles further from the player.
                    let tile_distance = (((x as isize - player.position.x as isize).pow(2)
                        + (y as isize - player.position.y as isize).pow(2))
                        as f32)
                        .sqrt();

                    // If the tile is currently pulsed, set it to full visibility.
                    if pulsed_tiles.contains(&(x, y)) {
                        let blend_color = tile_state.original_blend_color;
                        tile_state.target_blend_color = blend_color;
                        tile_state.target_height_offset = 0.1;

                    // Distant tiles are almost fully obscured.
                    } else if tile_distance > VISION_RADIUS {
                        let mut new_blend_color = map::DEFAULT;
                        let opacity = 0.1;
                        new_blend_color.alpha = (opacity * 255.) as u8;
                        tile_state.target_blend_color = new_blend_color;
                        tile_state.target_height_offset = 0.0;

                    // Closer tiles retain their original blend color, but with
                    // reduced opacity based on distance.
                    } else {
                        let mut blend_color = tile_state.original_blend_color;
                        let opacity = 1.1 - (tile_distance / VISION_RADIUS);
                        blend_color.alpha = (opacity * 255.) as u8;
                        tile_state.target_blend_color = blend_color;
                        tile_state.target_height_offset = 0.0;
                    };
                }
            }
        }

        // Render the map.
        map.map.update(frame_time);
        map.map.draw_tiles();
        map.map.draw_sprite(
            &player.sprite,
            player.position.x,
            player.position.y,
            0.5,
            map::FOREGROUND_LAYER,
            player.sprite_flipped,
        );

        // Load the next map if all objectives are cleared.
        if map.objectives_remaining == 0 {
            // Stop all tracks.
            track_1.muted = true;
            track_2_lo.muted = true;
            track_2_hi.muted = true;
            track_3_lo.muted = true;
            track_3_hi.muted = true;
            track_4_lo.muted = true;
            track_4_hi.muted = true;
            next_track = 0;

            // Clear all pulses.
            player_pulses.clear();

            // Load the next map.
            let spawn_point = map.load_map(&tilemaps[next_map_index]);
            player.position = spawn_point;
            next_map_index = (next_map_index + 1) % tilemaps.len();
        }

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
        // macroquad::prelude::draw_text(
        //     "[e]: debug info",
        //     10.,
        //     screen_height - 20.,
        //     20.,
        //     macroquad::prelude::GRAY,
        // );

        // Await next frame.
        macroquad::prelude::next_frame().await
    }
}
