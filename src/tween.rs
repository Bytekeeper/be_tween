use interpolation::{Ease, EaseFunction};
use std::time::Duration;

pub trait TweenApplier<T> {
    fn apply(&mut self, target: &mut T, value: f32);
}

pub trait Interpolator {
    fn interpolate(&self, position: f32) -> f32;
}

pub struct Lerp;

pub enum Tween<T> {
    Once {
        duration: Duration,
        elapsed: Duration,
        function: Box<dyn Interpolator + Sync + Send + 'static>,
        applier: Box<dyn TweenApplier<T> + Sync + Send + 'static>,
    },
    Repeat {
        tween: Box<Tween<T>>,
        times: RepeatTimes,
        count: usize,
    },
    Sequence {
        index: usize,
        tweens: Vec<Tween<T>>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TweenProgress {
    Running,
    Done { surplus: Duration },
}

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

impl<T> Tween<T> {
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
        }
    }

    pub fn repeat(times: RepeatTimes, tween: Tween<T>) -> Self {
        Self::Repeat {
            times,
            count: 0,
            tween: Box::new(tween),
        }
    }

    pub fn sequence(tweens: impl Into<Vec<Tween<T>>>) -> Self {
        Self::Sequence {
            index: 0,
            tweens: tweens.into(),
        }
    }

    pub fn advance(&mut self, target: &mut T, mut duration: Duration) -> TweenProgress {
        match self {
            Tween::Once {
                duration: tween_duration,
                elapsed,
                function,
                applier,
            } => {
                *elapsed += duration;
                let result = if elapsed >= tween_duration {
                    let surplus = *elapsed - *tween_duration;
                    *elapsed = *tween_duration;
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
            } => {
                while !duration.is_zero() {
                    let done = match times {
                        RepeatTimes::N(amount) => count >= amount,
                        RepeatTimes::Infinite => false,
                    };
                    if done {
                        return TweenProgress::Done { surplus: duration };
                    }
                    let delegate_result = tween.advance(target, duration);
                    match delegate_result {
                        TweenProgress::Done { surplus } => {
                            *count += 1;
                            duration = surplus;
                            tween.reset();
                        }
                        TweenProgress::Running => {
                            break;
                        }
                    }
                }
                TweenProgress::Running
            }
            Tween::Sequence { index, tweens } => {
                while let Some(tween) = tweens.get_mut(*index) {
                    let delegate_result = tween.advance(target, duration);
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
        }
    }

    fn reset(&mut self) {
        match self {
            Tween::Once {
                elapsed,
                //ref mut value,
                //function,
                ..
            } => {
                *elapsed = Duration::ZERO;
                //value.apply(function.interpolate(0.0));
            }
            Tween::Repeat { tween, count, .. } => {
                *count = 0;
                tween.reset();
            }
            Tween::Sequence { index, tweens } => {
                for tween in tweens.iter_mut().take(*index) {
                    tween.reset();
                }
                *index = 0;
            }
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
        let mut tween = Tween::Once {
            duration: Duration::from_secs(2),
            elapsed: Duration::from_millis(3),
            function: Box::new(Lerp),
            applier: Box::new(1.0_f32),
        };

        let mut value = 0.0;
        tween.advance(&mut value, Duration::from_millis(1));

        let Tween::Once { elapsed, .. } = tween else {
            panic!()
        };
        assert_eq!(elapsed, Duration::from_millis(4));
        assert_eq!(value, 0.004);
    }

    #[test]
    fn tween_repeat() {
        let mut tween = Tween::Repeat {
            tween: Box::new(Tween::Once {
                duration: Duration::from_secs(1),
                elapsed: Duration::from_millis(0),
                function: Box::new(Lerp),
                applier: Box::new(2.0_f32),
            }),
            count: 0,
            times: RepeatTimes::N(2),
        };

        let mut value = 0.0;
        let progress = tween.advance(&mut value, Duration::from_millis(1500));
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

        let progress = tween.advance(&mut value, Duration::from_millis(505));
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
        let mut tween = Tween::Sequence {
            index: 0,
            tweens: vec![
                Tween::Once {
                    duration: Duration::from_secs(1),
                    elapsed: Duration::from_millis(0),
                    function: Box::new(Lerp),
                    applier: Box::new(2.0_f32),
                },
                Tween::Repeat {
                    tween: Box::new(Tween::Once {
                        duration: Duration::from_secs(1),
                        elapsed: Duration::from_millis(0),
                        function: Box::new(Lerp),
                        applier: Box::new(4.0_f32),
                    }),
                    count: 0,
                    times: RepeatTimes::N(2),
                },
            ],
        };

        let mut value = 0.0;
        let progress = tween.advance(&mut value, Duration::from_millis(500));
        assert_eq!(progress, TweenProgress::Running);
        assert_eq!(value, 2.0);

        let progress = tween.advance(&mut value, Duration::from_millis(1000));
        assert_eq!(progress, TweenProgress::Running);
        assert_eq!(value, 4.0);

        let progress = tween.advance(&mut value, Duration::from_millis(2000));
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
        let progress = tween.advance(&mut value, Duration::from_millis(20000));
        assert_eq!(progress, TweenProgress::Running);
        assert_eq!(value, 10.0);
    }
}
