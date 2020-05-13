pub type ErrBox = Box<dyn std::error::Error>;

#[derive(std::fmt::Debug)]
pub struct Error(String);

impl Error {
    pub fn new(text: &str) -> Box<Self> {
        Box::new(Error(String::from(text)))
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for Error {}
