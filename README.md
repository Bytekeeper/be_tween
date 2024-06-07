# Be-Tween 
This will allow you to write rather complex tween sequences. 
It should work well with "Bevy". The current supported version is Bevy 0.13.

## Q&A
## Can I use it?
Yes, but I recommend using a more mature library like [bevy_tweening](https://crates.io/crates/bevy_tweening).

## Why did you write it?
I used bevy_tweening at first. But it has some limits:
* Sequences ending with a endlessly looping tween are not supported.
* Repeating complex sequences of tweens is not supported
I did at first try to tweak bevy_tweening. But in the end decided to write my own library - it's fun and more variety in choice is a good thing.
