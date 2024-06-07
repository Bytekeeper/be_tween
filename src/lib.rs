//! Be-Tween
//!
//! Provides tweening values over time.

//! These examples run with Bevy:
//! ```no_run
//! use bevy::prelude::*;
//! use be_tween::*;
//!
//! App::default()
//!     .add_plugins(DefaultPlugins)
//!     // Add the plugin - NoEvent means that no custom events will be used
//!     .add_plugins(TweenPlugin::<NoEvent>::new())
//!     // Add the systems performing the tweening
//!     .add_systems(Update, (play_tween_animation::<Transform, NoEvent>, play_tween_animation::<Sprite, NoEvent>))
//!     .run();
//! ```
//!
//! Animate the position ([`Transform::translation`]) of an [`Entity`]:
//! ```
//! # use bevy::prelude::*;
//! # use be_tween::*;
//! # use std::time::Duration;
//! # fn system(mut commands: Commands) {
//! // Create a single animation (tween) to move an entity.
//! let tween = Tween::new(
//!     // Animation time.
//!     Duration::from_secs(1),
//!     // Use a quadratic easing on both endpoints.
//!     EaseFunction::QuadraticInOut,
//!     // What we want to tween - the translation part of a transform
//!     TweenTranslation {
//!         start: Vec3::ZERO,
//!         end: Vec3::X,
//!     },
//! );
//!
//! commands.spawn((
//!     // Spawn an entity to animate the position of.
//!     TransformBundle::default(),
//!     // Add an Animator component to control and execute the animation.
//!     PlayTween::new(tween),
//! ));
//! # }
//! ```
//! A more elaborate example:
//! ```
//! # use bevy::prelude::*;
//! # use be_tween::*;
//! # use std::time::Duration;
//! # fn system(mut commands: Commands) {
//! let tween_translation = Tween::sequence([
//!   // First move to Vec3::X
//!   Tween::new(
//!     Duration::from_secs(1),
//!     EaseFunction::QuadraticInOut,
//!     TweenTranslation {
//!         start: Vec3::ZERO,
//!         end: Vec3::X,
//!     }),
//!   // Then move from Vec3::X to Vec3::Y 12 times
//!   Tween::repeat(
//!     RepeatTimes::N(12),
//!     Tween::new(
//!      Duration::from_secs(1),
//!      EaseFunction::QuadraticInOut,
//!      TweenTranslation {
//!          start: Vec3::X,
//!          end: Vec3::Y,
//!      })),
//!   // Then move up and down forever
//!   Tween::repeat(
//!     RepeatTimes::Infinite,
//!     Tween::sequence([
//!       Tween::new(
//!         Duration::from_secs(1),
//!         EaseFunction::QuadraticInOut,
//!         TweenTranslation {
//!             start: Vec3::Y,
//!             end: -Vec3::Y,
//!         }),
//!       Tween::new(
//!         Duration::from_secs(1),
//!         EaseFunction::QuadraticInOut,
//!         TweenTranslation {
//!             start: -Vec3::Y,
//!             end: Vec3::Y,
//!         }),
//!     ]))
//!   ]);
//! let tween_color = Tween::sequence([Tween::new(
//!     Duration::from_secs(1),
//!     EaseFunction::QuadraticInOut,
//!     TweenSpriteColor {
//!         start: Color::WHITE,
//!         end: Color::RED
//!     })]
//! );
//!
//! commands.spawn((
//!     // Spawn an entity to animate the position of.
//!     TransformBundle::default(),
//!     // Now play the tweens
//!     PlayTween::new(tween_translation),
//!     // Note that both will be played in parallel!
//!     PlayTween::new(tween_color),
//! ));
//! # }
//! ```

#[cfg(feature = "bevy")]
mod plugin;
mod tween;

#[cfg(feature = "bevy")]
pub use plugin::*;
pub use tween::*;
