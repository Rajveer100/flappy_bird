use bevy::prelude::*;

const SCOREBOARD_FONT_SIZE: f32 = 40.0;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);

const TEXT_COLOR: Color = Color::srgb(0.98, 0.561, 0.329);
const SCORE_COLOR: Color = Color::srgb(0.98, 0.722, 0.11);

#[derive(Resource, Deref, DerefMut)]
struct Score(u32);

#[derive(Component)]
struct ScoreBoardUi;

fn update_scoreboard(score: Res<Score>, mut query: Query<&mut Text, With<ScoreBoardUi>>) {
    let mut text = query.single_mut();
    text.sections[1].value = score.to_string();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

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

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Score(0))
        .add_systems(Startup, setup)
        .add_systems(Update, update_scoreboard)
        .run();
}
