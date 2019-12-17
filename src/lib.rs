#![no_std]
#![cfg_attr(feature = "unstable", feature(core_intrinsics, const_fn))]

use const_default::ConstDefault;
use num_traits::{CheckedAdd, Saturating};
use core::ops::{Add, Sub};

mod unit;

pub use unit::*;

macro_rules! impl_wrapper {
    ($id:ident) => {
        #[derive(Copy, Clone, Default, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
        pub struct $id<T>(T);

        impl<T: ConstDefault> ConstDefault for $id<T> {
            const DEFAULT: Self = $id(T::DEFAULT);
        }

        impl<T> $id<T> {
            #[inline]
            pub const fn new(value: T) -> Self {
                Self(value)
            }

            #[inline]
            pub fn value(&self) -> &T {
                &self.0
            }

            #[inline]
            pub fn value_mut(&mut self) -> &mut T {
                &mut self.0
            }
        }

        impl<T: Unit> From<T> for $id<T> {
            #[inline]
            fn from(value: T) -> Self {
                Self::new(value)
            }
        }
    };
}

impl_wrapper! { Duration }
impl_wrapper! { Instant }

#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum Moment<T> {
    Relative(Duration<T>),
    Absolute(Instant<T>),
}

#[cfg(feature = "unstable")]
impl<T: ConstDefault> Moment<T> {
    #[inline]
    pub const fn immediate_const() -> Self {
        Moment::Relative(ConstDefault::DEFAULT)
    }
}

impl<T: Default> Moment<T> {
    #[inline]
    pub fn immediate() -> Self {
        Moment::Relative(Default::default())
    }
}

impl<T: Unit + CheckedAdd + Default> Moment<T> {
    #[inline]
    pub fn to_absolute<F: FnOnce() -> Instant<T>>(self, now: F) -> Option<Instant<T>> where
        T: Copy,
    {
        match self {
            Moment::Absolute(instant) => Some(instant),
            Moment::Relative(duration) => now().value().checked_add(duration.value()).map(Instant),
        }
    }
}

impl<T> From<Duration<T>> for Moment<T> {
    #[inline]
    fn from(v: Duration<T>) -> Self {
        Self::Relative(v)
    }
}

impl<T> From<Instant<T>> for Moment<T> {
    #[inline]
    fn from(v: Instant<T>) -> Self {
        Self::Absolute(v)
    }
}

impl<T: Add<Output=T>> Add for Duration<T> {
    type Output = Self;

    #[inline]
    fn add(self, v: Self) -> Self::Output {
        Self(self.0.add(v.0))
    }
}

impl<T: Sub<Output=T>> Sub for Duration<T> {
    type Output = Self;

    #[inline]
    fn sub(self, v: Self) -> Self::Output {
        Self(self.0.sub(v.0))
    }
}

impl<T: Saturating> Saturating for Duration<T> {
    #[inline]
    fn saturating_add(self, v: Self) -> Self {
        Self(self.0.saturating_add(v.0))
    }

    #[inline]
    fn saturating_sub(self, v: Self) -> Self {
        Self(self.0.saturating_sub(v.0))
    }
}

impl<T: Saturating> Instant<T> {
    #[inline]
    pub fn saturating_add(self, v: Duration<T>) -> Self {
        Instant(self.0.saturating_add(v.0))
    }

    #[inline]
    pub fn saturating_sub(self, v: Self) -> Duration<T> {
        Duration(self.0.saturating_sub(v.0))
    }
}

impl<T: Add<Output=T>> Add<Duration<T>> for Instant<T> {
    type Output = Self;

    #[inline]
    fn add(self, v: Duration<T>) -> Self::Output {
        Self(self.0.add(v.0))
    }
}

impl<T: Sub<Output=T>> Sub<Duration<T>> for Instant<T> {
    type Output = Self;

    #[inline]
    fn sub(self, v: Duration<T>) -> Self::Output {
        Self(self.0.sub(v.0))
    }
}

impl<T: Sub<Output=T>> Sub for Instant<T> {
    type Output = Duration<T>;

    #[inline]
    fn sub(self, v: Self) -> Self::Output {
        Duration(self.0.sub(v.0))
    }
}
