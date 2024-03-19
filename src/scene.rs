use bevy::prelude::*;

pub struct ScenePlugin;

#[derive(Component)]
struct PlayerCamera;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(CurrentMode::default())
            .add_systems(Startup, (spawn_light, spawn_camera, spawn_ui_text))
            .add_systems(Update, (keyboard_input, update_current_mode_text));
    }
}

#[derive(Component)]
struct ControlsDesc;

#[derive(Resource, Default, Debug)]
pub enum CurrentMode {
    #[default]
    Non,
    CreateShape,
}

fn spawn_light(mut commands: Commands) {
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_xyz(50.0, 50.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
        directional_light: DirectionalLight {
            illuminance: 1_500.,
            ..default()
        },
        ..default()
    });
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: CAMERA_ORIGIN.looking_at(Vec3::Y * 200., Vec3::Y),
            ..default()
        },
        PlayerCamera,
    ));
}

fn spawn_ui_text(mut commands: Commands) {
    let style = TextStyle {
        font_size: 20.0,
        ..default()
    };
    commands.spawn((
        TextBundle::from_sections(vec![
            TextSection::new("Controls\n", style.clone()),
            TextSection::new("---------------\n", style.clone()),
            TextSection::new("Current mode: ", style.clone()),
            TextSection::new("Non", style.clone()),
            TextSection::new("\n", style.clone()),
            TextSection::new("A - Start creating a new shape", style.clone()),
        ])
            .with_style(Style {
                position_type: PositionType::Absolute,
                bottom: Val::Px(12.0),
                left: Val::Px(12.0),
                ..default()
            }),
        ControlsDesc,
    ));
}

fn update_current_mode_text(current_mode: Res<CurrentMode>, mut text: Query<&mut Text, With<ControlsDesc>>) {
    text.single_mut().sections[3].value = format!("{:?}", *current_mode);
}

fn keyboard_input(keys: Res<ButtonInput<KeyCode>>, mut current_mode: ResMut<CurrentMode>) {
    if keys.just_pressed(KeyCode::KeyA) {
        *current_mode = match *current_mode {
            CurrentMode::Non => CurrentMode::CreateShape,
            CurrentMode::CreateShape => CurrentMode::Non,
        };
    }
}
