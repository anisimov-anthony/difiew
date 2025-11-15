use bincode::{Decode, Encode};
use std::borrow::Cow;

#[derive(Debug, Encode, Decode, PartialEq, Clone)]
pub enum StoreCommandResult<'a> {
    DEL(DELResult),
    EXISTS(EXISTSResult),
    GET(GETResult<'a>),
    KEYS(KEYSResult<'a>),
    SET(SETResult),
    UNDEFINED(UNDEFINEDResult<'a>),
}

impl<'a> StoreCommandResult<'a> {
    pub fn del(removed: usize) -> Self {
        StoreCommandResult::DEL(DELResult { payload: removed })
    }

    pub fn exists(count: usize) -> Self {
        StoreCommandResult::EXISTS(EXISTSResult { payload: count })
    }

    pub fn get<V>(value: Option<V>) -> Self
    where
        V: Into<Cow<'a, str>>,
    {
        StoreCommandResult::GET(GETResult {
            payload: value.map(|v| v.into()),
        })
    }

    pub fn keys<I>(keys: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Cow<'a, str>>,
    {
        let payload = keys.into_iter().map(|k| k.into()).collect();
        StoreCommandResult::KEYS(KEYSResult { payload })
    }

    pub fn set(success: bool) -> Self {
        StoreCommandResult::SET(SETResult { payload: success })
    }

    pub fn undefined<V>(message: V) -> Self
    where
        V: Into<Cow<'a, str>>,
    {
        StoreCommandResult::UNDEFINED(UNDEFINEDResult {
            payload: message.into(),
        })
    }
}

#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub struct DELResult {
    /// the number of keys that were removed
    pub payload: usize,
}

#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub struct EXISTSResult {
    /// the number of keys that exist from those specified as arguments
    pub payload: usize,
}

#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub struct GETResult<'a> {
    /// the value associated with the key, or `None` if not found
    pub payload: Option<Cow<'a, str>>,
}

#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub struct KEYSResult<'a> {
    /// a list of keys matching pattern
    pub payload: Vec<Cow<'a, str>>,
}

#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub struct SETResult {
    /// `true` if the key was set
    pub payload: bool,
}

#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub struct UNDEFINEDResult<'a> {
    /// service message about the reason for the uncertainty of the result
    pub payload: Cow<'a, str>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;

    #[test]
    fn result_del() {
        let res = StoreCommandResult::del(3);
        assert_eq!(res, StoreCommandResult::DEL(DELResult { payload: 3 }));
    }

    #[test]
    fn result_exists() {
        let res = StoreCommandResult::exists(2);
        assert_eq!(res, StoreCommandResult::EXISTS(EXISTSResult { payload: 2 }));
    }

    #[test]
    fn result_get_some() {
        let res = StoreCommandResult::get(Some("value"));
        assert!(
            matches!(res, StoreCommandResult::GET(r) if r.payload == Some(Cow::Borrowed("value")))
        );
    }

    #[test]
    fn result_get_none() {
        let res = StoreCommandResult::get::<String>(None);
        assert!(matches!(res, StoreCommandResult::GET(r) if r.payload.is_none()));
    }

    #[test]
    fn result_keys() {
        let keys = vec!["a", "b"];
        let res = StoreCommandResult::keys(keys);
        assert!(matches!(res, StoreCommandResult::KEYS(r) if r.payload.len() == 2));
    }

    #[test]
    fn result_set_success() {
        let res = StoreCommandResult::set(true);
        assert_eq!(res, StoreCommandResult::SET(SETResult { payload: true }));
    }

    #[test]
    fn result_undefined() {
        let res = StoreCommandResult::undefined("not supported");
        assert!(matches!(res, StoreCommandResult::UNDEFINED(r) if r.payload == "not supported"));
    }

    #[test]
    fn bincode_roundtrip_del() {
        let original = StoreCommandResult::del(5);
        let encoded = bincode::encode_to_vec(&original, bincode::config::standard()).unwrap();
        let (decoded, _): (StoreCommandResult, _) =
            bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn bincode_roundtrip_get_with_string() {
        let original = StoreCommandResult::get(Some("hello".to_string()));
        let encoded = bincode::encode_to_vec(&original, bincode::config::standard()).unwrap();
        let (decoded, _): (StoreCommandResult, _) =
            bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();
        assert_eq!(original, decoded);
    }
}
