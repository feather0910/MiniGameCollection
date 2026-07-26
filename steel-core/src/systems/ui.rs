use bevy::prelude::*;

use crate::components::{Building, CoreTag};
use crate::config::BuildingKind;
use crate::map::{BuildMode, GameOver, Inventory, WaveState};

#[derive(Resource, Clone)]
#[allow(dead_code)]
pub struct UiFont(pub Handle<Font>);

#[derive(Component)]
pub struct UiCopper;
#[derive(Component)]
pub struct UiLead;
#[derive(Component)]
pub struct UiWave;
#[derive(Component)]
pub struct UiTimer;
#[derive(Component)]
pub struct UiCoreHp;
#[derive(Component)]
pub struct UiHint;
#[derive(Component)]
pub struct UiOverlay;
#[derive(Component)]
pub struct UiOverlayTitle;
#[derive(Component)]
pub struct BuildButton(pub BuildingKind);

pub fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/ui.ttf");
    commands.insert_resource(UiFont(font.clone()));

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(56.0),
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: Val::Px(10.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.09, 0.11, 0.82)),
        ))
        .with_children(|parent| {
            spawn_stat(parent, &font, "铜", Color::srgb(0.88, 0.54, 0.24), UiCopper);
            spawn_stat(parent, &font, "铅", Color::srgb(0.55, 0.48, 0.72), UiLead);
            spawn_stat(parent, &font, "波次", Color::srgb(0.49, 0.88, 1.0), UiWave);
            spawn_stat(parent, &font, "下一波", Color::srgb(0.24, 0.84, 0.78), UiTimer);
            spawn_stat(parent, &font, "核心HP", Color::srgb(0.24, 0.84, 0.55), UiCoreHp);
        });

    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            bottom: Val::Px(48.0),
            left: Val::Px(0.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|parent| {
            for (i, kind) in BuildingKind::selectable().iter().enumerate() {
                let (c, l) = kind.cost();
                let cost = if l > 0 {
                    format!("铜{c} 铅{l}")
                } else {
                    format!("铜{c}")
                };
                parent
                    .spawn((
                        Button,
                        BuildButton(*kind),
                        Node {
                            width: Val::Px(108.0),
                            height: Val::Px(58.0),
                            padding: UiRect::all(Val::Px(8.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::FlexStart,
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(2.0),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.07, 0.13, 0.16)),
                        BorderColor(Color::srgb(0.16, 0.29, 0.32)),
                    ))
                    .with_children(|b| {
                        b.spawn((
                            Text::new(format!("{}. {}", i + 1, kind.name())),
                            TextFont {
                                font: font.clone(),
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.9, 0.95, 0.97)),
                        ));
                        b.spawn((
                            Text::new(cost),
                            TextFont {
                                font: font.clone(),
                                font_size: 11.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.58, 0.64, 0.70)),
                        ));
                    });
            }
        });

    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.0),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            ..default()
        })
        .with_children(|p| {
            p.spawn((
                Text::new("WASD 移动 · 空格/左键射击 · E 手挖 · 1-5 建造 · R 旋转 · 右键拆除"),
                TextFont {
                    font: font.clone(),
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.58, 0.64, 0.70)),
                UiHint,
            ));
        });

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                display: Display::None,
                ..default()
            },
            BackgroundColor(Color::srgba(0.03, 0.06, 0.08, 0.82)),
            UiOverlay,
        ))
        .with_children(|p| {
            p.spawn((
                Text::new(""),
                TextFont {
                    font: font.clone(),
                    font_size: 36.0,
                    ..default()
                },
                TextColor(Color::srgb(0.24, 0.84, 0.78)),
                UiOverlayTitle,
            ));
        });
}

fn spawn_stat(
    parent: &mut ChildBuilder,
    font: &Handle<Font>,
    label: &str,
    color: Color,
    marker: impl Component,
) {
    parent
        .spawn((
            Node {
                min_width: Val::Px(78.0),
                padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.07, 0.13, 0.16)),
            BorderColor(Color::srgb(0.16, 0.29, 0.32)),
        ))
        .with_children(|p| {
            p.spawn((
                Text::new(label),
                TextFont {
                    font: font.clone(),
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgb(0.58, 0.64, 0.70)),
            ));
            p.spawn((
                Text::new("0"),
                TextFont {
                    font: font.clone(),
                    font_size: 18.0,
                    ..default()
                },
                TextColor(color),
                marker,
            ));
        });
}

pub fn update_ui(
    inv: Res<Inventory>,
    wave: Res<WaveState>,
    mut mode: ResMut<BuildMode>,
    over: Res<GameOver>,
    cores: Query<&Building, With<CoreTag>>,
    mut texts: ParamSet<(
        Query<&mut Text, With<UiCopper>>,
        Query<&mut Text, With<UiLead>>,
        Query<&mut Text, With<UiWave>>,
        Query<&mut Text, With<UiTimer>>,
        Query<&mut Text, With<UiCoreHp>>,
        Query<&mut Text, With<UiHint>>,
        Query<&mut Text, With<UiOverlayTitle>>,
    )>,
    mut overlay: Query<&mut Node, With<UiOverlay>>,
    mut all_btns: Query<(&BuildButton, &mut BorderColor)>,
    interactions: Query<(&Interaction, &BuildButton), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, btn) in interactions.iter() {
        if *interaction == Interaction::Pressed {
            mode.selected = Some(btn.0);
        }
    }

    if let Ok(mut t) = texts.p0().get_single_mut() {
        *t = Text::new(inv.copper.to_string());
    }
    if let Ok(mut t) = texts.p1().get_single_mut() {
        *t = Text::new(inv.lead.to_string());
    }
    if let Ok(mut t) = texts.p2().get_single_mut() {
        *t = Text::new(wave.wave.to_string());
    }
    if let Ok(mut t) = texts.p3().get_single_mut() {
        *t = Text::new(if wave.between {
            format!("{}s", wave.timer.ceil().max(0.0) as i32)
        } else {
            "战斗中".into()
        });
    }
    if let Ok(mut t) = texts.p4().get_single_mut() {
        let hp = cores
            .get_single()
            .map(|b| b.hp.max(0.0).ceil() as i32)
            .unwrap_or(0);
        *t = Text::new(hp.to_string());
    }
    if let Ok(mut t) = texts.p5().get_single_mut() {
        *t = Text::new(match mode.selected {
            Some(k) => {
                let (c, l) = k.cost();
                let cost = if l > 0 {
                    format!("铜{c} 铅{l}")
                } else {
                    format!("铜{c}")
                };
                format!(
                    "建造 {}：{} · 花费 {} · R 旋转 · 右键/X 拆除",
                    k.name(),
                    k.description(),
                    cost
                )
            }
            None => "WASD 移动 · 空格/左键射击 · E 手挖 · 1-5 建造 · R 旋转 · 右键拆除".into(),
        });
    }

    for (btn, mut border) in all_btns.iter_mut() {
        border.0 = if mode.selected == Some(btn.0) {
            Color::srgb(0.24, 0.84, 0.78)
        } else {
            Color::srgb(0.16, 0.29, 0.32)
        };
    }

    if let Ok(mut node) = overlay.get_single_mut() {
        node.display = if over.done {
            Display::Flex
        } else {
            Display::None
        };
    }
    if over.done {
        if let Ok(mut t) = texts.p6().get_single_mut() {
            *t = Text::new(if over.won {
                "防线稳固"
            } else {
                "核心被毁"
            });
        }
    }
}
