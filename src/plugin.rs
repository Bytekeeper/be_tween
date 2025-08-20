use crate::tween::*;
use bevy::audio::Volume;
use bevy::color::ColorRange;
use bevy::ecs::component::Mutable;
use bevy::prelude::*;
use std::marker::PhantomData;
use std::time::Duration;

#[derive(Clone)]
pub struct Start<T>(pub T);

impl TweenApplier<TweenBuffer<TweenTranslation>> for Start<TweenTranslation> {
    fn apply(&mut self, target: &mut TweenBuffer<TweenTranslation>, value: f32) {
        target.tween.start = self.0.start.lerp(self.0.end, value);
    }
}

#[derive(Clone)]
pub struct End<T>(pub T);

#[derive(Component, Clone, Debug, Default)]
pub struct TweenBuffer<T> {
    tween: T,
}

impl<T> TweenBuffer<T> {
    pub fn new(tween: T) -> Self {
        Self { tween }
    }
}

impl TweenApplier<TweenBuffer<TweenTranslation>> for End<TweenTranslation> {
    fn apply(&mut self, target: &mut TweenBuffer<TweenTranslation>, value: f32) {
        target.tween.end = self.0.start.lerp(self.0.end, value);
    }
}

#[derive(Bundle, Default)]
pub struct PlayBufferedTweenBundle<
    T: Component,
    B: 'static + Send + Sync,
    I: 'static + Send + Sync = (),
> {
    pub play_tween: PlayTween<(T, TweenBuffer<B>), I>,
    pub buffer: TweenBuffer<B>,
}

#[derive(Clone, Default)]
pub struct BufferApplier<T> {
    _phantom: PhantomData<T>,
}

impl<T> BufferApplier<T> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl TweenApplier<(Transform, TweenBuffer<TweenTranslation>)> for BufferApplier<TweenTranslation> {
    fn apply(&mut self, target: &mut (Transform, TweenBuffer<TweenTranslation>), value: f32) {
        target.1.tween.apply(&mut target.0, value);
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct TweenTweenTranslation {
    pub start: Vec3,
    pub end: Vec3,
}

#[derive(Component, Clone, Default)]
pub struct PlayTween<T, I> {
    tween: Tween<T>,
    despawn: bool,
    remove: bool,
    _time: PhantomData<I>,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct TweenTranslation {
    pub start: Vec3,
    pub end: Vec3,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct TweenScale {
    pub start: Vec3,
    pub end: Vec3,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct TweenSpriteColor {
    pub start: Color,
    pub end: Color,
}

impl TweenSpriteColor {
    pub fn new(start: impl Into<Color>, end: impl Into<Color>) -> Self {
        Self {
            start: start.into(),
            end: end.into(),
        }
    }
}

pub trait TweenTo {
    type Target;

    fn tween_to(self, to: impl Into<Self>) -> impl TweenApplier<Self::Target>
    where
        Self: Sized;
}

impl<I: Into<Color>> TweenTo for I {
    type Target = Sprite;

    fn tween_to(self, to: impl Into<Self>) -> impl TweenApplier<Self::Target> {
        TweenSpriteColor {
            start: self.into(),
            end: to.into().into(),
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct TweenBackgroundColor {
    pub start: Color,
    pub end: Color,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct TweenVolume {
    pub start: Volume,
    pub end: Volume,
}

impl TweenApplier<AudioSink> for TweenVolume {
    fn apply(&mut self, target: &mut AudioSink, value: f32) {
        target.set_volume(Volume::Linear(
            self.start.to_linear().lerp(self.end.to_linear(), value),
        ));
    }
}

pub trait ToTween<T> {
    fn tween(self, duration: Duration, function: impl Interpolator + 'static) -> Tween<T>;
}

impl<T, U: TweenApplier<T> + 'static> ToTween<T> for U {
    fn tween(self, duration: Duration, function: impl Interpolator + 'static) -> Tween<T> {
        Tween::new(duration, function, self)
    }
}

#[derive(Default)]
pub struct TweenPlugin;

impl<T> PlayTween<T, ()> {
    pub fn new(tween: Tween<T>) -> Self {
        Self::new_with_time(tween)
    }
}

impl<T> PlayTween<T, Real> {
    pub fn new_real_time(tween: Tween<T>) -> Self {
        Self::new_with_time(tween)
    }
}

impl<T, I> PlayTween<T, I> {
    pub fn new_with_time(tween: Tween<T>) -> Self {
        Self {
            tween,
            despawn: false,
            remove: false,
            _time: default(),
        }
    }

    /// After completing this tween, despawn the entity.
    pub fn despawn(self) -> Self {
        Self {
            despawn: true,
            ..self
        }
    }

    // After completing this tween, remove it (the component).
    pub fn remove_when_done(self) -> Self {
        Self {
            remove: true,
            ..self
        }
    }
}

impl Plugin for TweenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                play_tween_animation::<Transform, ()>,
                play_tween_animation::<Transform, Real>,
                play_tween_animation::<Sprite, ()>,
                play_tween_animation::<Sprite, Real>,
                play_tween_animation::<BackgroundColor, ()>,
                play_tween_animation::<BackgroundColor, Real>,
                play_tween_animation::<AudioSink, ()>,
                play_tween_animation::<AudioSink, Real>,
                play_tween_animation::<TweenBuffer<TweenTranslation>, ()>,
                play_tween_animation::<TweenBuffer<TweenTranslation>, Real>,
                play_buffered_tween_animation::<Transform, TweenTranslation, ()>,
                play_buffered_tween_animation::<Transform, TweenTranslation, Real>,
            )
                .chain(),
        );
    }
}

pub fn play_buffered_tween_animation<
    T: Component<Mutability = Mutable> + Clone,
    W: TweenApplier<T> + 'static + Clone,
    I: Default + Send + Sync + 'static,
>(
    time: Res<Time<I>>,
    mut tweens_to_play: Query<(
        Entity,
        &mut PlayTween<(T, TweenBuffer<W>), I>,
        &mut T,
        Option<&mut TweenBuffer<W>>,
    )>,
    mut commands: Commands,
) {
    for (entity, mut play, mut target, tween_buffer) in tweens_to_play.iter_mut() {
        let Some(mut tween_buffer) = tween_buffer else {
            error!("Buffered PlayTween without Buffer component");
            continue;
        };
        // TODO find a way without moving data around
        let mut tmp_target = (target.clone(), tween_buffer.clone());
        let result = play.tween.advance(&mut tmp_target, time.delta());
        *target = tmp_target.0;
        *tween_buffer = tmp_target.1;
        if matches!(result, TweenProgress::Done { .. }) {
            if play.remove {
                commands
                    .entity(entity)
                    .remove::<PlayTween<(T, TweenBuffer<W>), I>>();
            }
            if play.despawn {
                commands.entity(entity).despawn();
            }
        }
    }
}

pub fn play_tween_animation<
    T: Component<Mutability = Mutable>,
    I: Default + Send + Sync + 'static,
>(
    time: Res<Time<I>>,
    mut tweens_to_play: Query<(Entity, &mut PlayTween<T, I>, &mut T)>,
    mut commands: Commands,
) {
    for (entity, mut play, mut target) in tweens_to_play.iter_mut() {
        let result = play.tween.advance(&mut target, time.delta());
        if matches!(result, TweenProgress::Done { .. }) {
            if play.remove {
                commands.entity(entity).remove::<PlayTween<T, I>>();
            }
            if play.despawn {
                commands.entity(entity).despawn();
            }
        }
    }
}

impl TweenApplier<Transform> for TweenTranslation {
    fn apply(&mut self, target: &mut Transform, value: f32) {
        target.translation = self.start.lerp(self.end, value);
    }
}

impl TweenApplier<BackgroundColor> for TweenBackgroundColor {
    fn apply(&mut self, target: &mut BackgroundColor, value: f32) {
        target.0 = (self.start..self.end).at(value);
    }
}

/// Please note this uses LCH color space and RGB
impl TweenApplier<Sprite> for TweenSpriteColor {
    fn apply(&mut self, target: &mut Sprite, value: f32) {
        target.color = (self.start..self.end).at(value);
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
        world.insert_resource(Time::<Real>::default());
        let play_tween_id = world.register_system(play_tween_animation::<Transform, ()>);
        let play_tween = PlayTween::new(Tween::new(
            Duration::from_secs(2),
            Lerp,
            TweenTranslation {
                start: Vec3::ZERO,
                end: Vec3::X,
            },
        ));
        let to_transform = world.spawn((Transform::default(), play_tween)).id();

        // WHEN
        world.run_system(play_tween_id).unwrap();

        // THEN
        let transform = world.get::<Transform>(to_transform).unwrap();
        assert_eq!(transform.translation, Vec3::X * 0.5);
    }

    #[test]
    fn test_real_time() {
        // GIVEN
        let mut world = World::new();
        let mut time = Time::<Real>::default();
        time.advance_by(Duration::from_secs(1));
        world.insert_resource(time);
        let play_tween_id_real = world.register_system(play_tween_animation::<Transform, Real>);
        let play_tween =
            PlayTween::new_real_time(Tween::<Transform>::pause(Duration::from_secs(2)));
        let entity = world.spawn((Transform::default(), play_tween)).id();

        // WHEN
        world.run_system(play_tween_id_real).unwrap();

        // THEN
        let mut tween = world.get_mut::<PlayTween<Transform, Real>>(entity).unwrap();
        assert_eq!(
            tween.tween.skip(Duration::from_secs(2)),
            TweenProgress::Done {
                surplus: Duration::from_secs(1)
            }
        );
    }

    #[test]
    fn test_tweening_tweens() {
        let mut world = World::new();
        let mut time = Time::<()>::default();
        time.advance_by(Duration::from_secs(2));
        world.insert_resource(time);
        let mut time = Time::<Real>::default();
        time.advance_by(Duration::from_secs(1));
        world.insert_resource(time);
        world.insert_resource(time);
        let play_tween_id_real = world
            .register_system(play_buffered_tween_animation::<Transform, TweenTranslation, Real>);
        let play_tween_tween =
            world.register_system(play_tween_animation::<TweenBuffer<TweenTranslation>, ()>);
        let play_tween_tween_real =
            world.register_system(play_tween_animation::<TweenBuffer<TweenTranslation>, Real>);

        let real_time_tween = PlayTween::new_real_time(Tween::new(
            Duration::from_secs(2),
            Lerp,
            Start(TweenTranslation {
                start: Vec3::ZERO,
                end: Vec3::X,
            }),
        ));
        let virtual_time_tween = PlayTween::new(Tween::new(
            Duration::from_secs(2),
            Lerp,
            End(TweenTranslation {
                start: Vec3::ZERO,
                end: Vec3::X,
            }),
        ));
        let to_transform = world
            .spawn((
                Transform::default(),
                PlayBufferedTweenBundle {
                    play_tween: PlayTween::new_real_time(Tween::new(
                        Duration::from_secs(2),
                        Lerp,
                        BufferApplier::new(),
                    )),
                    ..default()
                },
                real_time_tween,
                virtual_time_tween,
            ))
            .id();

        // WHEN
        world.run_system(play_tween_tween).unwrap();
        world.run_system(play_tween_tween_real).unwrap();
        world.run_system(play_tween_id_real).unwrap();

        // THEN
        let transform = world.get::<Transform>(to_transform).unwrap();
        assert_eq!(transform.translation, Vec3::X * 0.75);
        let tween_buffer = world
            .get::<TweenBuffer<TweenTranslation>>(to_transform)
            .unwrap();
        assert_eq!(tween_buffer.tween.start, Vec3::X * 0.5);
        assert_eq!(tween_buffer.tween.end, Vec3::X);
    }
}
