use std::ops::{Add, Div, Mul, Sub, Index};

/// trait to encompass basic arithmetic operations
pub trait Numeric<T>:
    Clone + Copy + Add<T, Output = T> + Div<T, Output = T> + Mul<T, Output = T> + Sub<T, Output = T>
{
}

/// implement Numeric<T> for all T satisfying the basic arithmetic operations.
impl<
        T: Clone
            + Copy
            + Add<T, Output = T>
            + Div<T, Output = T>
            + Mul<T, Output = T>
            + Sub<T, Output = T>,
    > Numeric<T> for T
{
}

/// generic point type that supports adding, subtracting, multiplying, and dividing points
/// as well as scaling points by a provided T.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Point<T, const N: usize>
where
    T: Numeric<T>,
{
    pub v: [T; N],
}

impl<T, const N: usize> Point<T, N>
where
    T: Numeric<T>,
{
    pub const fn new(v: [T; N]) -> Point<T, N> {
        Point { v }
    }
}

impl<const N: usize> Point<f64, N> {
    pub fn lerp(self, other: Point<f64, N>, amount: f64) -> Point<f64, N> {
        let mut combined = [0.0; N];
        for i in 0..N {
            combined[i] = self.v[i] * amount + other.v[i] * (1.0 - amount);
        }
        return Point::new(combined);
    }
}

impl<T, const N: usize> Add<Point<T, N>> for Point<T, N>
where
    T: Numeric<T>,
{
    type Output = Point<T, N>;

    fn add(self, rhs: Self) -> Self::Output {
        let mut i = 0;
        self.v
            .map(|e| {
                let res = e + rhs.v[i];
                i += 1;
                res
            })
            .into()
    }
}

impl<T, const N: usize> Sub<Point<T, N>> for Point<T, N>
where
    T: Numeric<T>,
{
    type Output = Point<T, N>;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut i = 0;
        self.v
            .map(|e| {
                let res = e - rhs.v[i];
                i += 1;
                res
            })
            .into()
    }
}

impl<T, const N: usize> Mul<Point<T, N>> for Point<T, N>
where
    T: Numeric<T>,
{
    type Output = Point<T, N>;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut i = 0;
        self.v
            .map(|e| {
                let res = e * rhs.v[i];
                i += 1;
                res
            })
            .into()
    }
}

impl<T, const N: usize> Div<Point<T, N>> for Point<T, N>
where
    T: Numeric<T>,
{
    type Output = Point<T, N>;

    fn div(self, rhs: Self) -> Self::Output {
        let mut i = 0;
        self.v
            .map(|e| {
                let res = e / rhs.v[i];
                i += 1;
                res
            })
            .into()
    }
}

impl<T, const N: usize> Div<T> for Point<T, N>
where
    T: Numeric<T>,
{
    type Output = Point<T, N>;

    fn div(self, rhs: T) -> Self::Output {
        self.v.map(|e| e / rhs).into()
    }
}

impl<T, const N: usize> Mul<T> for Point<T, N>
where
    T: Numeric<T>,
{
    type Output = Point<T, N>;

    fn mul(self, rhs: T) -> Self::Output {
        self.v.map(|e| e * rhs).into()
    }
}

impl<T, const N: usize> From<[T; N]> for Point<T, N>
where
    T: Numeric<T>,
{
    fn from(value: [T; N]) -> Self {
        Point::new(value)
    }
}

impl<T, U, const N: usize> From<(Point<T, N>,)> for Point<U, N>
where
    U: From<T> + Numeric<U>,
    T: Numeric<T>,
{
    fn from(value: (Point<T, N>,)) -> Self {
        value.0.v.map(|x| x.into()).into()
    }
}

impl<T, const N: usize> Index<usize> for Point<T, N> 
where
    T: Numeric<T>,
{
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.v[index]
    }
}