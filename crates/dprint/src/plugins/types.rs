use crate::types::ErrBox;

// trait alias hack (https://www.worthe-it.co.za/programming/2017/01/15/aliasing-traits-in-rust.html)
pub trait CompileFn: Fn(&[u8]) -> Result<Vec<u8>, ErrBox> {
}

impl<T> CompileFn for T where T : Fn(&[u8]) -> Result<Vec<u8>, ErrBox> {
}
