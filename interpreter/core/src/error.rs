#[derive(Debug)]
pub enum Error {
    MethodNotFound,
    ObjectNotFound,
    Runtime,
}

pub type Result<T> = std::result::Result<T, Error>;
