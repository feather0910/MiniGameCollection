//! 钢核防线 — Mindustry 风格工厂塔防
//! 技术栈：Rust + Bevy ECS

mod components;
mod config;
mod map;
mod systems;

use bevy::prelude::*;
use systems::SteelCorePlugin;

fn main() {
    // cargo run 时把工作目录切到 crate 根，确保 assets/ 可加载
    if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
        let _ = std::env::set_current_dir(manifest);
    }

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "钢核防线 — Steel Core".into(),
                        resolution: (1280.0_f32, 720.0_f32).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(SteelCorePlugin)
        .run();
}
