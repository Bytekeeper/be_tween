use bevy::log::error;
use dyn_clone::DynClone;
use interpolation::Ease;
pub use interpolation::EaseFunction;
use std::time::Duration;

pub trait TweenApplier<T>: Send + Sync + DynClone {
    fn apply(&mut self, target: &mut T, value: f32);
}

pub trait Interpolator: Send + Sync + 'static + DynClone {
    fn interpolate(&self, position: f32) -> f32;
}

dyn_clone::clone_trait_object!(<T> TweenApplier<T>);
dyn_clone::clone_trait_object!(Interpolator);

pub trait EventSender<E> {
    fn send(&mut self, event: &E);
}

#[derive(Copy, Clone)]
pub struct NoEvent;

#[derive(Copy, Clone)]
pub struct Lerp;

#[derive(Clone)]
pub enum Tween<T, E> {
    Once {
        duration: Duration,
        elapsed: Duration,
        function: Box<dyn Interpolator>,
        applier: Box<dyn TweenApplier<T> + 'static>,
        completed_event: Option<E>,
    },
    Repeat {
        tween: Box<Tween<T, E>>,
        times: RepeatTimes,
        count: usize,
        completed_event: Option<E>,
    },
    Sequence {
        index: usize,
        tweens: Vec<Tween<T, E>>,
        completed_event: Option<E>,
    },
    Parallel {
        tweens: Vec<Tween<T, E>>,
        completed_event: Option<E>,
    },
    Pause {
        duration: Duration,
        elapsed: Duration,
        completed_event: Option<E>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TweenProgress {
    Running,
    Done { surplus: Duration },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeatTimes {
    N(usize),
    Infinite,
}

impl Default for RepeatTimes {
    fn default() -> Self {
        Self::N(1)
    }
}

impl Interpolator for Lerp {
    fn interpolate(&self, position: f32) -> f32 {
        position
    }
}

impl Interpolator for EaseFunction {
    fn interpolate(&self, position: f32) -> f32 {
        position.calc(*self)
    }
}

impl<E> EventSender<E> for NoEvent {
    fn send(&mut self, _: &E) {}
}

impl<T> Tween<T, NoEvent> {
    pub fn new(
        duration: Duration,
        function: impl Interpolator + Sync + Send + 'static,
        applier: impl TweenApplier<T> + Sync + Send + 'static,
    ) -> Self {
        Self::Once {
            duration,
            elapsed: Duration::ZERO,
            function: Box::new(function),
            applier: Box::new(applier),
            completed_event: None,
        }
    }
}

impl<T, E> Tween<T, E> {
    pub fn new_with_event(
        duration: Duration,
        function: impl Interpolator + Sync + Send + 'static,
        applier: impl TweenApplier<T> + Sync + Send + 'static,
        completed_event: E,
    ) -> Self {
        Self::Once {
            duration,
            elapsed: Duration::ZERO,
            function: Box::new(function),
            applier: Box::new(applier),
            completed_event: Some(completed_event),
        }
    }

    pub fn pause(duration: Duration) -> Self {
        Self::Pause {
            duration,
            elapsed: Duration::ZERO,
            completed_event: None,
        }
    }

    pub fn repeat(times: RepeatTimes, tween: Tween<T, E>) -> Self {
        Self::Repeat {
            times,
            count: 0,
            tween: Box::new(tween),
            completed_event: None,
        }
    }

    pub fn sequence(tweens: impl Into<Vec<Tween<T, E>>>) -> Self {
        Self::Sequence {
            index: 0,
            tweens: tweens.into(),
            completed_event: None,
        }
    }

    pub fn parallel(tweens: impl Into<Vec<Tween<T, E>>>) -> Self {
        Self::Parallel {
            tweens: tweens.into(),
            completed_event: None,
        }
    }

    pub fn with_completed(mut self, event: E) -> Self {
        match &mut self {
            Tween::Once {
                completed_event, ..
            }
            | Tween::Repeat {
                completed_event, ..
            }
            | Tween::Sequence {
                completed_event, ..
            }
            | Tween::Parallel {
                completed_event, ..
            }
            | Tween::Pause {
                completed_event, ..
            } => *completed_event = Some(event),
        }
        self
    }

    pub fn skip(&mut self, mut duration: Duration) -> TweenProgress {
        match self {
            Tween::Once {
                duration: tween_duration,
                elapsed,
                ..
            } => {
                *elapsed += duration;
                if elapsed >= tween_duration {
                    let surplus = *elapsed - *tween_duration;
                    *elapsed = *tween_duration;
                    TweenProgress::Done { surplus }
                } else {
                    TweenProgress::Running
                }
            }
            Tween::Repeat {
                tween,
                times,
                count,
                ..
            } => loop {
                let done = match times {
                    RepeatTimes::N(amount) => count >= amount,
                    RepeatTimes::Infinite => false,
                };
                if done {
                    return TweenProgress::Done { surplus: duration };
                }
                let delegate_result = tween.skip(duration);
                match delegate_result {
                    TweenProgress::Done { surplus } => {
                        *count += 1;
                        if duration <= surplus && *times == RepeatTimes::Infinite {
                            error!("Found infinite repeating tween with zero duration child (infinite loop)");
                            return TweenProgress::Running;
                        }
                        duration = surplus;
                        tween.reset();
                    }
                    TweenProgress::Running => {
                        break TweenProgress::Running;
                    }
                }
            },
            Tween::Sequence { index, tweens, .. } => {
                while let Some(tween) = tweens.get_mut(*index) {
                    let delegate_result = tween.skip(duration);
                    match delegate_result {
                        TweenProgress::Done { surplus } => {
                            *index += 1;
                            duration = surplus;
                        }
                        TweenProgress::Running => {
                            return TweenProgress::Running;
                        }
                    }
                }
                TweenProgress::Done { surplus: duration }
            }
            Tween::Parallel { tweens, .. } => {
                tweens
                    .iter_mut()
                    .fold(TweenProgress::Done { surplus: duration }, |acc, tween| {
                        let delegate_result = tween.skip(duration);
                        if let (
                            TweenProgress::Done {
                                surplus: acc_surplus,
                            },
                            TweenProgress::Done {
                                surplus: delegate_surplus,
                            },
                        ) = (acc, delegate_result)
                        {
                            TweenProgress::Done {
                                surplus: acc_surplus.min(delegate_surplus),
                            }
                        } else {
                            TweenProgress::Running
                        }
                    })
            }
            Tween::Pause {
                duration: tween_duration,
                elapsed,
                ..
            } => {
                *elapsed += duration;
                if elapsed >= tween_duration {
                    let surplus = *elapsed - *tween_duration;
                    *elapsed = *tween_duration;
                    TweenProgress::Done { surplus }
                } else {
                    TweenProgress::Running
                }
            }
        }
    }

    pub fn advance<'a, ES: EventSender<E>>(
        &'a mut self,
        target: &'a mut T,
        event_sender: &'a mut ES,
        mut duration: Duration,
    ) -> TweenProgress {
        match self {
            Tween::Once {
                duration: tween_duration,
                elapsed,
                function,
                applier,
                completed_event,
            } => {
                *elapsed += duration;
                let result = if elapsed >= tween_duration {
                    let surplus = *elapsed - *tween_duration;
                    *elapsed = *tween_duration;
                    if let Some(e) = completed_event {
                        event_sender.send(e);
                    }
                    TweenProgress::Done { surplus }
                } else {
                    TweenProgress::Running
                };
                let v = function.interpolate(elapsed.as_secs_f32() / tween_duration.as_secs_f32());
                applier.apply(target, v);
                result
            }
            Tween::Repeat {
                tween,
                times,
                count,
                completed_event,
            } => loop {
                let done = match times {
                    RepeatTimes::N(amount) => count >= amount,
                    RepeatTimes::Infinite => false,
                };
                if done {
                    if let Some(e) = completed_event {
                        event_sender.send(e);
                    }
                    return TweenProgress::Done { surplus: duration };
                }
                let delegate_result = tween.advance(target, event_sender, duration);
                match delegate_result {
                    TweenProgress::Done { surplus } => {
                        *count += 1;
                        if duration <= surplus && *times == RepeatTimes::Infinite {
                            error!("Found infinite repeating tween with zero duration child (infinite loop)");
                            return TweenProgress::Running;
                        }
                        duration = surplus;
                        tween.reset();
                    }
                    TweenProgress::Running => {
                        break TweenProgress::Running;
                    }
                }
            },
            Tween::Sequence {
                index,
                tweens,
                completed_event,
            } => {
                while let Some(tween) = tweens.get_mut(*index) {
                    let delegate_result = tween.advance(target, event_sender, duration);
                    match delegate_result {
                        TweenProgress::Done { surplus } => {
                            *index += 1;
                            duration = surplus;
                        }
                        TweenProgress::Running => {
                            return TweenProgress::Running;
                        }
                    }
                }
                if let Some(e) = completed_event {
                    event_sender.send(e);
                }
                TweenProgress::Done { surplus: duration }
            }
            Tween::Parallel {
                tweens,
                completed_event,
            } => {
                let result = tweens.iter_mut().fold(
                    TweenProgress::Done { surplus: duration },
                    |acc, tween| {
                        let delegate_result = tween.advance(target, event_sender, duration);
                        if let (
                            TweenProgress::Done {
                                surplus: acc_surplus,
                            },
                            TweenProgress::Done {
                                surplus: delegate_surplus,
                            },
                        ) = (acc, delegate_result)
                        {
                            TweenProgress::Done {
                                surplus: acc_surplus.min(delegate_surplus),
                            }
                        } else {
                            TweenProgress::Running
                        }
                    },
                );
                if matches!(result, TweenProgress::Done { .. }) {
                    if let Some(e) = completed_event {
                        event_sender.send(e);
                    }
                }
                result
            }
            Tween::Pause {
                duration: tween_duration,
                elapsed,
                completed_event,
            } => {
                *elapsed += duration;
                if elapsed >= tween_duration {
                    let surplus = *elapsed - *tween_duration;
                    *elapsed = *tween_duration;
                    if let Some(e) = completed_event {
                        event_sender.send(e);
                    }
                    TweenProgress::Done { surplus }
                } else {
                    TweenProgress::Running
                }
            }
        }
    }

    fn reset(&mut self) {
        match self {
            Tween::Once { elapsed, .. } => {
                *elapsed = Duration::ZERO;
            }
            Tween::Repeat { tween, count, .. } => {
                *count = 0;
                tween.reset();
            }
            Tween::Sequence { index, tweens, .. } => {
                for tween in tweens.iter_mut().take(*index) {
                    tween.reset();
                }
                *index = 0;
            }
            Tween::Parallel { tweens, .. } => {
                for tween in tweens.iter_mut() {
                    tween.reset();
                }
            }
            Tween::Pause { elapsed, .. } => *elapsed = Duration::ZERO,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl TweenApplier<f32> for f32 {
        fn apply(&mut self, target: &mut f32, value: f32) {
            *target = *self * value * 2.0;
        }
    }

    #[test]
    fn tween_once() {
        let mut tween = Tween::new(Duration::from_secs(2), Lerp, 1.0_f32);
        tween.skip(Duration::from_millis(3));

        let mut value = 0.0;
        tween.advance(&mut value, &mut NoEvent, Duration::from_millis(1));

        let Tween::Once { elapsed, .. } = tween else {
            panic!()
        };
        assert_eq!(elapsed, Duration::from_millis(4));
        assert_eq!(value, 0.004);
    }

    #[test]
    fn tween_repeat() {
        let mut tween = Tween::repeat(
            RepeatTimes::N(2),
            Tween::new(Duration::from_secs(1), Lerp, 2.0_f32),
        );

        let mut value = 0.0;
        let progress = tween.advance(&mut value, &mut NoEvent, Duration::from_millis(1500));
        assert_eq!(progress, TweenProgress::Running);

        let Tween::Repeat {
            count,
            tween: ref delegate,
            ..
        } = tween
        else {
            panic!()
        };
        assert_eq!(count, 1);
        let Tween::Once { elapsed, .. } = **delegate else {
            panic!();
        };
        assert_eq!(elapsed, Duration::from_millis(500));
        assert_eq!(value, 2.0);

        let progress = tween.advance(&mut value, &mut NoEvent, Duration::from_millis(505));
        assert_eq!(
            progress,
            TweenProgress::Done {
                surplus: Duration::from_millis(5)
            }
        );
        let Tween::Repeat {
            count,
            tween: delegate,
            ..
        } = tween
        else {
            panic!()
        };
        assert_eq!(count, 2);
        let Tween::Once { elapsed, .. } = *delegate else {
            panic!();
        };
        assert_eq!(elapsed, Duration::from_millis(0));
        assert_eq!(value, 4.0);
    }

    #[test]
    fn tween_sequence() {
        let mut tween = Tween::sequence(vec![
            Tween::new(Duration::from_secs(1), Lerp, 2.0_f32),
            Tween::repeat(
                RepeatTimes::N(2),
                Tween::new(Duration::from_secs(1), Lerp, 4.0_f32),
            ),
        ]);

        let mut value = 0.0;
        let progress = tween.advance(&mut value, &mut NoEvent, Duration::from_millis(500));
        assert_eq!(progress, TweenProgress::Running);
        assert_eq!(value, 2.0);

        let progress = tween.advance(&mut value, &mut NoEvent, Duration::from_millis(1000));
        assert_eq!(progress, TweenProgress::Running);
        assert_eq!(value, 4.0);

        let progress = tween.advance(&mut value, &mut NoEvent, Duration::from_millis(2000));
        assert_eq!(
            progress,
            TweenProgress::Done {
                surplus: Duration::from_millis(500)
            }
        );
        assert_eq!(value, 8.0);
    }

    #[test]
    fn tween_sequence_with_endless_loop() {
        let mut tween = Tween::sequence(vec![
            Tween::new(Duration::from_secs(1), Lerp, 2.0_f32),
            Tween::repeat(
                RepeatTimes::Infinite,
                Tween::new(Duration::from_secs(1), Lerp, 5.0_f32),
            ),
        ]);

        let mut value = 0.0;
        let progress = tween.advance(&mut value, &mut NoEvent, Duration::from_millis(20000));
        assert_eq!(progress, TweenProgress::Running);
        assert_eq!(value, 0.0);
    }

    #[test]
    fn tween_parallel() {
        let mut tween = Tween::parallel(vec![
            Tween::new(Duration::from_secs(1), Lerp, 2.0_f32),
            Tween::repeat(
                RepeatTimes::N(2),
                Tween::new(Duration::from_secs(1), Lerp, 4.0_f32),
            ),
        ]);

        let mut value = 0.0;
        let progress = tween.advance(&mut value, &mut NoEvent, Duration::from_millis(1000));
        assert_eq!(progress, TweenProgress::Running);

        let progress = tween.advance(&mut value, &mut NoEvent, Duration::from_millis(1000));
        assert_eq!(
            progress,
            TweenProgress::Done {
                surplus: Duration::ZERO
            }
        );
    }
}
