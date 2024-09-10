pub trait Pipe {
    fn pipe<T>(self, f: impl FnOnce(Self) -> T) -> T
    where
        Self: Sized,
    {
        f(self)
    }
}
impl<T> Pipe for T where T: ?Sized {}

pub trait MapFirst {
    type A;
    type B;
    fn map_first<T>(self, f: impl FnOnce(Self::A) -> T) -> (T, Self::B);
}
impl<A, B> MapFirst for (A, B) {
    type A = A;
    type B = B;
    fn map_first<T>(self, f: impl FnOnce(Self::A) -> T) -> (T, Self::B) {
        (f(self.0), self.1)
    }
}

pub trait MapSecond<A, B, T> {
    fn map_second(self, f: impl FnOnce(B) -> T) -> (A, T);
}
impl<A, B, T> MapSecond<A, B, T> for (A, B) {
    fn map_second(self, f: impl FnOnce(B) -> T) -> (A, T) {
        (self.0, f(self.1))
    }
}

pub trait BindFirst {
    type A;
    type B;
    fn bind_first<T, E>(self, f: impl FnOnce(Self::A) -> Result<T, E>) -> Result<(T, Self::B), E>;
}
impl<A, B> BindFirst for (A, B)  {
    type A = A;
    type B = B;
    fn bind_first<T, E>(self, f: impl FnOnce(Self::A) -> Result<T, E>) -> Result<(T, Self::B), E> {
        match f(self.0) {
            Ok(x) => Ok((x, self.1)),
            Err(e) => Err(e),
        }
    }
}

pub trait BindSecond {
    type A;
    type B;
    fn bind_second<T, E>(self, f: impl FnOnce(Self::B) -> Result<T, E>) -> Result<(Self::A, T), E>;
}
impl<A, B> BindSecond for (A, B) {
    type A = A;
    type B = B;
    fn bind_second<T, E>(self, f: impl FnOnce(Self::B) -> Result<T, E>) -> Result<(Self::A, T), E> {
        match f(self.1) {
            Ok(y) => Ok((self.0, y)),
            Err(e) => Err(e),
        }
    }
}

pub mod sequence_result {
    pub fn tuple_second<A, B, E>(x: (A, Result<B, E>)) -> Result<(A, B), E> {
        match x.1 {
            Ok(y) => Ok((x.0, y)),
            Err(e) => Err(e),
        }
    }

    pub fn tuple_first<A, B, E>(x: (Result<A, E>, B)) -> Result<(A, B), E> {
        match x.0 {
            Ok(y) => Ok((y, x.1)),
            Err(e) => Err(e),
        }
    }
}
