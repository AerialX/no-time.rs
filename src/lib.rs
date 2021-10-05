#![no_std]
#![cfg_attr(feature = "unstable", feature(core_intrinsics, const_fn_trait_bound))]

use const_default::ConstDefault;
use num_traits::{CheckedAdd, Saturating};
use unchecked_ops::{UncheckedDiv, UncheckedSub, UncheckedMul, UncheckedAdd, UncheckedRem};
use typenum_fractional::{Fractional, ToPrimitive};
use core::ops::{Add, Sub, Mul, Div, Rem};
use core::marker::PhantomData;
use core::convert::TryFrom;

pub mod unit;
pub use unit::{Unit, UnitConversionTo, UnitConversionToValue, UnitConversionFrom, UnitConversionFromValue, units};

pub trait Repr: Copy { }

#[repr(transparent)]
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

impl<R, U: Unit> Duration<R, U> {
    pub fn convert_from<SU, SR>(value: Duration<SR, SU>) -> Option<Self> where
        SU: UnitConversionTo<U>,
        <SU::Factor as Fractional>::Denom: ToPrimitive<R> + ToPrimitive<SR>,
        <SU::Factor as Fractional>::Num: ToPrimitive<R>,
        R: TryFrom<SR>,
        SR: UncheckedDiv + UncheckedSub + UncheckedMul,
        <SU::Factor as Fractional>::Num: typenum::IsGreaterOrEqual<<SU::Factor as Fractional>::Denom>,
        R: num_traits::CheckedMul + num_traits::CheckedAdd + UncheckedSub + UncheckedMul + UncheckedDiv,
    {
        let value = value.value();
        let denom = <<SU::Factor as Fractional>::Denom as ToPrimitive<SR>>::VALUE;
        let denom_dest = <<SU::Factor as Fractional>::Denom as ToPrimitive<R>>::VALUE;
        let num = <<SU::Factor as Fractional>::Num as ToPrimitive<R>>::VALUE;

        unsafe {
            let div = value.unchecked_div(denom); // denom is NonZero
            let div_res = R::try_from(div).ok()?;
            let res = div_res.checked_mul(&num);
            if <<<SU::Factor as Fractional>::Num as typenum::IsGreaterOrEqual<<SU::Factor as Fractional>::Denom>>::Output as typenum::Bit>::BOOL {
                // assuming an integer repr, this add+div only needs to occur if factor > 1/1, which can determined at compile-time
                let rem = value.unchecked_sub(div.unchecked_mul(denom)); // value % denom
                let rem_res = R::try_from(rem).ok()?; // TODO: can this be unchecked or something..? We already assume the denom must fit in the dest repr...
                res?.checked_add(&rem_res.checked_mul(&num)?.unchecked_div(denom_dest))
            } else {
                res
            }.map(Self::from_value)
        }
    }

    #[inline]
    pub fn typenum<V: typenum::Unsigned, S>() -> Self where
        /*S: UnitConversionToValue<U, V>,
        S::Output: ToPrimitive<R>,*/
        U: UnitConversionFromValue<S, V>,
        U::Output: ToPrimitive<R>,
    {
        //Self::from_value(<S::Output as ToPrimitive<R>>::VALUE)
        Self::from_value(<U::Output as ToPrimitive<R>>::VALUE)
    }
}

#[cfg(feature = "unstable")]
impl<U: Unit> Duration<u64, U> {
    #[inline]
    pub const fn const_value(&self) -> u64 {
        self.value
    }

    #[inline]
    pub const fn literal_value<S: UnitConversionTo<U>>(value: u64) -> u64 {
        unit::lit::<S, U>(value)
    }

    #[inline]
    pub const fn literal<S: UnitConversionTo<U>>(value: u64) -> Self {
        Self::from_value(Self::literal_value::<S>(value))
    }

    /// Ugly hack around the inability to call generic const code.
    ///
    /// This should be optimized away easily enough at least?
    #[inline]
    pub fn trunc<R: Copy + 'static>(self) -> Duration<R, U> where
        u64: num_traits::AsPrimitive<R>,
    {
        Duration::from_value(num_traits::AsPrimitive::as_(self.value))
    }
}

#[doc(hidden)]
pub mod __export {
    pub use typenum::consts as typenum_consts;
}

#[macro_export]
#[cfg(feature = "unstable")]
macro_rules! duration_for {
    ($num:ident $unit:path) => {
        $crate::Duration::typenum::<$crate::__export::typenum_consts::$num, $unit>()
    };
    ($num:tt $unit:path: $dest:path) => {
        {
            const __NO_TIME_DURATION_INIT: u64 = $crate::Duration::<u64, $dest>::literal::<$unit>($num).const_value();
            $crate::Duration::<_, $dest>::from_value(__NO_TIME_DURATION_INIT as _)
        }
    };
}

#[macro_export]
#[cfg(not(feature = "unstable"))]
macro_rules! duration_for {
    ($num:ident $unit:path) => {
        $crate::Duration::typenum::<$crate::__export::typenum_consts::$num, $unit>()
    };
}

#[repr(transparent)]
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
    pub fn from_origin(duration: Duration<R, U>) -> Self {
        Self {
            duration,
        }
    }

    #[inline]
    pub fn since_origin(self) -> Duration<R, U> {
        self.duration
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

impl<R, U: Unit> Instant<R, U> {
    pub fn convert_from<SU, SR>(value: Instant<SR, SU>) -> Option<Self> where
        SU: UnitConversionTo<U>,
        <SU::Factor as Fractional>::Denom: ToPrimitive<R> + ToPrimitive<SR>,
        <SU::Factor as Fractional>::Num: ToPrimitive<R>,
        R: TryFrom<SR>,
        SR: UncheckedDiv + UncheckedSub + UncheckedMul,
        <SU::Factor as Fractional>::Num: typenum::IsGreaterOrEqual<<SU::Factor as Fractional>::Denom>,
        R: num_traits::CheckedMul + num_traits::CheckedAdd + UncheckedSub + UncheckedMul + UncheckedDiv,
    {
        Duration::convert_from(value.since_origin()).map(Self::from_origin)
    }
}

impl<R: Saturating, U> Instant<R, U> {
    #[inline]
    pub fn saturating_sub(self, rhs: Duration<R, U>) -> Self {
        Self::from(self.duration.saturating_sub(rhs))
    }

    #[inline]
    pub fn saturating_add(self, rhs: Duration<R, U>) -> Self {
        Self::from(self.duration.saturating_add(rhs))
    }

    #[inline]
    pub fn saturating_diff(self, rhs: Instant<R, U>) -> Duration<R, U> {
        self.duration.saturating_sub(rhs.duration)
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

impl<R: Mul<Output=R>, U> Mul<R> for Duration<R, U> {
    type Output = Self;

    #[inline]
    fn mul(self, v: R) -> Self::Output {
        Self::from_value(self.value.mul(v))
    }
}

impl<R: Div<Output=R>, U> Div for Duration<R, U> {
    type Output = R;

    #[inline]
    fn div(self, v: Self) -> Self::Output {
        self.value.div(v.value)
    }
}

impl<R: Rem<Output=R>, U> Rem for Duration<R, U> {
    type Output = Self;

    #[inline]
    fn rem(self, rhs: Self) -> Self::Output {
        Self::from_value(self.value.rem(rhs.value))
    }
}

impl<R: UncheckedAdd, U> UncheckedAdd for Duration<R, U> {
    #[inline]
    unsafe fn unchecked_add(self, rhs: Self) -> Self {
        Self::from_value(self.value.unchecked_add(rhs.value))
    }
}

impl<R: UncheckedSub, U> UncheckedSub for Duration<R, U> {
    #[inline]
    unsafe fn unchecked_sub(self, rhs: Self) -> Self {
        Self::from_value(self.value.unchecked_sub(rhs.value))
    }
}

impl<R: UncheckedRem, U> UncheckedRem for Duration<R, U> {
    #[inline]
    unsafe fn unchecked_rem(self, rhs: Self) -> Self {
        Self::from_value(self.value.unchecked_rem(rhs.value))
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

#[test]
fn smoke_convert() {
    let duration: Duration<u8, units::Seconds> = Duration::from_value(8);
    let converted: Duration<u64, units::Subseconds32> = Duration::convert_from(duration).unwrap();

    assert_eq!(converted.value(), duration.value() as u64 * 0x100000000)
}
