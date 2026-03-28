use crate::constants::constants::INVALID;

pub struct SparseSet<C> {
    sparse: Vec<u32>, // Index: entity id , Value: dense index
    dense: Vec<C>,    // Index: entity index, Value: data
    entity: Vec<u32>, // Index: dense index , Value: entity id
}

impl<C> SparseSet<C> {
    pub fn new() -> Self {
        Self {
            sparse: Vec::new(),
            dense: Vec::new(),
            entity: Vec::new(),
        }
    }

    fn ensure_capacity(&mut self, entity_id: u32) {
        if entity_id >= self.sparse.len() as u32 {
            self.sparse.resize(entity_id as usize + 1, INVALID);
        }
    }

    pub fn insert(&mut self, entity_id: u32, component: C) {
        self.ensure_capacity(entity_id);

        let sparse_val = self.sparse[entity_id as usize];
        if sparse_val != INVALID {
            self.dense[sparse_val as usize] = component;
            return;
        }

        let dense_index = self.dense.len() as u32;
        self.entity.push(entity_id);
        self.dense.push(component);
        self.sparse[entity_id as usize] = dense_index;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_sparse_capacity() {
        let mut sparse_set = SparseSet::<u32>::new();
        sparse_set.ensure_capacity(10);
        assert_eq!(sparse_set.sparse.len(), 11);
    }

    #[test]
    fn ensure_sparse_invalid() {
        let mut sparse_set = SparseSet::<u32>::new();
        sparse_set.ensure_capacity(10);
        assert_eq!(sparse_set.sparse.len(), 11);
        for entry in sparse_set.sparse {
            assert_eq!(entry, INVALID);
        }
    }

    #[test]
    fn insert_entity() {
        let mut sparse_set = SparseSet::<u32>::new();
        // insert entity 0 with a u32 component type holding data 100
        sparse_set.insert(0, 100);
        // insert entity 10 with a u32 component type holding data 200
        sparse_set.insert(10, 200);
        // entity 0 should point to dense 0
        assert_eq!(sparse_set.sparse[0], 0);
        // entity 1 should point to no component making it an INVALID entity
        assert_eq!(sparse_set.sparse[1], INVALID);
        // entity 10 should point to dense 1 as all other entities are INVALID
        assert_eq!(sparse_set.sparse[10], 1);
        // dense 0 should hold data 100
        assert_eq!(sparse_set.dense[0], 100);
        // dense 1 should hold data 200
        assert_eq!(sparse_set.dense[1], 200);
        // ensure dense len
        assert_eq!(sparse_set.dense.len(), 2);
    }
}
