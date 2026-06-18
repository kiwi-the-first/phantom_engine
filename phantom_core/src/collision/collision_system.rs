use glam::Vec2;

use crate::ecs::{
    World,
    components::{Collider, Transform},
};

#[derive(Default)]
pub struct CollisionSystem;

impl CollisionSystem {
    pub fn update(world: &mut World) {
        let entities = world.query_with2::<Collider, Transform>();

        for &entity in &entities {
            if let Some(col) = world.get_component_mut::<Collider>(entity) {
                col.colliding_with.clear();
            }
        }

        let snapshots: Vec<_> = entities
            .iter()
            .map(|&entity| {
                let col = world.get_component::<Collider>(entity).unwrap();
                let transform = world.get_component::<Transform>(entity).unwrap();
                let pos = transform.position.truncate() + col.offset;
                let half = Vec2::new(
                    col.width * transform.scale.x / 2.0,
                    col.height * transform.scale.y / 2.0,
                );
                (entity, pos, half)
            })
            .collect();

        for i in 0..snapshots.len() {
            for j in (i + 1)..snapshots.len() {
                let (ea, pa, ha) = snapshots[i];
                let (eb, pb, hb) = snapshots[j];
                if aabb_overlap(pa, ha, pb, hb) {
                    world.get_component_mut::<Collider>(ea).unwrap().colliding_with.push(eb);
                    world.get_component_mut::<Collider>(eb).unwrap().colliding_with.push(ea);
                }
            }
        }
    }
}

fn aabb_overlap(pa: Vec2, ha: Vec2, pb: Vec2, hb: Vec2) -> bool {
    (pa.x - pb.x).abs() < ha.x + hb.x && (pa.y - pb.y).abs() < ha.y + hb.y
}
