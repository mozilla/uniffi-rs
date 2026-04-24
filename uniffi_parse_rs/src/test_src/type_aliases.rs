#[derive(uniffi::Record)]
pub struct Record { }

pub type RecordAlias = Record;

pub mod submod {
    #[derive(uniffi::Record)]
    pub struct Record2 { }
}

mod submod2 {
    // alias that spans several submodules
    pub type SubmoduleRecordAlias = super::SubmoduleRecordAlias;
}

pub type SubmoduleRecordAlias = submod::Record2;

pub type UnitAlias = ();

#[derive(uniffi::Error)]
pub enum FileError { }

#[derive(uniffi::Error)]
pub enum OtherError { }

// Test a couple different forms for Result aliases
pub type FileResult<T> = Result<T, FileError>;
pub type FileResult2<T, E = FileError> = Result<T, E>;
pub type FileResult3 = Result<u32, FileError>;

// Type aliases that form a circular reference
pub type CircularAlias = CircularAlias2;
pub type CircularAlias2 = CircularAlias;
