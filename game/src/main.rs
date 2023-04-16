use s4reader::*;
use bevy::prelude::*;
use bevy::render::render_resource::{Texture, Extent3d, TextureDimension, TextureFormat};

#[derive(Component)]
struct Person;

#[derive(Component)]
struct Name(String);

#[derive(Resource)]
struct GreetTimer(Timer);

fn add_people(mut commands: Commands) {
    commands.spawn((Person, Name("Elaina Proctor".to_string())));
    commands.spawn((Person, Name("Renzo Hume".to_string())));
    commands.spawn((Person, Name("Zayna Nieves".to_string())));
}

fn greet_people(time: Res<Time>, mut timer: ResMut<GreetTimer>, query: Query<&Name, With<Person>>) {
    if timer.0.tick(time.delta()).just_finished() {
        for name in &query {
            println!("hello {}!", name.0);
        }
    }
}

fn setup_sprite_thing(
    mut commands: Commands,
    mut textures: ResMut<Assets<Image>>,
) {
    // let mut preview = settlers::Preview {
    //     data: Vec::<u8>::new(),
    // };

    let (width, height) = (256, 256);
    let mut bytes = Vec::with_capacity(width * height * 4);
    for _y in 0..height {
        for _x in 0..width {
            bytes.push(0xff);
            bytes.push(0x00);
            bytes.push(0x00);
            bytes.push(0xff);
        }
    }

    let texture = Image::new(
        Extent3d { width: width as u32, height: height as u32, depth_or_array_layers: 1 },
        TextureDimension::D2,
        bytes,
        TextureFormat::Rgba8Unorm,
    );

    let texture_handle = textures.add(texture);
    //asset_server.add_texture(texture_handle);

    commands.spawn(Camera2dBundle::default());
    commands.spawn(SpriteBundle {
        texture: texture_handle,
        ..Default::default()
    });
}

// fn preview(mut commands: Commands, asset_server: Res<AssetServer>) {
//     commands.spawn(Camera2dBundle::default());
//     commands.spawn(SpriteBundle {
//         texture: Image::new(),
//         ..default()
//     });
// }

pub struct HelloPlugin;

impl Plugin for HelloPlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(GreetTimer(Timer::from_seconds(2.0, TimerMode::Repeating)))
        .add_startup_system(setup_sprite_thing)
        .add_system(greet_people);
    }
}

fn main() {
    let game_map = map::file::GameMap::from_file("s4reader/map/Aeneas.map").unwrap();
    dbg!(game_map);
    // App::new()
    // .add_plugins(DefaultPlugins)
    // .add_plugin(HelloPlugin)
    // .run();
    //map::file::Map::open("../s4reader/map/Aeneas.map").unwrap();
}
