use uuid::Uuid;

use crate::ecs::{
    Entity, World,
    components::{Animator, Sprite},
};
use crate::reflecton::asset_types::SpriteAsset;

struct AnimationUpdate {
    entity: Entity,
    clamped_current: usize,
    new_frame: f32,
    still_playing: bool,
    sprite_id: Uuid,
    save_original: Option<Uuid>,
    restore_original: Option<Uuid>,
}

#[derive(Default)]
pub struct AnimationSystem;

impl AnimationSystem {
    pub fn update(world: &mut World, delta: f32) {
        let entities = world.query_with::<Animator>();

        let restore: Vec<(Entity, Uuid)> = entities
            .iter()
            .filter_map(|&entity| {
                let anim = world.get_component::<Animator>(entity)?;
                if anim.playing {
                    return None;
                }
                let orig = anim.original_sprite?;
                Some((entity, orig))
            })
            .collect();
        for (entity, orig) in restore {
            if let Some(anim) = world.get_component_mut::<Animator>(entity) {
                anim.original_sprite = None;
            }
            if let Some(sprite) = world.get_component_mut::<Sprite>(entity) {
                sprite.asset = SpriteAsset(orig);
            }
        }

        let updates: Vec<AnimationUpdate> = entities
            .iter()
            .filter_map(|&entity| {
                let anim = world.get_component::<Animator>(entity)?;
                if !anim.playing || anim.sprite_ids.is_empty() {
                    return None;
                }
                let cur = anim.current;
                let clamped_current = if cur >= anim.sprite_ids.len() {
                    let c = anim.sprite_ids.len() - 1;
                    log::warn!(
                        "Animator current={cur} out of bounds for {} clips, clamping to {c}",
                        anim.sprite_ids.len()
                    );
                    c
                } else {
                    cur
                };
                let cur = clamped_current;
                let count = *anim.frame_counts.get(cur)? as f32;
                let fps = *anim.fps.get(cur)?;
                let looping = *anim.looping.get(cur)?;
                let sprite_id = *anim.sprite_ids.get(cur)?;
                if sprite_id.is_nil() {
                    log::warn!("Animator clip {cur} has no sprite assigned");
                    return None;
                }

                let save_original = if anim.original_sprite.is_none() {
                    world.get_component::<Sprite>(entity).map(|s| s.asset.0)
                } else {
                    None
                };

                let mut new_frame = anim.frame + fps * delta;
                let mut still_playing = true;
                if fps >= 0.0 {
                    if new_frame >= count {
                        if looping {
                            new_frame %= count;
                        } else {
                            new_frame = count - 1.0;
                            still_playing = false;
                        }
                    }
                } else {
                    // Reverse playback: frame decrements toward 0.
                    if new_frame < 0.0 {
                        if looping {
                            new_frame = (new_frame % count + count) % count;
                        } else {
                            new_frame = 0.0;
                            still_playing = false;
                        }
                    }
                }

                let restore_original = if !still_playing {
                    anim.original_sprite.or(save_original)
                } else {
                    None
                };

                Some(AnimationUpdate {
                    entity,
                    clamped_current,
                    new_frame,
                    still_playing,
                    sprite_id,
                    save_original,
                    restore_original,
                })
            })
            .collect();

        for u in updates {
            if let Some(anim) = world.get_component_mut::<Animator>(u.entity) {
                anim.current = u.clamped_current;
                anim.frame = u.new_frame;
                anim.playing = u.still_playing;
                if u.still_playing {
                    if let Some(orig) = u.save_original {
                        anim.original_sprite = Some(orig);
                    }
                } else {
                    anim.original_sprite = None;
                }
            }
            if let Some(sprite) = world.get_component_mut::<Sprite>(u.entity) {
                if u.still_playing {
                    sprite.asset = SpriteAsset(u.sprite_id);
                } else if let Some(orig) = u.restore_original {
                    sprite.asset = SpriteAsset(orig);
                }
            }
        }
    }
}
