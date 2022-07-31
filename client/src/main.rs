use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin, EguiSettings};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_startup_system(ui_start)
        .add_startup_system(setup)
        .add_system(ui_scale_factor)
        .add_system(ui)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn_bundle(SceneBundle {
        scene: asset_server.load("tiles.gltf#Scene0"),
        ..default()
    });

    // plane
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });
    // cube
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
    // light
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn ui_start(mut egui_ctx: ResMut<EguiContext>) {
    egui_ctx.ctx_mut().set_visuals(egui::Visuals {
        window_rounding: 0.0.into(),
        ..default()
    });
}
fn ui_scale_factor(mut egui_settings: ResMut<EguiSettings>, windows: Res<Windows>) {
    if let Some(window) = windows.get_primary() {
        egui_settings.scale_factor = 1.0 / window.scale_factor();
    }
}
fn ui(mut egui_context: ResMut<EguiContext>, mut msaa: ResMut<Msaa>) {
    egui::Window::new("Settings").resizable(false).show(egui_context.ctx_mut(), move |ui| {
        let mut msaa_enabled = msaa.samples > 1;
        if ui.checkbox(&mut msaa_enabled, "Msaa").clicked() {
            msaa.samples = if msaa_enabled {
                4
            } else {
                1
            };
        }
    });
}
