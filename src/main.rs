use bevy::prelude::*;
use rand::Rng;

const SCOREBOARD_FONT_SIZE: f32 = 40.0;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);

const TEXT_COLOR: Color = Color::srgb(0.98, 0.561, 0.329);
const SCORE_COLOR: Color = Color::srgb(0.98, 0.722, 0.11);

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
struct PipesSpawnTimer(Timer);

fn update_scoreboard(score: Res<Score>, mut query: Query<&mut Text, With<ScoreBoardUi>>) {
    let mut text = query.single_mut();
    text.sections[1].value = score.to_string();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn(PipesSpawnTimer(Timer::from_seconds(
        1.0,
        TimerMode::Repeating,
    )));

    commands.spawn((
        ScoreBoardUi,
        TextBundle::from_sections([
            TextSection::new(
                "Score: ",
                TextStyle {
                    font_size: SCOREBOARD_FONT_SIZE,
                    color: TEXT_COLOR,
                    ..default()
                },
            ),
            TextSection::from_style(TextStyle {
                font_size: SCOREBOARD_FONT_SIZE,
                color: SCORE_COLOR,
                ..default()
            }),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: SCOREBOARD_TEXT_PADDING,
            left: SCOREBOARD_TEXT_PADDING,
            ..default()
        }),
    ));
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
        texture: background.0.clone(),
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
            texture: bird.0.clone(),
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
    let window = query_window.single();
    let (width, height) = (window.width(), window.height());

    let mut timer = query_timer.single_mut();
    let mut rng = rand::thread_rng();

    if timer.0.tick(time.delta()).just_finished() && !game_state.did_end {
        commands.spawn((
            PipesTranslate { velocity: 0.0 },
            SpriteBundle {
                texture: pipes.0.clone(),
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
        ));

        commands.spawn((
            PipesTranslate { velocity: 0.0 },
            SpriteBundle {
                texture: pipes.0.clone(),
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
        ));
    }
}

#[derive(Component)]
struct BirdTranslate {
    velocity: f32,
}

#[derive(Component)]
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
    if !game_state.did_start {
        return;
    }

    let window = query_window.single();
    let (mut bird, mut transform) = query.single_mut();

    let t = time.delta_seconds();
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
    if !game_state.did_start {
        return;
    }

    let t = time.delta_seconds();
    for (pipes, mut transform) in query.iter_mut() {
        transform.translation.x += (pipes.velocity - 200.0) * t;
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Score(0))
        .insert_resource(GameState {
            did_start: false,
            did_end: false,
        })
        .add_systems(Startup, setup)
        .add_systems(Startup, (load_images, spawn_background, spawn_bird).chain())
        .add_systems(Update, spawn_pipes)
        .add_systems(Update, update_scoreboard)
        .add_systems(Update, (bird_flap, bird_mechanics, pipe_mechanics))
        .run();
}
