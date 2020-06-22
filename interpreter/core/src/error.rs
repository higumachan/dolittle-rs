#[derive(Debug)]
pub enum Error {
    MethodNotFound,
    ObjectNotFound,
    MemberNotFound,
    ArgumentError,
    Runtime,
}

pub type Result<T> = std::result::Result<T, Error>;
