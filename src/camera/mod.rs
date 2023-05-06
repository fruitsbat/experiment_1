use crate::player::Player;
use bevy::prelude::*;
use bevy_pixel_camera::{PixelCameraBundle, PixelCameraPlugin};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(PixelCameraPlugin)
            .add_startup_system(init_camera)
            .add_system(follow_player);
    }
}

#[derive(Component)]
struct PlayerCam;

fn init_camera(mut commands: Commands) {
    commands.spawn((PixelCameraBundle::from_resolution(320, 320), PlayerCam));
}

fn follow_player(
    mut camera_query: Query<&mut Transform, (With<PlayerCam>, Without<Player>)>,
    player_query: Query<(&Transform, &Player), Without<PlayerCam>>,
    time: Res<Time>,
) {
    for mut camera in camera_query.iter_mut() {
        for (transform, player) in player_query.iter() {
            camera.translation = camera.translation.lerp(
                transform.translation
                    + Vec3 {
                        x: player.velocity.x * 20.,
                        y: 0.,
                        z: 0.,
                    },
                10. * time.delta_seconds(),
            );
        }
    }
}
