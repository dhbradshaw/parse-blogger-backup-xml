use std::fmt::Debug;

pub type BtError = Box<dyn std::error::Error>;
pub type BtResult<T> = Result<T, BtError>;
pub type EmptyResult = Result<(), BtError>;

pub fn errstr<T: Debug>(message: &str, error: T) -> String {
    format!("{} Error is `{:?}`", message, error)
}
