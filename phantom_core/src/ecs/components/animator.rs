use uuid::Uuid;

use crate::ecs::AnyStorage;
use crate::ecs::Entity;
use crate::ecs::SparseSet;
use crate::ecs::World;
use crate::ecs::component::Component;
use crate::ecs::component_registry::ComponentEntry;
use crate::reflecton::Reflection;
use crate::reflecton::fields::Field;

/// Drives sprite-sheet animation for an entity.
///
/// An `Animator` holds one or more *clips*, each identified by a sequential
/// index. A clip is a slice of a sprite-sheet asset described by:
///
/// - `sprite_ids` — the sprite-sheet asset for that clip
/// - `frame_widths` / `frame_heights` — pixel size of a single frame
/// - `frame_counts` — total number of frames in the clip
/// - `fps` — playback speed; **negative values play the clip in reverse**
/// - `looping` — whether the clip wraps or stops at the boundary
///
/// Use [`play`](Self::play) / [`stop`](Self::stop) to control playback.
/// [`AnimationSystem`](crate::animation::AnimationSystem) advances `frame`
/// each tick and writes the correct UV region back to the entity's `Sprite`.
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Animator {
    pub sprite_ids: Vec<Uuid>,
    pub frame_widths: Vec<u32>,
    pub frame_heights: Vec<u32>,
    pub frame_counts: Vec<u32>,
    /// Frames per second for each clip. Negative values reverse playback.
    pub fps: Vec<f32>,
    pub looping: Vec<bool>,
    pub current: usize,
    #[serde(skip)]
    pub frame: f32,
    pub playing: bool,
    /// Sprite the entity had before playback started, restored on stop.
    #[serde(skip)]
    pub original_sprite: Option<Uuid>,
}

impl Default for Animator {
    fn default() -> Self {
        Self {
            sprite_ids: Vec::new(),
            frame_widths: Vec::new(),
            frame_heights: Vec::new(),
            frame_counts: Vec::new(),
            fps: Vec::new(),
            looping: Vec::new(),
            current: 0,
            frame: 0.0,
            playing: false,
            original_sprite: None,
        }
    }
}

impl Animator {
    /// Switch to `clip` and begin playback.
    ///
    /// Resets the frame only when switching to a different clip or resuming
    /// from stopped. Calling `play` on the already-playing clip is a no-op
    /// on the frame position, so it is safe to call every frame from a script.
    ///
    /// Clips with negative `fps` start at the last frame so playback runs in
    /// reverse. Does nothing if `clip` is out of range.
    pub fn play(&mut self, clip: usize) {
        if clip >= self.sprite_ids.len() {
            return;
        }
        let reset = !self.playing || self.current != clip;
        self.current = clip;
        if reset {
            self.frame = if self.fps.get(clip).copied().unwrap_or(0.0) < 0.0 {
                self.frame_counts.get(clip).copied().unwrap_or(1) as f32 - 1.0
            } else {
                0.0
            };
        }
        self.playing = true;
    }

    /// Stop the current animation. `AnimationSystem` will restore the
    /// pre-animation sprite on the next frame.
    pub fn stop(&mut self) {
        self.playing = false;
    }

    /// Set the playback speed of `clip` in frames per second.
    ///
    /// Negative values reverse playback direction. If `clip` is out of range
    /// or the animator has no clips yet, this is a no-op.
    pub fn set_fps(&mut self, clip: usize, fps: f32) {
        if let Some(entry) = self.fps.get_mut(clip) {
            *entry = fps;
        }
    }

    /// Whether `clip` is set to loop. Returns `false` if `clip` is out of range.
    pub fn is_looping(&self, clip: usize) -> bool {
        self.looping.get(clip).copied().unwrap_or(false)
    }

    /// Set whether `clip` loops. No-op if `clip` is out of range.
    pub fn set_looping(&mut self, clip: usize, looping: bool) {
        if let Some(entry) = self.looping.get_mut(clip) {
            *entry = looping;
        }
    }

    /// Index of the currently active clip.
    pub fn current_clip(&self) -> usize {
        self.current
    }

    /// Current fractional frame within the active clip.
    pub fn current_frame(&self) -> f32 {
        self.frame
    }

    /// Whether a clip is currently playing.
    pub fn is_playing(&self) -> bool {
        self.playing
    }

    /// Number of clips registered on this animator.
    pub fn clip_count(&self) -> usize {
        self.sprite_ids.len()
    }
}

impl Reflection for Animator {
    fn get_fields(&self) -> Vec<Field> {
        vec![]
    }
    fn set_feilds(&mut self, _fields: Vec<Field>) {}
}

impl Component for Animator {
    const NAME: &'static str = "Animator";
}

#[::ctor::ctor]
fn __register_animator() {
    crate::ecs::component_registry::register_component(
        "Animator",
        ComponentEntry(
            __deserialize_animator,
            __add_default_animator,
            __remove_animator,
            true,
        ),
    );
}

fn __deserialize_animator(data: &[u8]) -> Box<dyn AnyStorage> {
    Box::new(serde_json::from_slice::<SparseSet<Animator>>(data).unwrap())
}

fn __add_default_animator(entity: Entity) -> Box<dyn FnOnce(&mut World)> {
    Box::new(move |world| {
        world.add_component(entity, Animator::default());
    })
}

fn __remove_animator(world: &mut World, entity: Entity) {
    world.remove_component::<Animator>(entity);
}
