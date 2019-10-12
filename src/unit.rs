use core::convert::TryFrom;
use core::marker::PhantomData;

use num_traits::{self, CheckedAdd, CheckedSub, CheckedMul, Saturating};
use typenum::{Unsigned, NonZero, Prod, consts::*};
use core::ops::{Add, Sub, Mul};
use const_default::ConstDefault;

pub trait Ratio {
    type Num: Unsigned + NonZero;
    type Denom: Unsigned + NonZero;
}

pub struct ConstRatio<N, D> {
    _internal: PhantomData<fn() -> (N, D)>,
}

impl<N: Unsigned + NonZero, D: Unsigned + NonZero> Ratio for ConstRatio<N, D> {
    type Num = N;
    type Denom = D;
}

pub trait UncheckedOps {
    unsafe fn unchecked_add(&self, val: Self) -> Self;
    unsafe fn unchecked_sub(&self, val: Self) -> Self;
    unsafe fn unchecked_mul(&self, val: Self) -> Self;
    unsafe fn unchecked_div(&self, val: Self) -> Self;
}

pub trait Repr:
Sized + Clone + num_traits::CheckedAdd
+ num_traits::CheckedSub + num_traits::CheckedMul + num_traits::CheckedDiv
+ num_traits::Saturating + UncheckedOps
{
    fn from_const<I: Unsigned>() -> Self;
}

pub trait Unit: Sized + ConstDefault {
    type Repr: Repr;
    type Seconds: Ratio;

    fn from_repr(repr: Self::Repr) -> Self;
    fn to_repr(&self) -> Self::Repr;

    #[inline]
    fn to_unit<U: Unit>(&self) -> Option<U> where ConvImpl<Self, U>: Conv<Self, U> {
        <ConvImpl<Self, U> as Conv<Self, U>>::conv(self.to_repr())
            .map(U::from_repr)
    }

    #[inline]
    fn from_unit<U: Unit>(u: &U) -> Option<Self> where ConvImpl<U, Self>: Conv<U, Self> {
        <ConvImpl<U, Self> as Conv<U, Self>>::conv(u.to_repr())
            .map(Self::from_repr)
    }
}

pub trait Conv<S: Unit, D: Unit> {
    fn conv(src: S::Repr) -> Option<D::Repr>;

    type Num: Unsigned + NonZero;
    type Denom: Unsigned + NonZero;
}

pub struct ConvImpl<S, D>(PhantomData<fn(S) -> D>);

impl<Num, Denom, SR: Ratio, DR: Ratio, S: Unit<Seconds=SR>, D: Unit<Seconds=DR>> Conv<S, D> for ConvImpl<S, D> where
SR::Denom: Mul<DR::Num, Output=Num>, DR::Denom: Mul<SR::Num, Output=Denom>,
D::Repr: TryFrom<S::Repr>,
Num: Unsigned + NonZero, Denom: Unsigned + NonZero,
{
    type Num = Num;
    type Denom = Denom;

    #[inline]
    fn conv(src: S::Repr) -> Option<D::Repr> {
        use core::mem::size_of;

        // s * d.num * s.denom / (d.denom * s.num);

        // TODO these num/denoms need to be reduced :(
        let s_num = S::Repr::from_const::<Num>();
        let d_num = D::Repr::from_const::<Num>();
        let s_denom = S::Repr::from_const::<Denom>();
        let d_denom = D::Repr::from_const::<Denom>();

        if size_of::<D>() >= size_of::<S>() {
            let d_src = D::Repr::try_from(src).ok()?;
            d_src.checked_mul(&d_num).map(|d| unsafe { d.unchecked_div(d_denom) })
        } else {
            src.checked_mul(&s_num).map(|s| unsafe { s.unchecked_div(s_denom) })
                .and_then(|s| D::Repr::try_from(s).ok())
        }
    }
}

macro_rules! impl_number {
    ($($ty:ty),*) => {
        $(
            impl Repr for $ty {

                #[inline]
                fn from_const<I: Unsigned>() -> Self {
                    // TODO statically assert that I::U64 <= Self::max_value()
                    I::U64 as Self
                }
            }

            #[cfg(feature = "unstable")]
            impl UncheckedOps for $ty {
                #[inline]
                unsafe fn unchecked_add(&self, val: Self) -> Self {
                    core::intrinsics::unchecked_add(*self, val)
                }

                #[inline]
                unsafe fn unchecked_sub(&self, val: Self) -> Self {
                    core::intrinsics::unchecked_sub(*self, val)
                }

                #[inline]
                unsafe fn unchecked_mul(&self, val: Self) -> Self {
                    core::intrinsics::unchecked_mul(*self, val)
                }

                #[inline]
                unsafe fn unchecked_div(&self, val: Self) -> Self {
                    core::intrinsics::unchecked_div(*self, val)
                }
            }

            #[cfg(not(feature = "unstable"))]
            impl UncheckedOps for $ty {
                // TODO can also implement using checked + unreachable_unchecked
                // but apparently that doesn't generate the desired instructions...
                #[inline]
                unsafe fn unchecked_add(&self, val: Self) -> Self {
                    *self + val
                }

                #[inline]
                unsafe fn unchecked_sub(&self, val: Self) -> Self {
                    *self - val
                }

                #[inline]
                unsafe fn unchecked_mul(&self, val: Self) -> Self {
                    *self * val
                }

                #[inline]
                unsafe fn unchecked_div(&self, val: Self) -> Self {
                    *self / val
                }
            }
        )*
    };
}

impl_number! { u8, u16, u32, u64, usize }

macro_rules! impl_unit {
    (@lit $id:ident, $per:ty, $ty:ty) => {
        #[cfg(feature = "unstable")]
        impl $id<$ty> {
            #[inline]
            pub const fn literal<S: Unit>(value: u64) -> Self {
                Self((value * <<S::Seconds as Ratio>::Num as Unsigned>::U64 * <<$per as Ratio>::Denom as Unsigned>::U64 / <<S::Seconds as Ratio>::Denom as Unsigned>::U64 / <<$per as Ratio>::Num as Unsigned>::U64) as $ty)
            }
        }
    };
    ($($id:ident / $per:ty),*) => {
        $(
            #[derive(Debug, Default, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
            pub struct $id<T>(T);
            impl<T: Repr + ConstDefault> Unit for $id<T> {
                type Repr = T;
                type Seconds = $per;

                #[inline]
                fn from_repr(repr: Self::Repr) -> Self {
                    $id(repr)
                }

                #[inline]
                fn to_repr(&self) -> Self::Repr {
                    self.0.clone()
                }
            }

            impl<T: Repr> $id<T> {
                #[inline]
                #[cfg(feature = "unstable")]
                pub const fn new(val: T) -> Self {
                    Self(val)
                }

                #[inline]
                #[cfg(not(feature = "unstable"))]
                pub fn new(val: T) -> Self {
                    Self(val)
                }

                #[inline]
                pub fn as_repr(&self) -> &T {
                    &self.0
                }

                #[inline]
                pub fn repr_mut(&mut self) -> &mut T {
                    &mut self.0
                }

                #[inline]
                pub fn into_repr(self) -> T {
                    self.0
                }
            }

            impl<T: ConstDefault> ConstDefault for $id<T> {
                const DEFAULT: Self = $id(T::DEFAULT);
            }

            impl<T: Add<Output=T>> Add for $id<T> {
                type Output = Self;

                fn add(self, v: Self) -> Self::Output {
                    Self(self.0.add(v.0))
                }
            }

            impl<T: CheckedAdd> CheckedAdd for $id<T> {
                fn checked_add(&self, v: &Self) -> Option<Self> {
                    self.0.checked_add(&v.0).map(Self)
                }
            }

            impl<T: Sub<Output=T>> Sub for $id<T> {
                type Output = Self;

                fn sub(self, v: Self) -> Self::Output {
                    Self(self.0.sub(v.0))
                }
            }

            impl<T: CheckedSub> CheckedSub for $id<T> {
                fn checked_sub(&self, v: &Self) -> Option<Self> {
                    self.0.checked_sub(&v.0).map(Self)
                }
            }

            impl<T: Saturating> Saturating for $id<T> {
                fn saturating_add(self, v: Self) -> Self {
                    Self(self.0.saturating_add(v.0))
                }

                fn saturating_sub(self, v: Self) -> Self {
                    Self(self.0.saturating_sub(v.0))
                }
            }

            impl<T: UncheckedOps> UncheckedOps for $id<T> {
                unsafe fn unchecked_add(&self, val: Self) -> Self {
                    Self(self.0.unchecked_add(val.0))
                }

                unsafe fn unchecked_sub(&self, val: Self) -> Self {
                    Self(self.0.unchecked_sub(val.0))
                }

                unsafe fn unchecked_mul(&self, val: Self) -> Self {
                    Self(self.0.unchecked_mul(val.0))
                }

                unsafe fn unchecked_div(&self, val: Self) -> Self {
                    Self(self.0.unchecked_div(val.0))
                }
            }

            impl_unit! { @lit $id, $per, usize }
            impl_unit! { @lit $id, $per, u8 }
            impl_unit! { @lit $id, $per, u16 }
            impl_unit! { @lit $id, $per, u32 }
            impl_unit! { @lit $id, $per, u64 }
        )*
    };
}

impl_unit! {
    Days / ConstRatio<Prod<U24, Prod<U60, U60>>, U1>,
    Hours / ConstRatio<Prod<U60, U60>, U1>,
    Minutes / ConstRatio<U60, U1>,
    Seconds / ConstRatio<U1, U1>,
    Centiseconds / ConstRatio<U1, U100>,
    Cebiseconds / ConstRatio<U1, U128>,
    Milliseconds / ConstRatio<U1, U1000>,
    Mibiseconds / ConstRatio<U1, U1024>
}

#[test]
#[cfg(feature = "unstable")]
fn smoke() {
    let seconds = Seconds::<u32>::literal::<Days<u64>>(2);
    assert_eq!(seconds.to_repr(), 60 * 60 * 24 * 2);
}
