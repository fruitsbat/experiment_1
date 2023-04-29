use std::time::Duration;

use bevy::prelude::*;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(animate_sprites);
    }
}

/// give information about the frames of a sprite
pub enum Frames {
    /// count and speed
    Constant(usize, f32),
    /// just speed, count is the same as speed
    Variable(Vec<f32>),
}

#[derive(Component)]
pub struct SpriteAnimation {
    /// total number of frames
    pub frames: Frames,

    pub start_index: usize,

    pub looped: bool,

    current_index: usize,
    timer: Timer,
}

impl SpriteAnimation {
    pub fn new(frames: Frames, start_index: usize, looped: bool) -> Self {
        Self {
            current_index: 0,
            timer: Self::timer_for_frame(0, &frames),
            frames,
            looped,
            start_index,
        }
    }

    /// get a timer with correct length for a frame
    fn timer_for_frame(index: usize, frames: &Frames) -> Timer {
        Timer::from_seconds(
            match frames {
                Frames::Constant(_, duration) => duration.clone(),
                Frames::Variable(l) => l[index],
            },
            TimerMode::Once,
        )
    }

    pub fn frame_count(&self) -> usize {
        match &self.frames {
            Frames::Constant(len, _) => len.clone(),
            Frames::Variable(l) => l.len().clone(),
        }
    }

    pub fn index(&self) -> usize {
        self.current_index
    }

    pub fn advance_index(&mut self, delta: Duration) {
        if !self.looped && (self.frame_count() == (self.current_index + 1)) {
            return;
        }

        self.timer.tick(delta);
        if !self.timer.just_finished() {
            return;
        }

        // move animation forwards
        self.current_index = (self.current_index + 1).rem_euclid(self.frame_count());

        // set a new timer for the next frame
        self.timer = Self::timer_for_frame(self.index(), &self.frames);
    }
}

fn animate_sprites(
    time: Res<Time>,
    mut query: Query<(&mut SpriteAnimation, &mut TextureAtlasSprite)>,
) {
    for (mut anim, mut atlas) in query.iter_mut() {
        anim.advance_index(time.delta());
        atlas.index = anim.index() + anim.start_index
    }
}
