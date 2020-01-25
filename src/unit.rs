use core::convert::TryFrom;
use core::marker::PhantomData;

use num_traits::{self, CheckedAdd, CheckedSub, CheckedMul, Saturating};
use typenum::{Unsigned, NonZero, Bit, Prod, Quot, Mod, consts::*};
use core::ops::{Add, Sub, Mul, Div};
use const_default::ConstDefault;

pub trait Ratio {
    type Num: Unsigned + NonZero;
    type Denom: Unsigned + NonZero;
}

pub struct ConstRatio<N, D> {
    //_internal: PhantomData<fn() -> (N, D)>,
    _internal: PhantomData<(N, D)>,
}

impl<N, D> ConstRatio<N, D> {
    pub const fn new() -> Self {
        Self {
            _internal: PhantomData,
        }
    }
}

impl<N, D> ConstDefault for ConstRatio<N, D> {
    const DEFAULT: Self = ConstRatio::new();
}

impl<N: Unsigned + NonZero, D: Unsigned + NonZero> Ratio for ConstRatio<N, D> {
    type Num = N;
    type Denom = D;
}

pub mod gcd {
    use core::ops::Rem;
    use core::marker::PhantomData;
    use typenum::{Bit, Mod, Cmp, UTerm, NonZero, Unsigned, UInt, consts::{U0, U1, B0, B1}};

    pub trait Iff {
        type Output;
    }

    struct IffImpl<Cond: Bit, If, Else> {
        _internal: PhantomData<(Cond, If, Else)>,
    }

    impl<If, Else> Iff for IffImpl<B0, If, Else> {
        type Output = Else;
    }

    impl<If, Else> Iff for IffImpl<B1, If, Else> {
        type Output = If;
    }

    pub type If<Cond, If, Else> = <IffImpl<Cond, If, Else> as Iff>::Output;
    pub type IsZero<T> = typenum::Eq<T, U0>;
    pub type Gcd<LHS, RHS> = <LHS as Common<RHS>>::Output;

    pub trait Common<RHS> {
        type Output;
    }

    impl<LHS> Common<LHS> for UTerm {
        type Output = LHS;
    }

    impl<U: Unsigned, B: Bit> Common<UTerm> for UInt<U, B> {
        type Output = Self;
    }

    pub trait NonZero_ { }
    impl<U: Unsigned, B: Bit> NonZero_ for UInt<U, B> { }

    impl<U: Unsigned, B: Bit, RHS: Cmp<U0>> Common<RHS> for UInt<U, B> where
        //RHS: Common<Mod<Self, RHS>>,
        //Self: NonZero + Cmp<U0> + Rem<RHS>,
        RHS: NonZero_,
        Self: NonZero,
        RHS: Common<Mod<Self, RHS>>,
        //RHS: Rem<Self>,
        Self: Rem<RHS>,
    {
        //type Output = If<IsZero<LHS>, RHS, If<IsZero<RHS>, LHS, Gcd<RHS, Mod<LHS, RHS>>>>;
        //type Output = Gcd<RHS, Mod<Self, RHS>>;
        type Output = Gcd<RHS, Mod<Self, RHS>>;
    }

    /*impl<RHS: Cmp<U0>> Common<RHS> for U1 where
        /*RHS: Common<Mod<Self, RHS>>,
        Self: NonZero + Cmp<U0> + Rem<RHS>,*/
    {
        type Output = Self;
    }*/

    #[cfg(test)]
    mod tests {
        use super::Gcd;
        use typenum::assert_type_eq;
        use typenum::consts::*;

        #[test]
        fn test_gcd() {
            assert_type_eq!(Gcd<U0, U1>, U1);
            assert_type_eq!(Gcd<U1, U0>, U1);
            assert_type_eq!(Gcd<U1, U1>, U1);
            assert_type_eq!(Gcd<U1, U9>, U1);
            assert_type_eq!(Gcd<U2, U4>, U2);
            assert_type_eq!(Gcd<U4, U6>, U2);
        }
    }

    /*impl<LHS: Cmp<U0> + Rem<RHS>, RHS: Cmp<U0>> Common<RHS> for LHS {
        type Output = If<IsZero<LHS>, RHS, If<IsZero<RHS>, LHS, Gcd<RHS, Mod<LHS, RHS>>>>;
    }*/
}
use gcd::{Gcd, Common};

pub trait Reduce {
    type Output;
}

// TODO: impl Ratio for UTerm?

/*pub trait Over1 {
    type Output;
}

impl<T: Unsigned + NonZero> Over1 for T {
    type Output = ConstRatio<T, U1>;
}

pub type ToRatio<T> = <T as Over1>::Output;*/

impl<T: Unsigned + NonZero> Ratio for T {
    type Num = T;
    type Denom = U1;
}

pub trait Round {
    type Output: Unsigned;
}

impl<O, R: Ratio> Round for R where
    R::Num: Div<R::Denom, Output=O>,
    O: Unsigned,
{
    type Output = O;
}

impl<O, OR, N: Unsigned + NonZero, D: Unsigned + NonZero, RHS: Ratio> typenum::Cmp<RHS> for ConstRatio<N, D> where
    N: Mul<RHS::Denom, Output=O>,
    RHS::Num: Mul<D, Output=OR>,
    O: typenum::Cmp<OR>,
{
    type Output = typenum::Compare<O, OR>;
}

impl<N: Unsigned + NonZero + Common<D>, D: Unsigned + NonZero> Reduce for ConstRatio<N, D> where
    N: Div<Gcd<N, D>>,
    D: Div<Gcd<N, D>>,
{
    type Output = ConstRatio<Quot<N, Gcd<N, D>>, Quot<D, Gcd<N, D>>>;
}

pub type Reduced<T> = <T as Reduce>::Output;

impl<O, N: Unsigned + NonZero + Mul<R::Num>, D: Unsigned + NonZero + Mul<R::Denom>, R: Ratio> Mul<R> for ConstRatio<N, D> where
    ConstRatio<Prod<N, R::Num>, Prod<D, R::Denom>>: Reduce<Output=O>,
    O: ConstDefault,
{
    type Output = O;

    fn mul(self, _: R) -> Self::Output {
        ConstDefault::DEFAULT
    }
}

pub trait Reciprocal {
    type Output;
}

impl<N, D> Reciprocal for ConstRatio<N, D> {
    type Output = ConstRatio<D, N>;
}

pub type Invert<T> = <T as Reciprocal>::Output;

impl<O, N: Unsigned + NonZero, D: Unsigned + NonZero, R: Ratio> Div<R> for ConstRatio<N, D> where
    R: Reciprocal,
    Self: Mul<Invert<R>, Output=O>,
    O: ConstDefault,
{
    type Output = O;

    fn div(self, _: R) -> Self::Output {
        ConstDefault::DEFAULT
    }
}

#[test]
fn test_ratio_ops() {
    use typenum::assert_type_eq;
    assert_type_eq!(Prod<ConstRatio<U1, U2>, ConstRatio<U1, U4>>, ConstRatio<U1, U8>);
    assert_type_eq!(Prod<ConstRatio<U1, U2>, ConstRatio<U60, U1>>, ConstRatio<U30, U1>);
    assert_type_eq!(Quot<ConstRatio<U1, U2>, ConstRatio<U60, U1>>, ConstRatio<U1, U120>);
}

/*pub trait UncheckedOps {
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
}*/

pub trait Unit/*: Sized + ConstDefault + CheckedAdd*/ {
    //type Repr: Repr;
    type Seconds: Ratio;

    /*fn from_repr(repr: Self::Repr) -> Self;
    fn to_repr(&self) -> Self::Repr;*/

    /*
    #[inline]
    fn to_unit<U: Unit>(&self) -> Option<U> where ConvImpl<Self, U>: Conv<Self, U> {
        <ConvImpl<Self, U> as Conv<Self, U>>::conv(self.to_repr())
            .map(U::from_repr)
    }

    #[inline]
    fn from_unit<U: Unit>(u: &U) -> Option<Self> where ConvImpl<U, Self>: Conv<U, Self> {
        <ConvImpl<U, Self> as Conv<U, Self>>::conv(u.to_repr())
            .map(Self::from_repr)
    }*/
}

pub trait Literal<D, V: Unsigned> {
    type Output: Unsigned;
}

impl<O, OO, R, S: Unit, D: Unit, V: Unsigned> Literal<D, V> for S where
    S::Seconds: Div<D::Seconds, Output=R>,
    R: Mul<V, Output=OO>,
    OO: Round<Output=O>,
    O: Unsigned,
{
    type Output = O;
}

pub type Lit<V, S, D> = <S as Literal<D, V>>::Output;

pub trait ToInt<R> {
    const VALUE: R;
}

impl<T: Unsigned> ToInt<u8> for T {
    const VALUE: u8 = T::U8;
}

impl<T: Unsigned> ToInt<u16> for T {
    const VALUE: u16 = T::U16;
}

impl<T: Unsigned> ToInt<u32> for T {
    const VALUE: u32 = T::U32;
}

impl<T: Unsigned> ToInt<u64> for T {
    const VALUE: u64 = T::U64;
}

impl<T: Unsigned> ToInt<usize> for T {
    const VALUE: usize = T::USIZE;
}

pub trait ConstLiteral<D, R> {
    type Ratio: Ratio;
}

impl<O, US: Unit, UD: Unit, R> ConstLiteral<UD, R> for US where
    US::Seconds: Div<UD::Seconds, Output=O>,
    O: Ratio,
{
    type Ratio = O;
}

#[cfg(feature = "unstable")]
pub const fn literal<US: Unit, UD: Unit, R: Mul<Output=R> + Div<Output=R>>(val: R) -> R where
    US: ConstLiteral<UD, R>,
    <<US as ConstLiteral<UD, R>>::Ratio as Ratio>::Num: ToInt<R>,
    <<US as ConstLiteral<UD, R>>::Ratio as Ratio>::Denom: ToInt<R>,
{
    val * <<<US as ConstLiteral<UD, R>>::Ratio as Ratio>::Num as ToInt<R>>::VALUE / <<<US as ConstLiteral<UD, R>>::Ratio as Ratio>::Denom as ToInt<R>>::VALUE
}

/*pub trait Conv<S: Unit, D: Unit> {
    fn conv(src: S::Repr) -> Option<D::Repr>;

    type Num: Unsigned + NonZero;
    type Denom: Unsigned + NonZero;
}

pub struct ConvImpl<S, D>(PhantomData<fn(S) -> D>);*/

/*

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
}*/

/*
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
*/

macro_rules! impl_unit {
    /*(@lit $id:ident, $per:ty, $ty:ty) => {
        #[cfg(feature = "unstable")]
        impl $id<$ty> {
            #[inline]
            pub const fn literal<S: Unit>(value: u64) -> Self {
                Self((value * <<S::Seconds as Ratio>::Num as Unsigned>::U64 * <<$per as Ratio>::Denom as Unsigned>::U64 / <<S::Seconds as Ratio>::Denom as Unsigned>::U64 / <<$per as Ratio>::Num as Unsigned>::U64) as $ty)
            }
        }
    };*/
    ($($id:ident / $per:ty),*) => {
        $(
            #[derive(Debug, Default, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
            pub struct $id;
            impl Unit for $id {
                type Seconds = $per;

                                /*
                #[inline]
                fn from_repr(repr: Self::Repr) -> Self {
                    $id(repr)
                }

                #[inline]
                fn to_repr(&self) -> Self::Repr {
                    self.0.clone()
                }*/
            }

                        /*
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

                #[inline]
                fn add(self, v: Self) -> Self::Output {
                    Self(self.0.add(v.0))
                }
            }

            impl<T: CheckedAdd> CheckedAdd for $id<T> {
                #[inline]
                fn checked_add(&self, v: &Self) -> Option<Self> {
                    self.0.checked_add(&v.0).map(Self)
                }
            }

            impl<T: Sub<Output=T>> Sub for $id<T> {
                type Output = Self;

                #[inline]
                fn sub(self, v: Self) -> Self::Output {
                    Self(self.0.sub(v.0))
                }
            }

            impl<T: CheckedSub> CheckedSub for $id<T> {
                #[inline]
                fn checked_sub(&self, v: &Self) -> Option<Self> {
                    self.0.checked_sub(&v.0).map(Self)
                }
            }

            impl<T: Saturating> Saturating for $id<T> {
                #[inline]
                fn saturating_add(self, v: Self) -> Self {
                    Self(self.0.saturating_add(v.0))
                }

                #[inline]
                fn saturating_sub(self, v: Self) -> Self {
                    Self(self.0.saturating_sub(v.0))
                }
            }

            impl<T: UncheckedOps> UncheckedOps for $id<T> {
                #[inline]
                unsafe fn unchecked_add(&self, val: Self) -> Self {
                    Self(self.0.unchecked_add(val.0))
                }

                #[inline]
                unsafe fn unchecked_sub(&self, val: Self) -> Self {
                    Self(self.0.unchecked_sub(val.0))
                }

                #[inline]
                unsafe fn unchecked_mul(&self, val: Self) -> Self {
                    Self(self.0.unchecked_mul(val.0))
                }

                #[inline]
                unsafe fn unchecked_div(&self, val: Self) -> Self {
                    Self(self.0.unchecked_div(val.0))
                }
            }

            impl_unit! { @lit $id, $per, usize }
            impl_unit! { @lit $id, $per, u8 }
            impl_unit! { @lit $id, $per, u16 }
            impl_unit! { @lit $id, $per, u32 }
            impl_unit! { @lit $id, $per, u64 }*/
        )*
    };
}

impl_unit! {
    Days / ConstRatio<Prod<U24, Prod<U60, U60>>, U1>,
    Hours / ConstRatio<Prod<U60, U60>, U1>,
    Minutes / ConstRatio<U60, U1>,
    Seconds / ConstRatio<U1, U1>,
    Deciseconds / ConstRatio<U1, U10>,
    Debiseconds / ConstRatio<U1, U16>,
    Cebiseconds_2 / ConstRatio<U1, U64>,
    Centiseconds / ConstRatio<U1, U100>,
    Cebiseconds / ConstRatio<U1, U128>,
    Milliseconds / ConstRatio<U1, U1000>,
    Mibiseconds / ConstRatio<U1, U1024>
}

#[test]
fn smoke() {
    let seconds = Lit::<U2, Days, Seconds>::U32; // 2 days to seconds
    assert_eq!(seconds, 60 * 60 * 24 * 2);
    #[cfg(feature = "unstable")]
    {
        const DAY2_SECONDS: u32 = literal::<Days, Seconds, _>(2);
        assert_eq!(DAY2_SECONDS, 60 * 60 * 24 * 2);
    }
}
