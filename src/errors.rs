#[derive(Debug, PartialEq)]
pub enum MatecitoErrors {
    CacheFull(String),
}
#[derive(Debug, PartialEq)]
pub enum MatecitoResult<T> {
    Ok(T),
    Err(MatecitoErrors),
}
