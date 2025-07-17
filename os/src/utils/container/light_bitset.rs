
use core::marker::PhantomData;

use num_traits::{PrimInt, Unsigned};

use log::error;

#[derive(Debug, Clone)]
pub struct LightBitSet<T>(T)
where  
    T: Unsigned + PrimInt;

impl<T> LightBitSet<T> 
where  
    T: Unsigned + PrimInt {
    pub fn new() -> Self {
        LightBitSet(T::zero())
    }
    pub fn clear(&mut self) -> T {
        let ret = self.0;
        self.0 = T::zero();
        ret
    }
    pub fn insert(&mut self, index: usize) -> Result<bool, ()> {
        if index > size_of::<T>() * 8 {
            return Err(());
        }
        let index_bit = T::one() << index;
        if self.0 & index_bit == T::zero() {
            self.0 = self.0 | index_bit;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    pub fn remove(&mut self, index: usize) -> Result<bool, ()> {
        if index > size_of::<T>() * 8 {
            return Err(());
        }
        let index_bit = T::one() << index;
        if self.0 & index_bit != T::zero() {
            self.0 = self.0 & !index_bit;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn len(&self) -> usize {
        self.0.count_ones() as usize
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_zero()
    }

    pub fn is_full(&self) -> bool {
        self.0 == T::max_value()
    }

    pub fn contains(&self, index: usize) -> Result<bool, ()> {
        if index > size_of::<T>() * 8 {
            return Err(());
        }
        let index_bit = T::one() << index;
        Ok(self.0 & index_bit != T::zero())
    }

    pub fn min(&self) -> Option<usize> {
        (!self.0.is_zero()).then_some(
            self.0.trailing_zeros() as usize
        )
    }
    pub fn max(&self) -> Option<usize> {
        (!self.0.is_zero()).then_some(
            size_of::<T>() * 8 - self.0.leading_zeros() as usize - 1
        )
    }

    pub fn iter(&self) -> impl Iterator<Item = usize> {
        self.into_iter()
    }
}

pub struct LightBitIterator<T> 
where  
    T: Unsigned + PrimInt {
    bits: T,
}

impl<T> Iterator for LightBitIterator<T> 
where 
    T: Unsigned + PrimInt {
    type Item = usize;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.bits.is_zero() {
            None
        }
        else {
            let index = self.bits.trailing_zeros() as usize;
            self.bits = self.bits & !(T::one() << index);
            Some(index)
        }
    } 
}

impl<T> IntoIterator for LightBitSet<T> 
where  
    T: Unsigned + PrimInt {
    type Item = usize;
    type IntoIter = LightBitIterator<T>;
    fn into_iter(self) -> Self::IntoIter {
        LightBitIterator { bits: self.0 }
    }
}

impl<T> IntoIterator for &LightBitSet<T> 
where  
    T: Unsigned + PrimInt {
    type Item = usize;
    type IntoIter = LightBitIterator<T>;
    fn into_iter(self) -> Self::IntoIter {
        LightBitIterator { bits: self.0 }
    }
}

pub type BitSet8 = LightBitSet<u8>;
pub type BitSet16 = LightBitSet<u16>;
pub type BitSet32 = LightBitSet<u32>;
pub type BitSet64 = LightBitSet<u64>;
pub type BitSet128 = LightBitSet<u128>;