[![Build](https://github.com/Bytekeeper/be_tween/actions/workflows/rust.yml/badge.svg)](https://github.com/Bytekeeper/be_tween/actions/workflows/rust.yml)
[![crates.io](https://img.shields.io/crates/v/be_tween.svg)](https://crates.io/crates/be_tween)
[![docs.rs](https://img.shields.io/docsrs/be_tween)](https://docs.rs/be_tween/)


# Be-Tween 
This will allow you to write rather complex tween sequences. 
Although Bevy is one of the main targets, this library will work just fine without it.

## Q&A
## Can I use it?
Yes, but I recommend using a more mature library like [bevy_tweening](https://crates.io/crates/bevy_tweening).

## Why did you write it?
I used bevy_tweening at first. But it has some limits:
* Sequences ending with a endlessly looping tween are not supported.
* Repeating complex sequences of tweens is not supported
I did at first try to tweak bevy_tweening. But in the end decided to write my own library - it's fun and more variety in choice is a good thing.

## Version

| bevy | be_tween |
| ---- | -------- |
| 0.14 | 0.5 |
| 0.13 | 0.4 |
