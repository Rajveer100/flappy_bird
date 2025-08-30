use bevy::{
    math::bounding::{Aabb2d, BoundingCircle, IntersectsVolume},
    prelude::*,
    text::TextSpanComponent,
    utils::HashSet,
    winit::WinitSettings,
};
use rand::Rng;

const SCOREBOARD_FONT_SIZE: f32 = 40.0;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);

const TEXT_COLOR: Color = Color::srgb(0.98, 0.561, 0.329);
const SCORE_COLOR: Color = Color::srgb(0.98, 0.722, 0.11);

const BIRD_SIZE: (f32, f32) = (558.0, 447.0);
const PIPE_SIZE: (f32, f32) = (292.0, 855.0);

const BIRD_PIPE_COLLISION_OFFSET: (f32, f32) = (120.0, 60.0);

#[cfg(target_os = "ios")]
#[no_mangle]
pub extern "C" fn main_rs() {
    main();
}

fn main() {
    let mut app = App::new();

    #[cfg(target_os = "ios")]
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            resizable: false,
            mode: bevy::window::WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
            ..default()
        }),
        ..default()
    }))
    .insert_resource(WinitSettings::mobile());

    #[cfg(not(target_os = "ios"))]
    app.add_plugins(DefaultPlugins);

    app.insert_resource(Score(0))
        .insert_resource(GameState {
            did_start: false,
            did_end: false,
        })
        .insert_resource(PipesPassedThrough(HashSet::new()))
        .add_event::<CollisionEvent>()
        .add_systems(Startup, setup)
        .add_systems(Startup, (load_images, spawn_background, spawn_bird).chain())
        .add_systems(Update, spawn_pipes)
        .add_systems(Update, update_scoreboard)
        .add_systems(Update, (bird_flap, bird_mechanics, pipe_mechanics))
        .add_systems(Update, check_for_collisions)
        .run();
}

#[derive(Resource)]
struct GameState {
    did_start: bool,
    did_end: bool,
}

#[derive(Resource, Deref, DerefMut)]
struct Score(u32);

#[derive(Component)]
struct ScoreBoardUi;

#[derive(Component)]
struct ScoreValueText;

#[derive(Component)]
struct PipesSpawnTimer(Timer);

#[derive(Resource, Deref, DerefMut)]
struct PipesPassedThrough(HashSet<Entity>);

fn update_scoreboard(
    score: Res<Score>,
    query: Query<&Children, With<ScoreBoardUi>>,
    mut writer: TextUiWriter,
) {
    if let Ok(children) = query.get_single() {
        if children.len() > 2 {
            *writer.text(children[2], 0) = (**score / 2).to_string();
        }
    }
}

fn check_for_collisions(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    mut score: ResMut<Score>,
    mut pipes_passed_through: ResMut<PipesPassedThrough>,
    mut bird_query: Query<(&mut BirdTranslate, &Transform)>,
    collider_query: Query<(Entity, &Transform, Option<&PipesTranslate>), With<Collider>>,
    mut collision_events: EventWriter<CollisionEvent>,
) {
    if !game_state.did_start || game_state.did_end {
        return;
    }

    let (mut _bird_velocity, bird_transform) = bird_query.single_mut();

    for (collider_entity, collider_transform, pipes_translate) in &collider_query {
        if pipes_translate.is_some() {
            let bird_scale = bird_transform.scale.truncate();

            let bird_size_x = BIRD_SIZE.0 * bird_scale.x;
            let bird_size_y = BIRD_SIZE.1 * bird_scale.y;
            let bird_diameter = ((bird_size_x * bird_size_x) + (bird_size_y * bird_size_y)).sqrt();

            let pipe_scale = collider_transform.scale.truncate();

            let pipe_size_x = PIPE_SIZE.0 * pipe_scale.x - BIRD_PIPE_COLLISION_OFFSET.0;
            let pipe_size_y = PIPE_SIZE.1 * pipe_scale.y - BIRD_PIPE_COLLISION_OFFSET.1;

            if bird_collision(
                BoundingCircle::new(bird_transform.translation.truncate(), bird_diameter / 2.0),
                Aabb2d::new(
                    collider_transform.translation.truncate(),
                    Vec2::new(pipe_size_x, pipe_size_y) / 2.0,
                ),
            ) {
                game_state.did_end = true;
            } else {
                if bird_transform.translation.x - bird_diameter / 2.0
                    > collider_transform.translation.x
                {
                    if !pipes_passed_through.contains(&collider_entity) {
                        **score += 1;
                        pipes_passed_through.insert(collider_entity);
                    }
                }
            }
        }
    }
}

fn bird_collision(bird: BoundingCircle, bounding_box: Aabb2d) -> bool {
    return if bird.intersects(&bounding_box) {
        true
    } else {
        false
    };
}

#[derive(Component)]
struct Collider;

#[derive(Event, Default)]
struct CollisionEvent;

fn setup(mut commands: Commands) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    // PipesSpawn Timer
    commands.spawn(PipesSpawnTimer(Timer::from_seconds(
        1.0,
        TimerMode::Repeating,
    )));

    // Scoreboard UI
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: SCOREBOARD_TEXT_PADDING,
                left: SCOREBOARD_TEXT_PADDING,
                ..default()
            },
            ScoreBoardUi,
        ))
        .with_children(|parent| {
            parent.spawn(ScoreBoardUi);
            parent.spawn((
                Text::new("Score: "),
                TextFont {
                    font_size: SCOREBOARD_FONT_SIZE,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));
            parent.spawn((
                Text::new("0"),
                TextFont {
                    font_size: SCOREBOARD_FONT_SIZE,
                    ..default()
                },
                TextColor(SCORE_COLOR),
                ScoreValueText
            ));
        });
}

#[derive(Resource)]
struct Background(Handle<Image>);

#[derive(Resource)]
struct Bird(Handle<Image>);

#[derive(Resource)]
struct Pipes(Handle<Image>);

fn load_images(mut commands: Commands, server: Res<AssetServer>) {
    let background_handle: Handle<Image> = server.load("forest_bg.png");
    let bird_handle: Handle<Image> = server.load("bird.png");
    let pipes_handle: Handle<Image> = server.load("pipes.png");

    commands.insert_resource(Background(background_handle));
    commands.insert_resource(Bird(bird_handle));
    commands.insert_resource(Pipes(pipes_handle));
}

fn spawn_background(mut commands: Commands, background: Res<Background>) {
    commands.spawn(SpriteBundle {
        sprite: background.0.clone().into(),
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            scale: Vec3::new(1.0, 1.0, 1.0),
            ..default()
        },
        ..default()
    });
}

fn spawn_bird(mut commands: Commands, bird: Res<Bird>) {
    commands.spawn((
        BirdTranslate { velocity: 0.0 },
        SpriteBundle {
            sprite: bird.0.clone().into(),
            transform: Transform {
                translation: Vec3::new(-200.0, 0.0, 0.1),
                scale: Vec3::new(0.2, 0.2, 0.0),
                ..default()
            },
            ..default()
        },
    ));
}

fn spawn_pipes(
    mut commands: Commands,
    pipes: Res<Pipes>,
    game_state: Res<GameState>,
    query_window: Query<&Window>,
    time: Res<Time>,
    mut query_timer: Query<&mut PipesSpawnTimer>,
) {
    if !game_state.did_start || game_state.did_end {
        return;
    }

    let window = query_window.single();
    let (width, height) = (window.width(), window.height());

    let mut timer = query_timer.single_mut();
    let mut rng = rand::thread_rng();

    if timer.0.tick(time.delta()).just_finished() && !game_state.did_end {
        commands.spawn((
            PipesTranslate { velocity: 0.0 },
            SpriteBundle {
                sprite: pipes.0.clone().into(),
                transform: Transform {
                    translation: Vec3::new(
                        width / 2.0,
                        height / 2.0 - rng.gen_range(0.0..90.0),
                        0.2,
                    ),
                    scale: Vec3::new(0.5, 0.5, 0.0),
                    ..default()
                },
                ..default()
            },
            Collider,
        ));

        commands.spawn((
            PipesTranslate { velocity: 0.0 },
            SpriteBundle {
                sprite: pipes.0.clone().into(),
                transform: Transform {
                    translation: Vec3::new(
                        width / 2.0,
                        -height / 2.0 + rng.gen_range(0.0..90.0),
                        0.2,
                    ),
                    scale: Vec3::new(0.5, 0.5, 0.0),
                    ..default()
                },
                ..default()
            },
            Collider,
        ));
    }
}

#[derive(Component)]
struct BirdTranslate {
    velocity: f32,
}

#[derive(Component, Debug)]
struct PipesTranslate {
    velocity: f32,
}

fn bird_flap(
    mut query: Query<&mut BirdTranslate>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<GameState>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        let mut bird = query.single_mut();
        bird.velocity = 300.0;
        game_state.did_start = true;
    }
}

fn bird_mechanics(
    time: Res<Time>,
    mut game_state: ResMut<GameState>,
    query_window: Query<&Window>,
    mut query: Query<(&mut BirdTranslate, &mut Transform)>,
) {
    if !game_state.did_start || game_state.did_end {
        return;
    }

    let window = query_window.single();
    let (mut bird, mut transform) = query.single_mut();

    let t = time.delta_secs();
    let delta_dist = bird.velocity * t;

    if transform.translation.y + delta_dist < -window.height() / 2.0 {
        bird.velocity = 0.0;
        game_state.did_end = true;
    } else {
        bird.velocity -= 800.0 * t;
        transform.translation.y += bird.velocity * t;
    }
}

fn pipe_mechanics(
    time: Res<Time>,
    game_state: ResMut<GameState>,
    mut query: Query<(&mut PipesTranslate, &mut Transform)>,
) {
    if !game_state.did_start || game_state.did_end {
        return;
    }

    let t = time.delta_secs();
    for (pipes, mut transform) in query.iter_mut() {
        transform.translation.x += (pipes.velocity - 200.0) * t;
    }
}
