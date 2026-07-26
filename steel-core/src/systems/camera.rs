use bevy::prelude::*;

use crate::components::{MainCamera, Player};

pub fn follow_camera(
    player_q: Query<&Transform, (With<Player>, Without<MainCamera>)>,
    mut cam_q: Query<&mut Transform, With<MainCamera>>,
) {
    let Ok(player) = player_q.get_single() else {
        return;
    };
    let Ok(mut cam) = cam_q.get_single_mut() else {
        return;
    };
    let target = player.translation.truncate();
    let cur = cam.translation.truncate();
    let next = cur.lerp(target, 0.12);
    cam.translation.x = next.x;
    cam.translation.y = next.y;
}
