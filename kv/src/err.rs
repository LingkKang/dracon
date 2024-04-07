pub enum ErrCode {
    OpenDataFileFailed,
    ReadDataFileFailed,
    SyncDataFileFailed,
    WriteDataFileFailed,
}

impl std::fmt::Debug for ErrCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
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
