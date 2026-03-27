#[derive(uniffi::Error)]
pub enum Error {
}

pub type ResultAlias<T> = ResultAlias2<T, Error>;

pub type ResultAlias2<T, E> = ResultAlias2<T, ()>;
