use std::f32::consts::FRAC_PI_2;
use std::fs;

use bevy::{
    color::palettes::tailwind::*, 
    input::mouse::AccumulatedMouseMotion, 
    prelude::*, 
    render::view::RenderLayers, 
    window::PrimaryWindow,
    render::primitives::Aabb
};
use bevy_simple_text_input::{
    TextInput, TextInputPlugin, TextInputSubmitEvent, TextInputSystem, TextInputTextFont, TextInputTextColor, TextInputInactive
};

const BORDER_COLOR_INACTIVE: Color = Color::srgb(0.25, 0.25, 0.25);
const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const BACKGROUND_COLOR: Color = Color::srgb(0.15, 0.15, 0.15);
const DROPDOWN_WIDTH: f32 = 250.;
const NORMAL_BUTTON_COLOR: Color = Color::srgb(0.15, 0.15, 0.15);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(bevy_obj::ObjPlugin)
        .add_plugins(MeshPickingPlugin)
        .add_plugins(TextInputPlugin)
        .add_systems(
            Startup,
            (
                spawn_view_model,
                spawn_world_model,
                spawn_lights,
                spawn_inputs,
                spawn_dropdown,
                model_loader,
            ),
        )
        .init_resource::<MovementSettings>()
        .init_resource::<LoadedModelList>()
        .add_event::<ModelMoveEvent>()
        .add_systems(Update, (
            rotate_player, 
            change_fov, 
            move_player, 
            focus.before(TextInputSystem), 
            listener.after(TextInputSystem),
            handle_escape_key,
            handle_dropdown,
            update_dropdown_content,
            handle_model_selection,
        ))
        .run();
}

#[derive(Debug, Component)]
struct Player;

#[derive(Debug, Component, Deref, DerefMut)]
struct CameraSensitivity(Vec2);

impl Default for CameraSensitivity {
    fn default() -> Self {
        Self(
            // These factors are just arbitrary mouse sensitivity values.
            // It's often nicer to have a faster horizontal sensitivity than vertical.
            // We use a component for them so that we can make them user-configurable at runtime
            // for accessibility reasons.
            // It also allows you to inspect them in an editor if you `Reflect` the component.
            Vec2::new(0.003, 0.002),
        )
    }
}

#[derive(Event)]
struct ModelMoveEvent(Entity, Drag);


#[derive(Resource)]
pub struct MovementSettings {
    pub sensitivity: f32,
    pub speed: f32,
}

impl Default for MovementSettings {
    fn default() -> Self {
        Self {
            sensitivity: 0.00012,
            speed: 12.,
        }
    }
}

#[derive(Debug, Component)]
struct WorldModelCamera;

/// Used implicitly by all entities without a `RenderLayers` component.
/// Our world model camera and all objects other than the player are on this layer.
/// The light source belongs to both layers.
const DEFAULT_RENDER_LAYER: usize = 0;

/// Used by the view model camera and the player's arm.
/// The light source belongs to both layers.
const VIEW_MODEL_RENDER_LAYER: usize = 1;

#[derive(Component)]
struct DimensionBox;

#[derive(Component)]
struct DropdownButton;

#[derive(Component)]
struct DropdownContent;

#[derive(Component)]
struct DropdownOption;

#[derive(Debug, Resource)]
struct LoadedModelList(Vec<Handle<Scene>>);

#[derive(Component)]
struct ModelOption(Handle<Scene>);

impl Default for LoadedModelList {
    fn default() -> Self {
        Self(Vec::new())
    }
}

fn model_loader(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Read the models directory manually
    if let Ok(entries) = fs::read_dir("assets/models") {
        let model_paths: Vec<String> = entries
            .filter_map(Result::ok)
            .filter(|entry| {
                entry.path()
                    .extension()
                    .map_or(false, |ext| ext == "obj")
            })
            .map(|entry| {
                format!("models/{}", entry.file_name().to_string_lossy())
            })
            .collect();

        // Load each OBJ file individually
        let model_handles: Vec<Handle<Scene>> = model_paths
            .iter()
            .map(|path| asset_server.load(path))
            .collect();

        commands.insert_resource(LoadedModelList(model_handles));
    }
}

fn spawn_view_model(
    mut commands: Commands
) {
    commands
        .spawn((
            Player,
            CameraSensitivity::default(),
            Transform::from_xyz(0.0, 1.0, 0.0),
            Visibility::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                WorldModelCamera,
                Camera3d::default(),
                Projection::from(PerspectiveProjection {
                    fov: 90.0_f32.to_radians(),
                    ..default()
                }),
            ));

            // Spawn view model camera.
            parent.spawn((
                Camera3d::default(),
                Camera {
                    // Bump the order to render on top of the world model.
                    order: 1,
                    ..default()
                },
                Projection::from(PerspectiveProjection {
                    fov: 70.0_f32.to_radians(),
                    ..default()
                }),
                // Only render objects belonging to the view model.
                RenderLayers::layer(VIEW_MODEL_RENDER_LAYER),
            ));
        });
}

fn spawn_world_model(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Floor
    let floor = meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(10.0)));
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load("wood_texture.png")),
        perceptual_roughness: 0.8,
        cull_mode: None,
        ..default()
    });
    commands.spawn((Mesh3d(floor), MeshMaterial3d(material.clone())));

    // Walls
    let wall_height = 5.0;
    let wall_thickness = 0.5;
    let wall_length = 10.0;
    let wall_mesh = meshes.add(Cuboid::new(wall_length, wall_height, wall_thickness));
    let wall_material = materials.add(Color::WHITE); 

    // North wall
    commands.spawn((
        Mesh3d(wall_mesh.clone()),
        MeshMaterial3d(wall_material.clone()),
        Transform::from_xyz(0.0, wall_height / 2.0, -wall_length / 2.0),
        PickingBehavior::IGNORE
    ));

    // South wall
    commands.spawn((
        Mesh3d(wall_mesh.clone()),
        MeshMaterial3d(wall_material.clone()),
        Transform::from_xyz(0.0, wall_height / 2.0, wall_length / 2.0),
        PickingBehavior::IGNORE
    ));

    // East wall (rotated 90 degrees)
    commands.spawn((
        Mesh3d(wall_mesh.clone()),
        MeshMaterial3d(wall_material.clone()),
        Transform::from_xyz(wall_length / 2.0, wall_height / 2.0, 0.0)
            .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
        PickingBehavior::IGNORE
    ));

    // West wall (rotated 90 degrees)
    commands.spawn((
        Mesh3d(wall_mesh),
        MeshMaterial3d(wall_material),
        Transform::from_xyz(-wall_length / 2.0, wall_height / 2.0, 0.0)
            .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
        PickingBehavior::IGNORE
    ));
}

fn spawn_lights(mut commands: Commands) {
    commands.spawn((
        PointLight {
            color: Color::from(ROSE_300),
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(-2.0, 4.0, -0.75),
        // The light source illuminates both the world model and the view model.
        RenderLayers::from_layers(&[DEFAULT_RENDER_LAYER, VIEW_MODEL_RENDER_LAYER]),
    ));
}

fn spawn_inputs(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(10.0),
                top: Val::Px(10.0),
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(2.0)),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Start,
                row_gap: Val::Px(8.0),
                column_gap: Val::Px(8.0),
                ..default()
            },
            DimensionBox,
            PickingBehavior::IGNORE
        ))
        .with_children(|parent| {
            // Width input
            parent.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                ..default()
           
            }).with_children(|parent| {
                // Label
                parent.spawn((Text::new(
                    "Width:"),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
                // Input box
                parent.spawn((
                    Node {
                        height: Val::Px(30.0),
                        width: Val::Px(100.0),
                        border: UiRect::all(Val::Px(3.0)),
                        padding: UiRect::all(Val::Px(7.0)),
                        ..default()
                    },
                    BorderColor(BORDER_COLOR_INACTIVE.into()),
                    BackgroundColor(BACKGROUND_COLOR.into()),
                    TextInput,
                    TextInputTextFont (
                        TextFont {
                        font_size: 10.,
                        ..default()

                        }
                    ),
                    TextInputTextColor (
                        TextColor(TEXT_COLOR.into())
                    ),
                    TextInputInactive(true),
                )); 
            });
            // Height input
            parent.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                ..default()
       
            }).with_children(|parent| {
                // Label
                parent.spawn((Text::new(
                    "Height:"),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
                parent.spawn((
                    Node {
                        height: Val::Px(30.0),
                        width: Val::Px(100.0),
                        border: UiRect::all(Val::Px(3.0)),
                        padding: UiRect::all(Val::Px(7.0)),
                        ..default()
                    },
                    BorderColor(BORDER_COLOR_INACTIVE.into()),
                    BackgroundColor(BACKGROUND_COLOR.into()),
                    TextInput,
                    TextInputTextFont (
                        TextFont {
                            font_size: 10.,
                            ..default()
                        }
                    ),
                    TextInputTextColor (
                        TextColor(TEXT_COLOR.into())
                    ),
                    TextInputInactive(true)
                )); 
            });
    });
}

fn spawn_dropdown(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Px(DROPDOWN_WIDTH),
            height: Val::Auto,
            position_type: PositionType::Absolute,
            left: Val::Px(10.0),
            top: Val::Px(10.0),
            padding: UiRect::all(Val::Px(5.0)),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        PickingBehavior::IGNORE
    )).with_children(|parent| {
        // Dropdown button
        parent.spawn((
            Node {
                padding: UiRect::all(Val::Px(5.0)),
                width: Val::Px(DROPDOWN_WIDTH),
                height: Val::Px(30.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(NORMAL_BUTTON_COLOR.into()),
            Button,
            DropdownButton,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Available models"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });

        // Dropdown content
        parent.spawn((
            Node {
                width: Val::Px(DROPDOWN_WIDTH),
                height: Val::Auto,
                position_type: PositionType::Absolute,
                top: Val::Px(40.0),
                left: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Stretch,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BorderColor(BORDER_COLOR_INACTIVE.into()),
            BackgroundColor(NORMAL_BUTTON_COLOR.into()),
            DropdownContent,
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            // Dropdown options
            for model_name in ["Model 1", "Model 2", "Model 3"].iter() {
                parent.spawn((
                    Button,
                    DropdownOption,
                    Node {
                        height: Val::Px(35.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(5.0)),
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new(*model_name),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
            }
        });
    });
}

fn move_player(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut player: Query<&mut Transform, With<Player>>,
    settings: Res<MovementSettings>,
) {
    if let Ok(_window) = primary_window.get_single() {
        if let Ok(mut transform) = player.get_single_mut() {
            let mut velocity = Vec3::ZERO;
            let local_z = transform.local_z();
            let forward = -Vec3::new(local_z.x, 0., local_z.z);
            let right = Vec3::new(local_z.z, 0., -local_z.x);

            for key in keys.get_pressed() {
                let key = *key;
                if key == KeyCode::KeyW {
                    velocity += forward;
                } else if key == KeyCode::KeyS {
                    velocity -= forward;
                } else if key == KeyCode::KeyA {
                    velocity -= right;
                } else if key == KeyCode::KeyD {
                    velocity += right;
                }
            }

            velocity = velocity.normalize_or_zero();

            transform.translation += velocity * time.delta_secs() * settings.speed
        }
    } else {
        warn!("Primary window not found for `move_player`!");
    }
}

fn rotate_player(
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
    mut player: Query<(&mut Transform, &CameraSensitivity), With<Player>>,
) {
    let Ok((mut transform, camera_sensitivity)) = player.get_single_mut() else {
        return;
    };
    let delta = accumulated_mouse_motion.delta;

    if delta != Vec2::ZERO {
        // Note that we are not multiplying by delta_time here.
        // The reason is that for mouse movement, we already get the full movement that happened since the last frame.
        // This means that if we multiply by delta_time, we will get a smaller rotation than intended by the user.
        // This situation is reversed when reading e.g. analog input from a gamepad however, where the same rules
        // as for keyboard input apply. Such an input should be multiplied by delta_time to get the intended rotation
        // independent of the framerate.
        let delta_yaw = -delta.x * camera_sensitivity.x;
        let delta_pitch = -delta.y * camera_sensitivity.y;

        let (yaw, pitch, roll) = transform.rotation.to_euler(EulerRot::YXZ);
        let yaw = yaw + delta_yaw;

        // If the pitch was ±¹⁄₂ π, the camera would look straight up or down.
        // When the user wants to move the camera back to the horizon, which way should the camera face?
        // The camera has no way of knowing what direction was "forward" before landing in that extreme position,
        // so the direction picked will for all intents and purposes be arbitrary.
        // Another issue is that for mathematical reasons, the yaw will effectively be flipped when the pitch is at the extremes.
        // To not run into these issues, we clamp the pitch to a safe range.
        const PITCH_LIMIT: f32 = FRAC_PI_2 - 0.01;
        let pitch = (pitch + delta_pitch).clamp(-PITCH_LIMIT, PITCH_LIMIT);

        transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
    }
}

fn change_fov(
    input: Res<ButtonInput<KeyCode>>,
    mut world_model_projection: Query<&mut Projection, With<WorldModelCamera>>,
) {
    let Ok(mut projection) = world_model_projection.get_single_mut() else {
        return;
    };
    let Projection::Perspective(ref mut perspective) = projection.as_mut() else {
        unreachable!(
            "The `Projection` component was explicitly built with `Projection::Perspective`"
        );
    };

    if input.pressed(KeyCode::ArrowUp) {
        perspective.fov -= 1.0_f32.to_radians();
        perspective.fov = perspective.fov.max(20.0_f32.to_radians());
    }
    if input.pressed(KeyCode::ArrowDown) {
        perspective.fov += 1.0_f32.to_radians();
        perspective.fov = perspective.fov.min(160.0_f32.to_radians());
    }
}

fn listener(mut events: EventReader<TextInputSubmitEvent>) {
    for event in events.read() {
        info!("{:?} submitted: {}", event.entity, event.value);
    }
}

fn focus(
    query: Query<(Entity, &Interaction), Changed<Interaction>>,
    mut text_input_query: Query<(Entity, &mut TextInputInactive, &mut BorderColor)>,
) {
    for (interaction_entity, interaction) in &query {
        if *interaction == Interaction::Pressed {
            for (entity, mut inactive, _border_color) in &mut text_input_query {
                if entity == interaction_entity {
                    inactive.0 = false;
                } else {
                    inactive.0 = true;
                   
                }
            }
        }
    }
}

fn handle_escape_key(
    input: Res<ButtonInput<KeyCode>>,
    mut text_input_query: Query<&mut TextInputInactive>,
) {
    if input.just_pressed(KeyCode::Escape) {
        for mut inactive in &mut text_input_query {
            inactive.0 = true;
        }
    }
}

fn handle_dropdown(
    interaction_query: Query<
        (&Interaction, &DropdownButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut dropdown_content_query: Query<&mut Visibility, With<DropdownContent>>,
) {
    for (interaction, _) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            if let Ok(mut visibility) = dropdown_content_query.get_single_mut() {
                // Toggle visibility
                *visibility = match *visibility {
                    Visibility::Hidden => Visibility::Visible,
                    _ => Visibility::Hidden,
                };
            }
        }
    }
}

fn update_dropdown_content(
    model_list: Res<LoadedModelList>,
    mut commands: Commands,
    dropdown_content: Query<Entity, With<DropdownContent>>,
    dropdown_options: Query<Entity, With<DropdownOption>>,
) {
    if !model_list.is_changed() {
        return;
    }

    // Remove existing options
    for entity in dropdown_options.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // Add new options based on loaded models
    if let Ok(dropdown_entity) = dropdown_content.get_single() {
        for model_handle in model_list.0.iter() {
            let model_name = match model_handle.path() {
                Some(path) => path.path().file_stem().expect("Path should have a file stem").to_owned().into_string().expect("Failed to convert OsString to String"),
                None => "Unknown Model".to_string(),
            };

            commands.entity(dropdown_entity).with_children(|parent| {
                parent
                    .spawn((
                        Button,
                        DropdownOption,
                        ModelOption(model_handle.clone()),
                        Node {
                            height: Val::Px(35.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            padding: UiRect::all(Val::Px(5.0)),
                            ..default()
                        },
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Text::new(model_name.to_string()),
                            TextFont {
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
            });
        }
    }
}

fn handle_model_selection(
    mut commands: Commands,
    interaction_query: Query<(&Interaction, &ModelOption), (Changed<Interaction>, With<Button>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let hover_matl = materials.add(Color::from(CYAN_300));
    for (interaction, model_option) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            let mut transform = Transform::from_xyz(0.0, 1.5, 0.0);
            
            // Keep only the Y rotation
            let rot_y = transform.rotation.to_euler(EulerRot::XYZ).1;
            transform.rotation = Quat::from_euler(EulerRot::XYZ, 0., rot_y, 0.);
            
            // Apply scaling
            transform.scale = Vec3::splat(0.5);

            commands.spawn((
                SceneRoot(model_option.0.clone()),
                transform
            ))
            .observe(update_model_on::<Pointer<Over>>(hover_matl.clone()))
            .observe(move_model);
        }
    }
}

/// Returns an observer that updates the entity's material to the one specified.
fn update_model_on<E>(
    new_material: Handle<StandardMaterial>,
) -> impl Fn(Trigger<E>, Query<&mut MeshMaterial3d<StandardMaterial>>) {
    // An observer closure that captures `new_material`. We do this to avoid needing to write four
    // versions of this observer, each triggered by a different event and with a different hardcoded
    // material. Instead, the event type is a generic, and the material is passed in.
    move |trigger, mut query| {
        if let Ok(mut material) = query.get_mut(trigger.entity()) {
            material.0 = new_material.clone();
        }
    }
}

// /// An observer to rotate an entity when it is dragged
// fn rotate_on_drag(drag: Trigger<Pointer<Drag>>, mut transforms: Query<&mut Transform>) {
//     let mut transform = transforms.get_mut(drag.entity()).unwrap();
//     transform.rotate_y(drag.delta.x * 0.02);
//     transform.rotate_x(drag.delta.y * 0.02);
// }

fn move_model(
    drag: Trigger<Pointer<Drag>>,
    mut models: Query<
        &mut Transform,
        (
            With<SceneRoot>,
            Without<Player>,
            Without<Camera3d>,
        ),
    >,
    player: Query<&Transform, With<Player>>,
) {
    let Ok(transform) = player.get_single() else {
        return;
    };
    let camera_rotation = transform.rotation.to_euler(EulerRot::XYZ).2;
    let (sin, cos) = camera_rotation.sin_cos();
  
    let Ok(mut model) = models.get_mut(drag.entity()) else {
        error!("Event for nonexistent model");
        return;
    };
    let delta = drag.delta;
    match drag.button {
        PointerButton::Primary => {
            model.translation.x += (delta.x * cos + delta.y * sin) * 0.05;
            model.translation.z += (-delta.x * sin + delta.y * cos) * 0.03;
        }
        PointerButton::Secondary => {
            model.rotate_local_y(drag.delta.x / 50.0);
            model.scale *= (drag.delta.y / -100.).exp().min(10.);
        }
        PointerButton::Middle => {
            model.translation.y += drag.delta.y * -0.05;
        }
    }
    
}