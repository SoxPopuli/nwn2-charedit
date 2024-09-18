pub trait Pipe {
    fn pipe<T>(self, f: impl FnOnce(Self) -> T) -> T
    where
        Self: Sized,
    {
        f(self)
    }
}
impl<T> Pipe for T where T: ?Sized {}
