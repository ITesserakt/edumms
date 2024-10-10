use std::fmt::{Display, Formatter};
use std::ops::{Add, Div, Mul, Neg, Sub};
use itertools::{Itertools, MinMaxResult};

#[derive(Debug, Default, Hash, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
#[repr(C)]
pub struct Interval<T>(T, T);

impl<T> Interval<T> {
    pub fn new(start: T, end: T) -> Self {
        Self(start, end)
    }

    pub fn into_inner(self) -> (T, T) {
        (self.0, self.1)
    }
    pub fn start(self) -> T { self.0 }
    pub fn end(self) -> T { self.1 }
}

impl<T: Add> Add for Interval<T> {
    type Output = Interval<T::Output>;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Interval(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl<T: Sub> Sub for Interval<T> {
    type Output = Interval<T::Output>;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Interval(self.0 - rhs.1, self.1 - rhs.0)
    }
}

impl<T> Mul for Interval<T>
where
    T: Clone + Mul,
    T::Output: PartialOrd + Clone,
{
    type Output = Interval<T::Output>;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        match [self.0, self.1]
            .into_iter()
            .cartesian_product([rhs.0, rhs.1])
            .map(|(a, b)| a * b)
            .minmax()
        {
            MinMaxResult::MinMax(from, to) => Interval(from, to),
            MinMaxResult::NoElements => unreachable!(),
            MinMaxResult::OneElement(value) => Interval::from(value),
        }
    }
}

impl Mul<Interval<f64>> for f64 {
    type Output = Interval<f64>;

    fn mul(self, rhs: Interval<f64>) -> Self::Output {
        Interval(rhs.0 * self, rhs.1 * self)
    }
}

impl<T> Div for Interval<T>
where
    T: Clone + Div,
    T::Output: PartialOrd + Clone,
{
    type Output = Interval<T::Output>;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        match [self.0, self.1]
            .into_iter()
            .cartesian_product([rhs.0, rhs.1])
            .map(|(a, b)| a / b)
            .minmax()
        {
            MinMaxResult::MinMax(from, to) => Interval(from, to),
            MinMaxResult::NoElements => unreachable!(),
            MinMaxResult::OneElement(value) => Interval::from(value),
        }
    }
}

impl<T: Neg> Neg for Interval<T> {
    type Output = Interval<T::Output>;

    fn neg(self) -> Self::Output {
        Interval(-self.0, -self.1)
    }
}

impl<T: Clone> From<T> for Interval<T> {
    fn from(value: T) -> Self {
        Self(value.clone(), value)
    }
}

impl<T: Display> Display for Interval<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(precision) = f.precision() {
            write!(f, "[{:.*}..{:.*}]", precision, self.0, precision, self.1)
        } else {
            write!(f, "[{}..{}]", self.0, self.1)
        }
    }
}
