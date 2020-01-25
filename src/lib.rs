#![no_std]
#![cfg_attr(feature = "unstable", feature(core_intrinsics, const_fn))]

use const_default::ConstDefault;
use num_traits::{CheckedAdd, Saturating};
use core::ops::{Add, Sub};
use core::marker::PhantomData;

mod unit;

pub use unit::*;

pub trait Repr: Copy { }

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Duration<R, U> {
    value: R,
    _unit: PhantomData<U>,
}

impl<R: Copy, U> Copy for Duration<R, U> { }

impl<R: Clone, U> Clone for Duration<R, U> {
    fn clone(&self) -> Self {
        Duration::from_value(self.value.clone())
    }
}

impl<R: Default, U> Default for Duration<R, U> {
    fn default() -> Self {
        Duration::from_value(Default::default())
    }
}

impl<R: ConstDefault, U> ConstDefault for Duration<R, U> {
    const DEFAULT: Self = Self::from_value(R::DEFAULT);
}

impl<R: ConstDefault, U> ConstDefault for Instant<R, U> {
    const DEFAULT: Self = Self::from_value(R::DEFAULT);
}

impl<R, U> Duration<R, U> {
    #[inline]
    pub const fn from_value(value: R) -> Self {
        Self {
            value,
            _unit: PhantomData,
        }
    }

    #[inline]
    pub fn value(self) -> R {
        self.value
    }

    #[inline]
    pub const fn value_ref(&self) -> &R {
        &self.value
    }

    #[inline]
    pub fn value_mut(&mut self) -> &mut R {
        &mut self.value
    }
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Instant<R, U> {
    duration: Duration<R, U>,
}

impl<R: Copy, U> Copy for Instant<R, U> { }

impl<R: Clone, U> Clone for Instant<R, U> {
    fn clone(&self) -> Self {
        Instant::from(self.duration.clone())
    }
}

impl<R: Default, U> Default for Instant<R, U> {
    fn default() -> Self {
        Instant::from(Duration::default())
    }
}

impl<R, U> Instant<R, U> {
    #[inline]
    pub const fn from_value(value: R) -> Self {
        Self {
            duration: Duration::from_value(value),
        }
    }

    #[inline]
    pub fn value(self) -> R {
        self.duration.value()
    }

    #[inline]
    pub const fn value_ref(&self) -> &R {
        self.duration.value_ref()
    }

    #[inline]
    pub fn value_mut(&mut self) -> &mut R {
        self.duration.value_mut()
    }
}

/*impl<R: From<RD>, RD, U> From<Duration<RD, U>> for Duration<R, U> {
    fn from(duration: Duration<RD, U>) -> Self {
        Self::from_value(duration.value.into())
    }
}*/

impl<R: From<RD>, RD, U> From<Duration<RD, U>> for Instant<R, U> {
    #[inline]
    fn from(duration: Duration<RD, U>) -> Self {
        Self {
            duration: Duration::from_value(duration.value.into()),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum Moment<R, U> {
    Relative(Duration<R, U>),
    Absolute(Instant<R, U>),
}

#[cfg(feature = "unstable")]
impl<R: ConstDefault, U> Moment<R, U> {
    #[inline]
    pub const fn immediate_const() -> Self {
        Moment::Relative(ConstDefault::DEFAULT)
    }
}

impl<R: Default, U> Moment<R, U> {
    #[inline]
    pub fn immediate() -> Self {
        Moment::Relative(Default::default())
    }
}

impl<R: CheckedAdd + Default, U> Moment<R, U> {
    #[inline]
    pub fn to_absolute<F: FnOnce() -> Instant<R, U>>(self, now: F) -> Option<Instant<R, U>> where
        R: Copy,
    {
        match self {
            Moment::Absolute(instant) => Some(instant),
            Moment::Relative(duration) => now().value().checked_add(duration.value_ref()).map(Instant::from_value),
        }
    }
}

impl<R, U> From<Duration<R, U>> for Moment<R, U> {
    #[inline]
    fn from(v: Duration<R, U>) -> Self {
        Self::Relative(v)
    }
}

impl<R, U> From<Instant<R, U>> for Moment<R, U> {
    #[inline]
    fn from(v: Instant<R, U>) -> Self {
        Self::Absolute(v)
    }
}

impl<R: Add<Output=R>, U> Add for Duration<R, U> {
    type Output = Self;

    #[inline]
    fn add(self, v: Self) -> Self::Output {
        Self::from_value(self.value.add(v.value))
    }
}

impl<R: Sub<Output=R>, U> Sub for Duration<R, U> {
    type Output = Self;

    #[inline]
    fn sub(self, v: Self) -> Self::Output {
        Self::from_value(self.value.sub(v.value))
    }
}

impl<R: Saturating, U> Saturating for Duration<R, U> {
    #[inline]
    fn saturating_add(self, v: Self) -> Self {
        Self::from_value(self.value.saturating_add(v.value))
    }

    #[inline]
    fn saturating_sub(self, v: Self) -> Self {
        Self::from_value(self.value.saturating_sub(v.value))
    }
}

impl<RHS, R, RD, U> Add<RHS> for Instant<R, U> where
    Duration<R, U>: Add<RHS, Output=Duration<RD, U>>
{
    type Output = Instant<RD, U>;

    #[inline]
    fn add(self, v: RHS) -> Self::Output {
        Instant::from(self.duration.add(v))
    }
}

impl<RHS, R, RD, U> Sub<RHS> for Instant<R, U> where
    Duration<R, U>: Sub<RHS, Output=Duration<RD, U>>
{
    type Output = Instant<RD, U>;

    #[inline]
    fn sub(self, v: RHS) -> Self::Output {
        Instant::from(self.duration.sub(v))
    }
}

impl<R: Sub<Output=R>, U> Sub for Instant<R, U> {
    type Output = Duration<R, U>;

    #[inline]
    fn sub(self, v: Self) -> Self::Output {
        Duration::from_value(self.duration.value.sub(v.duration.value))
    }
}
