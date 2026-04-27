use crate::ecs::Entity;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct WorldData {
    pub next_entity_id: u32,
    pub deleted_entity_ids: Vec<Entity>,
    pub generations: Vec<u32>,
    pub components: Vec<(String, Vec<u8>)>, // (type_name, serialized_sparse_set)
}
