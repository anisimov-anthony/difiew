#[derive(Debug, PartialEq)]
pub enum StoreError {
    MonotreeError(String),
    RegexError(String),
}

impl From<monotree::Errors> for StoreError {
    fn from(err: monotree::Errors) -> Self {
        StoreError::MonotreeError(err.to_string())
    }
}

impl From<regex::Error> for StoreError {
    fn from(err: regex::Error) -> Self {
        StoreError::RegexError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_monotree_error() {
        let mock_err = "database locked".to_string();
        let err = StoreError::MonotreeError(mock_err.clone());
        let converted: StoreError = err;
        assert!(matches!(converted, StoreError::MonotreeError(_)));
    }

    #[test]
    fn from_regex_error() {
        let regex_err = regex::Error::Syntax("invalid".to_string());
        let store_err: StoreError = regex_err.into();
        assert!(matches!(store_err, StoreError::RegexError(s) if s.contains("invalid")));
    }

    #[test]
    fn store_error_debug() {
        let err = StoreError::MonotreeError("test".to_string());
        assert_eq!(format!("{:?}", err), "MonotreeError(\"test\")");
    }

    #[test]
    fn store_error_eq() {
        let e1 = StoreError::RegexError("bad".to_string());
        let e2 = StoreError::RegexError("bad".to_string());
        assert_eq!(e1, e2);
    }
}
