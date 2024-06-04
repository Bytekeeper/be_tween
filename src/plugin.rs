use crate::tween::*;
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_time::prelude::*;
use bevy_transform::prelude::*;
use glam::*;

#[derive(Component)]
pub struct PlayTween<T> {
    pub tween: Tween<T>,
}

pub struct TweenPlugin;

impl Plugin for TweenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, play_tween_animation::<Transform>);
    }
}

pub fn play_tween_animation<T: Component>(
    time: Res<Time<Virtual>>,
    mut tweens_to_play: Query<(&mut PlayTween<T>, &mut T)>,
) {
    for (mut play, mut target) in tweens_to_play.iter_mut() {
        play.tween.advance(&mut target, time.elapsed());
    }
}

pub struct TweenTranslation {
    pub start: Vec3,
    pub end: Vec3,
}

impl TweenApplier<Transform> for TweenTranslation {
    fn apply(&mut self, target: &mut Transform, value: f32) {
        target.translation = self.start.lerp(self.end, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_transform_tween() {
        // GIVEN
        let mut world = World::new();
        let mut time = Time::new_with(Virtual::default());
        time.advance_by(Duration::from_secs(1));
        world.insert_resource(time);
        let play_tween_id = world.register_system(play_tween_animation::<Transform>);
        let play_tween = PlayTween {
            tween: Tween::new(
                Duration::from_secs(2),
                Lerp,
                TweenTranslation {
                    start: Vec3::ZERO,
                    end: Vec3::X,
                },
            ),
        };
        let to_transform = world.spawn((Transform::default(), play_tween)).id();

        // WHEN
        world.run_system(play_tween_id).unwrap();

        // THEN
        let transform = world.get::<Transform>(to_transform).unwrap();
        assert_eq!(transform.translation, Vec3::X * 0.5);
    }
}
