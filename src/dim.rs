//! Layout dimension wrapping [zenith-float](https://crates.io/crates/zenith-float) 1.0 `ExactNum`.
//!
//! This crate depends on the published `zenith-float` crate, not on its internal
//! kernel package. Every arithmetic path uses software limbs. Hardware `f32` /
//! `f64` never appear as calculation terminals.

use core::cmp::Ordering;
use core::fmt;
use core::ops::{Add, Div, Mul, Neg, Sub};
use std::sync::Arc;

use crate::error::Error;
use zenith_float::{Consts, ExactNum, Radix, RoundingMode};

/// Working precision for layout `Dim` values, in bits.
pub const DIM_PREC: usize = 256;
const RM: RoundingMode = RoundingMode::ToEven;

fn consts() -> Consts {
    Consts::new().expect("zenith-float constants cache")
}

fn wrap(n: ExactNum) -> Dim {
    Dim { inner: Arc::new(n) }
}

/// TeX-style dimension: width, height, depth, italic correction, mu.
///
/// Values are zenith-float software floats. One unit is one em at the current
/// math style unless a method says otherwise.
///
/// # Examples
///
/// ```
/// use latex_rust::Dim;
///
/// let half = Dim::ratio(1, 2);
/// assert!(half.eq_dim(&(&Dim::one() / &Dim::from_i64(2))));
/// assert!(!Dim::zero().eq_dim(&Dim::one()));
/// ```
#[derive(Clone, Debug)]
pub struct Dim {
    inner: Arc<ExactNum>,
}

impl Dim {
    /// Zero em.
    #[must_use]
    pub fn zero() -> Self {
        wrap(ExactNum::new(DIM_PREC))
    }

    /// One em.
    #[must_use]
    pub fn one() -> Self {
        wrap(ExactNum::from_i32(1, DIM_PREC))
    }

    /// Integer em count.
    #[must_use]
    pub fn from_i64(v: i64) -> Self {
        wrap(ExactNum::from_i64(v, DIM_PREC))
    }

    /// Exact rational `num / den` em. `den == 0` yields NaN.
    #[must_use]
    pub fn ratio(num: i64, den: i64) -> Self {
        Self::from_i64(num) / Self::from_i64(den)
    }

    /// Parse a decimal string (including scientific form) with zenith-float.
    #[must_use]
    pub fn parse(s: &str) -> Self {
        let mut cc = consts();
        wrap(ExactNum::parse(s, Radix::Dec, DIM_PREC, RM, &mut cc))
    }

    /// Convert integer font units to em: `units / units_per_em`.
    #[must_use]
    pub fn from_font_units(units: i64, units_per_em: u16) -> Self {
        Self::from_i64(units) / Self::from_i64(i64::from(units_per_em))
    }

    /// One math unit (mu). TeX: `18 mu = 1 em`.
    #[must_use]
    pub fn mu() -> Self {
        Self::ratio(1, 18)
    }

    /// Convert this em value to mu (`* 18`).
    #[must_use]
    pub fn to_mu(&self) -> Self {
        self.clone() * Self::from_i64(18)
    }

    /// Convert a mu value to em (`/ 18`).
    #[must_use]
    pub fn from_mu(mu: &Self) -> Self {
        mu.clone() / Self::from_i64(18)
    }

    /// Absolute value.
    #[must_use]
    pub fn abs(&self) -> Self {
        wrap(self.inner.as_ref().abs())
    }

    /// Maximum of two dimensions.
    #[must_use]
    pub fn max(&self, other: &Self) -> Self {
        match self.cmp(other) {
            Some(Ordering::Less) => other.clone(),
            _ => self.clone(),
        }
    }

    /// Minimum of two dimensions.
    #[must_use]
    pub fn min(&self, other: &Self) -> Self {
        match self.cmp(other) {
            Some(Ordering::Greater) => other.clone(),
            _ => self.clone(),
        }
    }

    /// `max(self, 0)`.
    #[must_use]
    pub fn clamp_nonneg(&self) -> Self {
        self.max(&Self::zero())
    }

    /// True when the value is NaN.
    #[must_use]
    pub fn is_nan(&self) -> bool {
        self.inner.is_nan()
    }

    /// True when the value compares equal to zero.
    #[must_use]
    pub fn is_zero(&self) -> bool {
        matches!(self.inner.cmp(&ExactNum::new(DIM_PREC)), Some(0))
    }

    /// Decimal string from zenith-float (gold-stable for a given precision).
    #[must_use]
    pub fn to_dec_string(&self) -> String {
        let mut cc = consts();
        self.inner
            .format(Radix::Dec, RM, &mut cc)
            .unwrap_or_else(|_| "NaN".into())
    }

    /// Compact decimal for SVG attributes (same zenith-float decimal as layout golds).
    #[must_use]
    pub fn to_svg_string(&self) -> String {
        self.to_dec_string()
    }

    /// Layout dimension from an IEEE-754 binary32 bit pattern (ttf-parser outline boundary).
    #[must_use]
    pub fn from_ieee32_bits(bits: u32) -> Self {
        wrap(zenith_float::Ieee32::from_bits(bits).to_exact(DIM_PREC))
    }

    /// Round to IEEE-754 binary32 bits for PNG / raster emission only.
    ///
    /// Layout arithmetic stays in [`Dim`]. This is the pixel-coordinate terminal
    /// permitted for SVG/PNG backends.
    #[must_use]
    pub fn to_ieee32_bits(&self) -> u32 {
        zenith_float::Ieee32::from_exact(self.inner.as_ref()).to_bits()
    }

    /// Largest `u32` that is not greater than `self`. Negative and NaN fail.
    pub fn floor_to_u32(&self) -> Result<u32, Error> {
        if self.is_nan() {
            return Err(Error::InvalidOption {
                what: "dimension is NaN".into(),
            });
        }
        if matches!(self.cmp(&Self::zero()), Some(Ordering::Less)) {
            return Err(Error::InvalidOption {
                what: "negative dimension".into(),
            });
        }
        const MAX: u32 = 1 << 20;
        if matches!(
            self.cmp(&Self::from_i64(i64::from(MAX))),
            Some(Ordering::Greater) | Some(Ordering::Equal)
        ) {
            return Err(Error::InvalidOption {
                what: "dimension exceeds raster limit".into(),
            });
        }
        let mut ans = 0u32;
        let mut bit = 1u32 << 19;
        while bit > 0 {
            let cand = ans + bit;
            if Self::from_i64(i64::from(cand))
                .cmp(self)
                .is_some_and(|o| o != Ordering::Greater)
            {
                ans = cand;
            }
            bit /= 2;
        }
        Ok(ans)
    }

    /// Smallest `u32` that is not less than `self`.
    pub fn ceil_to_u32(&self) -> Result<u32, Error> {
        let floor = self.floor_to_u32()?;
        if self.eq_dim(&Self::from_i64(i64::from(floor))) {
            Ok(floor)
        } else {
            floor.checked_add(1).ok_or_else(|| Error::InvalidOption {
                what: "dimension overflow".into(),
            })
        }
    }

    /// Compare two dimensions. `None` if either is NaN.
    ///
    /// Named `cmp` after zenith-float's partial compare. [`Ord`] is not
    /// implemented because NaN has no total order.
    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn cmp(&self, other: &Self) -> Option<Ordering> {
        match self.inner.cmp(&other.inner) {
            Some(0) => Some(Ordering::Equal),
            Some(x) if x < 0 => Some(Ordering::Less),
            Some(_) => Some(Ordering::Greater),
            None => None,
        }
    }

    /// True when `self` and `other` compare equal.
    #[must_use]
    pub fn eq_dim(&self, other: &Self) -> bool {
        matches!(self.cmp(other), Some(Ordering::Equal))
    }
}

impl PartialEq for Dim {
    fn eq(&self, other: &Self) -> bool {
        self.eq_dim(other)
    }
}

impl Eq for Dim {}

impl PartialOrd for Dim {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.cmp(other)
    }
}

impl fmt::Display for Dim {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_dec_string())
    }
}

fn add_e(a: &ExactNum, b: &ExactNum) -> ExactNum {
    ExactNum::add(a, b, DIM_PREC, RM)
}

fn sub_e(a: &ExactNum, b: &ExactNum) -> ExactNum {
    ExactNum::sub(a, b, DIM_PREC, RM)
}

fn mul_e(a: &ExactNum, b: &ExactNum) -> ExactNum {
    ExactNum::mul(a, b, DIM_PREC, RM)
}

fn div_e(a: &ExactNum, b: &ExactNum) -> ExactNum {
    ExactNum::div(a, b, DIM_PREC, RM)
}

impl Neg for Dim {
    type Output = Self;

    fn neg(self) -> Self {
        wrap(self.inner.as_ref().neg())
    }
}

macro_rules! impl_dim_op {
    ($Trait:ident, $method:ident, $helper:ident) => {
        impl $Trait for Dim {
            type Output = Dim;
            fn $method(self, rhs: Dim) -> Dim {
                wrap($helper(self.inner.as_ref(), rhs.inner.as_ref()))
            }
        }
        impl $Trait<&Dim> for Dim {
            type Output = Dim;
            fn $method(self, rhs: &Dim) -> Dim {
                wrap($helper(self.inner.as_ref(), rhs.inner.as_ref()))
            }
        }
        impl $Trait<Dim> for &Dim {
            type Output = Dim;
            fn $method(self, rhs: Dim) -> Dim {
                wrap($helper(self.inner.as_ref(), rhs.inner.as_ref()))
            }
        }
        impl $Trait for &Dim {
            type Output = Dim;
            fn $method(self, rhs: &Dim) -> Dim {
                wrap($helper(self.inner.as_ref(), rhs.inner.as_ref()))
            }
        }
    };
}

impl_dim_op!(Add, add, add_e);
impl_dim_op!(Sub, sub, sub_e);
impl_dim_op!(Mul, mul, mul_e);
impl_dim_op!(Div, div, div_e);
