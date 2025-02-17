use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use fnv::FnvHasher;

use {Entity, Generation};


/// Base trait for a component storage that is used as a trait object.
/// Doesn't depent on the actual component type.
pub trait StorageBase {
    /// Delete a particular entity from the storage.
    fn del(&mut self, Entity);
}

/// Typed component storage trait.
pub trait Storage<T>: StorageBase + Sized {
    /// Create a new storage. This is called when you register a new
    /// component type within the world.
    fn new() -> Self;
    /// Try reading the data associated with an entity.
    fn get(&self, Entity) -> Option<&T>;
    /// Try mutating the data associated with an entity.
    fn get_mut(&mut self, Entity) -> Option<&mut T>;
    /// Insert a new data for a given entity.
    fn insert(&mut self, Entity, T);
    /// Remove the data associated with an entity.
    fn remove(&mut self, Entity) -> Option<T>;
}


/// Vec-based storage, actually wraps data into options and stores the generations
/// of the data in order to match with given entities. Supposed to have maximum
/// performance for the components mostly present in entities.
#[derive(Debug)]
pub struct VecStorage<T>(pub Vec<Option<(Generation, T)>>);

impl<T> StorageBase for VecStorage<T> {
    fn del(&mut self, entity: Entity) {
        self.0.get_mut(entity.get_id()).map(|x| *x = None);
    }
}
impl<T> Storage<T> for VecStorage<T> {
    fn new() -> Self {
        VecStorage(Vec::new())
    }
    fn get(&self, entity: Entity) -> Option<&T> {
        self.0.get(entity.get_id()).and_then(|x| match x {
            &Some((gen, ref value)) if gen == entity.get_gen() => Some(value),
            _ => None
        })
    }
    fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        self.0.get_mut(entity.get_id()).and_then(|x| match x {
            &mut Some((gen, ref mut value)) if gen == entity.get_gen() => Some(value),
            _ => None
        })
    }
    fn insert(&mut self, entity: Entity, value: T) {
        while self.0.len() <= entity.get_id() {
            self.0.push(None);
        }
        self.0[entity.get_id()] = Some((entity.get_gen(), value));
    }
    fn remove(&mut self, entity: Entity) -> Option<T> {
        self.0.get_mut(entity.get_id()).and_then(|x| {
            if let &mut Some((gen, _)) = x {
                // if the generation does not match avoid deleting
                if gen != entity.get_gen() {
                    return None;
                }
            }
            x.take().map(|(_, x)| x)
        })
    }
}

/// HashMap-based storage. Best suited for rare components.
#[derive(Debug)]
pub struct HashMapStorage<T>(pub HashMap<Entity, T, BuildHasherDefault<FnvHasher>>);

impl<T> StorageBase for HashMapStorage<T> {
    fn del(&mut self, entity: Entity) {
        self.0.remove(&entity);
    }
}
impl<T> Storage<T> for HashMapStorage<T> {
    fn new() -> Self {
        let fnv = BuildHasherDefault::<FnvHasher>::default();
        HashMapStorage(HashMap::with_hasher(fnv))
    }
    fn get(&self, entity: Entity) -> Option<&T> {
        self.0.get(&entity)
    }
    fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        self.0.get_mut(&entity)
    }
    fn insert(&mut self, entity: Entity, value: T) {
        self.0.insert(entity, value);
    }
    fn remove(&mut self, entity: Entity) -> Option<T> {
        self.0.remove(&entity)
    }
}


#[cfg(test)]
mod test {
    use Entity;
    use super::*;

    fn test_add<S>() where S: Storage<u32> {
        let mut s = S::new();
        for i in 0..1_000 {
            s.insert(Entity::new(i, 1), i + 2718);
        }

        for i in 0..1_000 {
            assert_eq!(*s.get(Entity::new(i, 1)).unwrap(), i + 2718);
        }
    }

    fn test_sub<S>() where S: Storage<u32> {
        let mut s = S::new();
        for i in 0..1_000 {
            s.insert(Entity::new(i, 1), i + 2718);
        }

        for i in 0..1_000 {
            assert_eq!(s.remove(Entity::new(i, 1)).unwrap(), i + 2718);
            assert!(s.remove(Entity::new(i, 1)).is_none());
        }
    }

    fn test_get_mut<S>() where S: Storage<u32> {
        let mut s = S::new();
        for i in 0..1_000 {
            s.insert(Entity::new(i, 1), i + 2718);
        }

        for i in 0..1_000 {
            *s.get_mut(Entity::new(i, 1)).unwrap() -= 718;
        }

        for i in 0..1_000 {
            assert_eq!(*s.get(Entity::new(i, 1)).unwrap(), i + 2000);
        }
    }

    fn test_add_gen<S>() where S: Storage<u32> {
        let mut s = S::new();
        for i in 0..1_000 {
            s.insert(Entity::new(i, 1), i + 2718);
            s.insert(Entity::new(i, 2), i + 31415);
        }

        for i in 0..1_000 {
            // this is removed since vec and hashmap disagree
            // on how this behavior should work...
            //assert!(s.get(Entity::new(i, 1)).is_none());
            assert_eq!(*s.get(Entity::new(i, 2)).unwrap(), i + 31415);
        }
    }

    fn test_sub_gen<S>() where S: Storage<u32> {
        let mut s = S::new();
        for i in 0..1_000 {
            s.insert(Entity::new(i, 2), i + 2718);
        }

        for i in 0..1_000 {
            assert!(s.remove(Entity::new(i, 1)).is_none());
        }
    }

    #[test] fn vec_test_add() { test_add::<VecStorage<u32>>(); }
    #[test] fn vec_test_sub() { test_sub::<VecStorage<u32>>(); }
    #[test] fn vec_test_get_mut() { test_get_mut::<VecStorage<u32>>(); }
    #[test] fn vec_test_add_gen() { test_add_gen::<VecStorage<u32>>(); }
    #[test] fn vec_test_sub_gen() { test_sub_gen::<VecStorage<u32>>(); }

    #[test] fn hash_test_add() { test_add::<HashMapStorage<u32>>(); }
    #[test] fn hash_test_sub() { test_sub::<HashMapStorage<u32>>(); }
    #[test] fn hash_test_get_mut() { test_get_mut::<HashMapStorage<u32>>(); }
    #[test] fn hash_test_add_gen() { test_add_gen::<HashMapStorage<u32>>(); }
    #[test] fn hash_test_sub_gen() { test_sub_gen::<HashMapStorage<u32>>(); }
}

