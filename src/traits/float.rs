/// Clamp a generic value between two other values.
#[inline(always)]
pub fn clamp_generic<T: PartialOrd>(x: T, lo: T, hi: T) -> T {
    if x < lo {
        lo
    } else if x > hi {
        hi
    } else {
        x
    }
}

/// A floating point trait that can be used synonymously
/// for f32 and f64 to cut down on code duplication.
/// This also prevents use of num-traits as a dependency.
pub trait Float: Copy + PartialOrd {
    const ZERO: Self;
    const ONE: Self;

    fn from_f32(x: f32) -> Self;
    fn from_f64(x: f64) -> Self;
    fn to_f32(self) -> f32;

    fn add(self, rhs: Self) -> Self;
    fn sub(self, rhs: Self) -> Self;
    fn mul(self, rhs: Self) -> Self;
    fn div(self, rhs: Self) -> Self;

    fn powf(self, e: Self) -> Self;
    fn cbrt(self) -> Self;
    /// Clamp this Float between 0.0 and 1.0
    fn clamp01(self) -> Self {
        clamp_generic(self, Self::ZERO, Self::ONE)
    }
}

impl Float for f32 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;

    #[inline]
    fn from_f32(x: f32) -> Self {
        x
    }
    #[inline]
    fn from_f64(x: f64) -> Self {
        x as f32
    }
    #[inline]
    fn to_f32(self) -> f32 {
        self
    }

    #[inline]
    fn add(self, rhs: Self) -> Self {
        self + rhs
    }
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        self - rhs
    }
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        self * rhs
    }
    #[inline]
    fn div(self, rhs: Self) -> Self {
        self / rhs
    }

    #[inline]
    fn powf(self, e: Self) -> Self {
        f32::powf(self, e)
    }
    #[inline]
    fn cbrt(self) -> Self {
        f32::cbrt(self)
    }
}

impl Float for f64 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;

    #[inline]
    fn from_f32(x: f32) -> Self {
        x as f64
    }
    #[inline]
    fn from_f64(x: f64) -> Self {
        x
    }
    #[inline]
    fn to_f32(self) -> f32 {
        self as f32
    }

    #[inline]
    fn add(self, rhs: Self) -> Self {
        self + rhs
    }
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        self - rhs
    }
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        self * rhs
    }
    #[inline]
    fn div(self, rhs: Self) -> Self {
        self / rhs
    }

    #[inline]
    fn powf(self, e: Self) -> Self {
        f64::powf(self, e)
    }
    #[inline]
    fn cbrt(self) -> Self {
        f64::cbrt(self)
    }
}
