use crate::tween::*;
use bevy::prelude::*;
use std::marker::PhantomData;

#[derive(Component, Clone)]
pub struct PlayTween<T, E, I> {
    tween: Tween<T, E>,
    despawn: bool,
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

#[derive(Default, Debug, Clone, Copy)]
pub struct TweenBackgroundColor {
    pub start: Color,
    pub end: Color,
}

pub struct TweenPlugin<E> {
    _phantom: std::marker::PhantomData<E>,
}

impl Event for NoEvent {}

impl<T, E> PlayTween<T, E, ()> {
    pub fn new(tween: Tween<T, E>) -> Self {
        Self::new_with_time(tween)
    }
}

impl<T, E> PlayTween<T, E, Real> {
    pub fn new_real_time(tween: Tween<T, E>) -> Self {
        Self::new_with_time(tween)
    }
}

impl<T, E, I> PlayTween<T, E, I> {
    pub fn new_with_time(tween: Tween<T, E>) -> Self {
        Self {
            tween,
            despawn: false,
            _time: default(),
        }
    }

    pub fn despawn(self) -> Self {
        Self {
            despawn: true,
            ..self
        }
    }
}

impl<E> TweenPlugin<E> {
    pub fn new() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }
}

impl<E> Default for TweenPlugin<E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E: Event + Clone> Plugin for TweenPlugin<E> {
    fn build(&self, app: &mut App) {
        app.add_event::<NoEvent>().add_event::<E>();
    }
}

impl<'w, E: Event + Clone> EventSender<E> for EventWriter<'w, E> {
    fn send(&mut self, event: &E) {
        EventWriter::send(self, event.clone());
    }
}

pub fn play_tween_animation<T: Component, E: Event + Clone, I: Default + Send + Sync + 'static>(
    time: Res<Time<I>>,
    mut tweens_to_play: Query<(Entity, &mut PlayTween<T, E, I>, &mut T)>,
    mut event_writer: EventWriter<E>,
    mut commands: Commands,
) {
    for (entity, mut play, mut target) in tweens_to_play.iter_mut() {
        let result = play
            .tween
            .advance(&mut target, &mut event_writer, time.delta());
        if play.despawn && matches!(result, TweenProgress::Done { .. }) {
            commands.entity(entity).despawn();
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
        target.0 = Color::lcha_from_array(
            self.start
                .lcha_to_vec4()
                .lerp(self.end.lcha_to_vec4(), value),
        );
    }
}

/// Please note this uses LCH color space and RGB
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

    #[derive(Event, Clone)]
    struct TestEvent;

    #[test]
    fn test_transform_tween() {
        // GIVEN
        let mut world = World::new();
        let mut time = Time::<()>::default();
        time.advance_by(Duration::from_secs(1));
        world.insert_resource(time);
        world.insert_resource(Time::<Real>::default());
        world.init_resource::<Events<NoEvent>>();
        let play_tween_id = world.register_system(play_tween_animation::<Transform, NoEvent, ()>);
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
    fn test_tween_event() {
        // GIVEN
        let mut world = World::new();
        let mut time = Time::<()>::default();
        time.advance_by(Duration::from_secs(3));
        world.insert_resource(time);
        world.insert_resource(Time::<Real>::default());
        world.init_resource::<Events<TestEvent>>();
        let play_tween_id = world.register_system(play_tween_animation::<Transform, TestEvent, ()>);
        let play_tween = PlayTween::new(
            Tween::<Transform, TestEvent>::pause(Duration::from_secs(2)).with_completed(TestEvent),
        );
        world.spawn((Transform::default(), play_tween));

        // WHEN
        world.run_system(play_tween_id).unwrap();

        // THEN
        let events = world.get_resource::<Events<TestEvent>>().unwrap();
        let mut reader = events.get_reader();
        assert_eq!(reader.read(&events).count(), 1);
    }

    #[test]
    fn test_real_time() {
        // GIVEN
        let mut world = World::new();
        let mut time = Time::<()>::default();
        time.advance_by(Duration::from_secs(2));
        world.insert_resource(time);
        let mut time = Time::<Real>::default();
        time.advance_by(Duration::from_secs(1));
        world.insert_resource(time);
        world.init_resource::<Events<TestEvent>>();
        let play_tween_id_real =
            world.register_system(play_tween_animation::<Transform, TestEvent, Real>);
        let play_tween_id_virtual =
            world.register_system(play_tween_animation::<Transform, TestEvent, ()>);
        let play_tween = PlayTween::<_, _, Real>::new_with_time(
            Tween::<Transform, TestEvent>::pause(Duration::from_secs(2)).with_completed(TestEvent),
        );
        world.spawn((Transform::default(), play_tween));

        // WHEN
        world.run_system(play_tween_id_real).unwrap();
        world.run_system(play_tween_id_virtual).unwrap();

        // THEN
        let events = world.get_resource::<Events<TestEvent>>().unwrap();
        assert!(events.is_empty());
    }
}
