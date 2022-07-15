//! Generational index

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
    pub fn new(v: I) -> Self {
        let mut id = Self { index: 0, gen: 0 };
        for i in 0..usize::BITS {
            id.index |= ((v & 1u128 << (2 * i)) >> i) as usize;
            id.gen |= ((v & 1u128 << (2 * i + 1)) >> (i + 1)) as usize;
        }
        id
    }
    pub fn pack(&self) -> I {
        let mut v = 0;
        let a = self.index as I;
        let b = self.gen as I;
        for i in 0..usize::BITS {
            v |= (a & 1u128 << i) << i;
            v |= (b & 1u128 << i) << (i + 1);
        }
        v
    }
}
pub type I = u128;
impl From<Id> for I {
    fn from(id: Id) -> Self {
        id.pack()
    }
}
impl From<I> for Id {
    fn from(v: I) -> Self {
        Self::new(v)
    }
}

#[derive(Debug)]
pub struct Array<K, V> {
    vals: Vec<Option<V>>,
    gens: Vec<usize>,
    free: VecDeque<usize>,
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

        K::from(Id { index, gen: self.gens[index] })
    }

    pub fn remove(&mut self, key: K) -> Result<V, NotFound> {
        let id = self.check(key)?;
        let v = self.vals[id.index].take().unwrap();
        self.gens[id.index] += 1;
        self.free.push_back(id.index);
        Ok(v)
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
        self.len() == 0
    }
    pub fn len(&self) -> usize {
        self.vals.len() - self.free.len()
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
    /// To use with split_at_mut
    pub fn iter_index(&self) -> std::ops::Range<usize> {
        0..self.vals.len()
    }
    /// Cannot implement .iter_mut_split() because of double borrow
    pub fn split_at_mut(&mut self, index: usize) -> Option<(K, &mut V, MutSplit<K, V>)> {
        let (left, v_right) = self.vals.split_at_mut(index);
        let (opt, right) = v_right.split_at_mut(1);
        opt[0].as_mut().map(|val| {
            (
                K::from(Id { index, gen: self.gens[index] }),
                val,
                MutSplit { gens: &self.gens[..], left, right, _marker: PhantomData },
            )
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
impl<K, V> std::ops::Index<K> for Array<K, V>
where
    K: Into<Id> + From<Id>,
{
    type Output = V;

    #[inline]
    fn index(&self, index: K) -> &Self::Output {
        self.get(index).expect("Index out of bound")
    }
}
impl<K, V> std::ops::IndexMut<K> for Array<K, V>
where
    K: Into<Id> + From<Id>,
{
    #[inline]
    fn index_mut(&mut self, index: K) -> &mut Self::Output {
        self.get_mut(index).expect("Index out of bound")
    }
}

pub struct MutSplit<'a, K, V> {
    gens: &'a [usize],
    left: &'a mut [Option<V>],
    right: &'a mut [Option<V>],
    _marker: PhantomData<fn(K) -> K>,
}
impl<K, V> MutSplit<'_, K, V>
where
    K: Into<Id>,
{
    pub fn get(&self, key: K) -> Result<&V, NotFound> {
        let id = self.part_check(key)?;
        if id.index < self.left.len() {
            &self.left[id.index]
        } else {
            &self.right[id.index - self.left.len() - 1]
        }
        .as_ref()
        .ok_or(NotFound::Deleted)
    }
    pub fn get_mut(&mut self, key: K) -> Result<&mut V, NotFound> {
        let id = self.part_check(key)?;
        if id.index < self.left.len() {
            &mut self.left[id.index]
        } else {
            &mut self.right[id.index - self.left.len() - 1]
        }
        .as_mut()
        .ok_or(NotFound::Deleted)
    }

    fn part_check(&self, key: K) -> Result<Id, NotFound> {
        let id = key.into();
        if id.index >= self.gens.len() || id.index == self.left.len() {
            return Err(NotFound::OutOfBounds);
        }
        if id.gen != self.gens[id.index] {
            return Err(NotFound::OutDated);
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
