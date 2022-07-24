#![allow(dead_code)]
use std::{collections::VecDeque, marker::PhantomData};

#[derive(Debug, Clone, Copy)]
pub struct Id {
    index: usize,
    gen: usize,
}
impl Id {
    pub fn index(&self) -> usize {
        self.index
    }
}
pub type I = u128;
impl From<Id> for I {
    fn from(id: Id) -> Self {
        let mut v = 0;
        for i in 0..usize::BITS {
            v |= ((id.index as I) & (0xF << i)) << i;
            v |= ((id.gen as I) & (0xF << i)) << (i + 1);
        }
        v
    }
}
impl From<I> for Id {
    fn from(v: I) -> Self {
        let mut id = Self { index: 0, gen: 0 };
        for i in 0..usize::BITS {
            id.index |= ((v & (0xF << i)) >> (2 * i)) as usize;
            id.gen |= ((v & (0xF << i)) >> (2 * i + 1)) as usize;
        }
        id
    }
}

#[derive(Debug)]
pub struct Array<K, V> {
    vals: Vec<Option<V>>,
    gens: Vec<usize>,
    free: VecDeque<usize>,
    len: usize,
    _marker: PhantomData<fn(K) -> K>,
}
impl<K, V> Array<K, V>
where
    K: Into<Id> + From<Id>,
{
    pub fn new() -> Self {
        Self {
            vals: Vec::<Option<V>>::new(),
            gens: Vec::<usize>::new(),
            free: VecDeque::<usize>::new(),
            len: 0,
            _marker: PhantomData,
        }
    }

    pub fn insert(&mut self, value: V) -> K {
        let index = match self.free.pop_front() {
            Some(index) => {
                self.vals[index] = Some(value);
                index
            }
            None => {
                let index = self.vals.len();
                self.vals.push(Some(value));
                self.gens.push(0);
                index
            }
        };
        self.len += 1;

        K::from(Id { index, gen: self.gens[index] })
    }

    pub fn remove(&mut self, key: K) -> Result<(), NotFound> {
        let id = self.check(key)?;

        self.vals[id.index] = None;
        self.gens[id.index] += 1;

        self.free.push_back(id.index);
        self.len -= 1;

        Ok(())
    }

    pub fn get(&self, key: K) -> Result<&V, NotFound> {
        let id = self.check(key)?;
        Ok(self.vals[id.index].as_ref().unwrap())
    }
    pub fn get_mut(&mut self, key: K) -> Result<&mut V, NotFound> {
        let id = self.check(key)?;
        Ok(self.vals[id.index].as_mut().unwrap())
    }

    pub fn exists(&self, key: K) -> bool {
        self.check(key).is_ok()
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn capacity(&self) -> usize {
        self.vals.capacity()
    }

    pub fn iter(&self) -> impl Iterator<Item = (K, &V)> {
        self.vals.iter().zip(self.gens.iter().enumerate()).filter_map(|(opt, (index, gen))| {
            opt.as_ref().map(|val| (K::from(Id { index, gen: *gen }), val))
        })
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (K, &mut V)> {
        self.vals.iter_mut().zip(self.gens.iter().enumerate()).filter_map(|(opt, (index, gen))| {
            opt.as_mut().map(|val| (K::from(Id { index, gen: *gen }), val))
        })
    }

    fn check(&self, key: K) -> Result<Id, NotFound> {
        let id = key.into();
        if id.index >= self.gens.len() {
            return Err(NotFound::OutOfBounds);
        }
        if id.gen != self.gens[id.index] {
            return Err(NotFound::OutDated);
        }
        if self.vals[id.index].is_none() {
            return Err(NotFound::Deleted);
        }
        Ok(id)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NotFound {
    Deleted,
    OutDated,
    OutOfBounds,
}
