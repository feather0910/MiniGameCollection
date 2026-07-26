mod build;
mod camera;
mod combat;
mod extract;
mod logistics;
mod player;
mod setup;
mod ui;

use bevy::prelude::*;

pub use build::*;
pub use camera::*;
pub use combat::*;
pub use extract::*;
pub use logistics::*;
pub use player::*;
pub use setup::*;
pub use ui::*;

use crate::map::{BuildMode, GameMap, GameOver, HoverTile, Inventory, WaveState};
use crate::config::WaveStats;

pub struct SteelCorePlugin;

impl Plugin for SteelCorePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameMap::new())
            .insert_resource(Inventory {
                copper: 100,
                lead: 30,
            })
            .insert_resource(BuildMode {
                selected: None,
                rot: crate::config::Dir::Right,
                last_place: None,
            })
            .insert_resource(WaveState {
                wave: 0,
                timer: WaveStats::FIRST_DELAY,
                between: true,
                spawn_queue: 0,
                spawn_acc: 0.0,
            })
            .insert_resource(GameOver {
                done: false,
                won: false,
            })
            .insert_resource(HoverTile::default())
            .add_systems(Startup, (setup_world, setup_ui).chain())
            .add_systems(
                Update,
                (
                    update_hover,
                    handle_build_hotkeys,
                    handle_build_click,
                    handle_demolish,
                    player_move,
                    player_mine,
                    player_shoot,
                    tick_extractors,
                    tick_conveyors,
                    tick_waves,
                    tick_enemies,
                    tick_turrets,
                    tick_bullets,
                    follow_camera,
                    update_hp_bars,
                    update_ghost,
                    update_ui,
                    check_game_over,
                )
                    .chain(),
            );
    }
}
