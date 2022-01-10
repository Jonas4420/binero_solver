use std::ops;

#[derive(Clone, Copy, Debug)]
pub struct Index(pub usize, pub usize);

impl Index {
    pub fn line<T>(&self, i: T) -> Self
    where
        T: Into<i64>,
    {
        let i = i.into();

        if i.is_positive() {
            *self + Index(i as usize, 0)
        } else {
            *self - Index(i.abs() as usize, 0)
        }
    }

    pub fn col<T>(&self, j: T) -> Self
    where
        T: Into<i64>,
    {
        let j = j.into();

        if j.is_positive() {
            *self + Index(0, j as usize)
        } else {
            *self - Index(0, j.abs() as usize)
        }
    }
}

impl<T> ops::Add<T> for Index
where
    T: Into<Index>,
{
    type Output = Self;

    fn add(self, other: T) -> Self::Output {
        let other = other.into();
        Index(self.0 + other.0, self.1 + other.1)
    }
}

impl<T> ops::Sub<T> for Index
where
    T: Into<Index>,
{
    type Output = Self;

    fn sub(self, other: T) -> Self::Output {
        let other = other.into();
        Index(self.0 - other.0, self.1 - other.1)
    }
}

impl From<(usize, usize)> for Index {
    fn from((i, j): (usize, usize)) -> Self {
        Index(i, j)
    }
}
