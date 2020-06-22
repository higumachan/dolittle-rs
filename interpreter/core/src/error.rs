#[derive(Debug)]
pub enum Error {
    MethodNotFound,
    ObjectNotFound,
    MemberNotFound,
    Runtime,
}

pub type Result<T> = std::result::Result<T, Error>;
