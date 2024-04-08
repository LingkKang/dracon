pub enum ErrCode {
    AcquireReadLockFailed,
    AcquireWriteLockFailed,
    EmptyKeyError,
    KeyNotFoundError,
    IndexUpdateFailed,
    OpenDataFileFailed,
    ReadDataFileFailed,
    SyncDataFileFailed,
    WriteDataFileFailed,
}

impl std::fmt::Debug for ErrCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrCode::AcquireReadLockFailed => {
                write!(f, "Failed to acquire read lock")
            }
            ErrCode::AcquireWriteLockFailed => {
                write!(f, "Failed to acquire write lock")
            }
            ErrCode::EmptyKeyError => write!(f, "The key is empty"),
            ErrCode::KeyNotFoundError => write!(f, "The key is not found"),
            ErrCode::IndexUpdateFailed => {
                write!(f, "Failed to update the index")
            }
            ErrCode::OpenDataFileFailed => {
                write!(f, "Failed to open data file")
            }
            ErrCode::ReadDataFileFailed => {
                write!(f, "Failed to read data file")
            }
            ErrCode::SyncDataFileFailed => {
                write!(f, "Failed to sync data file")
            }
            ErrCode::WriteDataFileFailed => {
                write!(f, "Failed to write file")
            }
        }
    }
}

pub struct Err {
    pub code: ErrCode,
    pub msg: String,
}

/// Match the result of acquiring a write lock.
/// If the result is an error, return the error
/// with the code [`ErrCode::AcquireWriteLockFailed`].
macro_rules! match_read_lock {
    ($result:expr) => {
        match $result {
            Ok(guard) => guard,
            Err(e) => {
                return Err(Err {
                    code: ErrCode::AcquireReadLockFailed,
                    msg: e.to_string(),
                });
            }
        }
    };
}

pub(crate) use match_read_lock;

/// Match the result of acquiring a write lock.
/// If the result is an error, return the error
/// with the code [`ErrCode::AcquireWriteLockFailed`].
macro_rules! match_write_lock {
    ($result:expr) => {
        match $result {
            Ok(guard) => guard,
            Err(e) => {
                return Err(Err {
                    code: ErrCode::AcquireWriteLockFailed,
                    msg: e.to_string(),
                });
            }
        }
    };
}

pub(crate) use match_write_lock;

impl std::fmt::Debug for Err {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: \"{}\"", self.code, self.msg)
    }
}

pub type Result<T> = std::result::Result<T, Err>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_err_debug() {
        let err = Err {
            code: ErrCode::ReadDataFileFailed,
            msg: "Error message here".to_string(),
        };
        println!("{:?}", err);
    }
}
