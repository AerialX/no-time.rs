use typenum::{Unsigned, Prod};
use typenum_fractional::{
    Fractional, Truncated, Truncate,
    ConstFraction,
};
use core::ops::{Div, Mul};

pub use typenum_fractional::ToPrimitive;

pub trait Unit {
    type Seconds: Fractional;
}

pub trait UnitConversionTo<D>: Unit {
    type Factor: Fractional;

    const FACTOR: ConstFraction;
}

pub type Lit<V, S, D> = Truncated<Prod<<S as UnitConversionTo<D>>::Factor, V>>;

pub trait UnitConversionToValue<D, V>: UnitConversionTo<D> {
    type Output: Unsigned;
}

impl<O, S, D, V> UnitConversionToValue<D, V> for S where
    S: UnitConversionTo<D>,
    S::Factor: Mul<V, Output=O>,
    O: Truncate,
{
    type Output = Truncated<O>;
}

pub trait UnitConversionFrom<S>: Unit {
    type Factor: Fractional;

    const FACTOR: ConstFraction;
}

pub trait UnitConversionFromValue<S, V>: UnitConversionFrom<S> {
    type Output: Unsigned;
}

impl<D: Unit, S: UnitConversionTo<D>> UnitConversionFrom<S> for D {
    type Factor = S::Factor;

    const FACTOR: ConstFraction = S::FACTOR;
}

impl<O, S, D, V> UnitConversionFromValue<S, V> for D where
    D: UnitConversionFrom<S>,
    D::Factor: Mul<V, Output=O>,
    O: Truncate,
{
    type Output = Truncated<O>;
}

#[cfg(feature = "unstable")]
pub const fn lit<S: Unit + UnitConversionTo<D>, D: Unit>(v: u64) -> u64 {
    let factor = S::FACTOR;
    factor.const_mul(ConstFraction::const_from_scalar(v)).const_truncate()
}

impl<O, S: Unit, D: Unit> UnitConversionTo<D> for S where
    S::Seconds: Div<D::Seconds, Output=O>,
    O: Fractional + ToPrimitive<ConstFraction>,
{
    type Factor = O;

    const FACTOR: ConstFraction = O::VALUE;
}

#[allow(non_camel_case_types)]
pub mod units {
    use super::Unit;
    use typenum::Prod;
    use typenum_fractional::Fraction;
    use typenum::consts::*;

    macro_rules! impl_unit {
        ($($id:ident / $per:ty),*) => {
            $(
                #[derive(Debug, Default, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
                pub struct $id;
                impl Unit for $id {
                    type Seconds = $per;
                }

                // TODO: inherent const and/or const fn for conversion factors?
            )*
        };
    }

    impl_unit! {
        Days / Fraction<Prod<U24, Prod<U60, U60>>, U1>,
        Hours / Fraction<Prod<U60, U60>, U1>,
        Minutes / Fraction<U60, U1>,
        Seconds / Fraction<U1, U1>,
        Deciseconds / Fraction<U1, U10>,
        Debiseconds / Fraction<U1, U16>,
        Cebiseconds_2 / Fraction<U1, U64>,
        Centiseconds / Fraction<U1, U100>,
        Cebiseconds / Fraction<U1, U128>,
        Milliseconds / Fraction<U1, U1000>,
        Mibiseconds / Fraction<U1, U1024>
    }
}

#[test]
fn smoke() {
    use units::{Days, Minutes};
    use typenum::Unsigned;
    use typenum::consts::*;

    let value = Lit::<U2, Days, Minutes>::U32; // 2 days to minutes
    assert_eq!(value, 60 * 24 * 2);
    #[cfg(feature = "unstable")]
    {
        const DAY2_MIN: u64 = lit::<Days, Minutes>(2); // const fn variant of the above
        assert_eq!(DAY2_MIN, 60 * 24 * 2);
    }
}
