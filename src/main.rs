use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::mesh::{MeshVertexBufferLayout, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{
    AsBindGroup, PolygonMode, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::leap_input::{HandPinch, LeapInputPlugin};

mod leap_input;

pub const CAMERA_ORIGIN: Transform = Transform::from_xyz(0., 400., 400.);

#[derive(Component)]
struct PlayerCamera;

#[derive(Component)]
struct ControlsDesc;

#[derive(Resource, Default, Debug)]
pub enum CurrentMode {
    #[default]
    Non,
    CreateShape,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            LeapInputPlugin,
            MaterialPlugin::<LineMaterial>::default(),
        ))
        .add_plugins(WorldInspectorPlugin::new())
        .insert_resource(ClearColor(Color::SEA_GREEN))
        .insert_resource(CurrentMode::default())
        .add_systems(Startup, (spawn_light, spawn_camera, spawn_ui_text))
        .add_systems(Update, (spawn_on_pinch, keyboard_input, update_current_mode_text))
        .run();
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

#[derive(Component, Eq, PartialEq, Ord, PartialOrd)]
struct NewShapePoint(usize);

fn spawn_on_pinch(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut line_materials: ResMut<Assets<LineMaterial>>,
    mut right_pinch_events: EventReader<HandPinch>,
    new_shape_points: Query<(&Transform, &NewShapePoint)>,
) {
    let number_of_points = new_shape_points.iter().len();

    if let Some(event) = right_pinch_events.read().next() {
        let distance = (event.transform.translation.z - CAMERA_ORIGIN.translation.z).abs();
        let normalized_distance = distance.min(600.0) / 600.0;

        let red = normalized_distance;
        let green = 1.0 - normalized_distance;
        let blue = normalized_distance;

        let debug_material = materials.add(StandardMaterial {
            base_color: Color::rgb(red, green, blue),
            metallic: 0.1,
            perceptual_roughness: 0.1,
            ..default()
        });

        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Sphere::default().mesh().uv(32, 18).scaled_by(Vec3::splat(15f32))),
                visibility: Visibility::Visible,
                material: debug_material,
                transform: event.transform,
                ..default()
            },
            NewShapePoint(number_of_points),
        ));
        if number_of_points > 1 {
            let (largest, second_largest) = find_two_largest(new_shape_points.iter(), |&(_, p)| p);
            commands.spawn((MaterialMeshBundle {
                mesh: meshes.add(LineList {
                    lines: vec![(largest.0.translation, second_largest.0.translation)],
                }),
                material: line_materials.add(LineMaterial { color: Color::GREEN }),
                ..default()
            },));
        }
    }
}

#[derive(Asset, TypePath, Default, AsBindGroup, Debug, Clone)]
struct LineMaterial {
    #[uniform(0)]
    color: Color,
}

impl Material for LineMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/line_material.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.polygon_mode = PolygonMode::Line;
        Ok(())
    }
}

/// A list of lines with a start and end position
#[derive(Debug, Clone)]
struct LineList {
    lines: Vec<(Vec3, Vec3)>,
}

impl From<LineList> for Mesh {
    fn from(line: LineList) -> Self {
        let vertices = line.lines.into_iter().flat_map(|(a, b)| [a, b]).collect::<Vec<_>>();

        Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::RENDER_WORLD)
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
    }
}

fn find_two_largest<T, F, K: Ord + Copy>(mut iter: impl Iterator<Item = T>, mut selector: F) -> (T, T)
where
    F: FnMut(&T) -> K,
{
    let mut largest = iter.next().unwrap();
    let mut second_largest = iter.next().unwrap();

    // Initialize based on the selector's criteria
    if selector(&second_largest) > selector(&largest) {
        std::mem::swap(&mut largest, &mut second_largest);
    }

    for item in iter {
        let key = selector(&item);
        if key > selector(&largest) {
            second_largest = largest;
            largest = item;
        } else if key > selector(&second_largest) {
            second_largest = item;
        }
    }

    (largest, second_largest)
}
