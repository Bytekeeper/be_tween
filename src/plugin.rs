use crate::tween::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct PlayTween<T> {
    pub tween: Tween<T>,
}

pub struct TweenTranslation {
    pub start: Vec3,
    pub end: Vec3,
}

pub struct TweenScale {
    pub start: Vec3,
    pub end: Vec3,
}

pub struct TweenSpriteColor {
    pub start: Color,
    pub end: Color,
}

pub struct TweenPlugin;

impl<T> PlayTween<T> {
    pub fn new(tween: Tween<T>) -> Self {
        Self { tween }
    }
}

impl Plugin for TweenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (
                play_tween_animation::<Transform>,
                play_tween_animation::<Sprite>,
            ),
        );
    }
}

pub fn play_tween_animation<T: Component>(
    time: Res<Time>,
    mut tweens_to_play: Query<(&mut PlayTween<T>, &mut T)>,
) {
    for (mut play, mut target) in tweens_to_play.iter_mut() {
        play.tween.advance(&mut target, time.delta());
    }
}

impl TweenApplier<Transform> for TweenTranslation {
    fn apply(&mut self, target: &mut Transform, value: f32) {
        target.translation = self.start.lerp(self.end, value);
    }
}

impl TweenApplier<Sprite> for TweenSpriteColor {
    fn apply(&mut self, target: &mut Sprite, value: f32) {
        target.color = Color::lcha_from_array(
            self.start
                .lcha_to_vec4()
                .lerp(self.end.lcha_to_vec4(), value),
        );
    }
}

impl TweenApplier<Transform> for TweenScale {
    fn apply(&mut self, target: &mut Transform, value: f32) {
        target.scale = self.start.lerp(self.end, value);
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
        let mut time = Time::<()>::default();
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
