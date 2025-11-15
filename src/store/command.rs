use bincode::{Decode, Encode};
use clap::Parser;
use std::borrow::Cow;

#[derive(Encode, Decode, Debug)]
pub enum StoreCommand<'a> {
    /// Removes the specified keys. A key is ignored if it does not exist
    DEL(DELParams<'a>),

    /// Checks keys existance
    EXISTS(EXISTSParams<'a>),

    /// Get the value of key
    GET(GETParams<'a>),

    /// Keys matching pattern
    KEYS(KEYSParams<'a>),

    /// Set key to hold the string value. If key already holds a value, it is overwritten
    SET(SETParams<'a>),
}

impl<'a> StoreCommand<'a> {
    pub fn del<K, I>(keys: I) -> Self
    where
        K: Into<Cow<'a, str>>,
        I: IntoIterator<Item = K>,
        I::IntoIter: 'a,
    {
        let keys = keys.into_iter().map(|k| k.into()).collect();
        StoreCommand::DEL(DELParams { keys })
    }

    pub fn exists<K, I>(keys: I) -> Self
    where
        K: Into<Cow<'a, str>>,
        I: IntoIterator<Item = K>,
        I::IntoIter: 'a,
    {
        let keys = keys.into_iter().map(|k| k.into()).collect();
        StoreCommand::EXISTS(EXISTSParams { keys })
    }

    pub fn get<K>(key: K) -> Self
    where
        K: Into<Cow<'a, str>>,
    {
        StoreCommand::GET(GETParams { key: key.into() })
    }

    pub fn keys<P>(pattern: P) -> Self
    where
        P: Into<Cow<'a, str>>,
    {
        StoreCommand::KEYS(KEYSParams {
            pattern: pattern.into(),
        })
    }

    pub fn set<K, V>(key: K, value: V) -> Self
    where
        K: Into<Cow<'a, str>>,
        V: Into<Cow<'a, str>>,
    {
        StoreCommand::SET(SETParams {
            key: key.into(),
            value: value.into(),
        })
    }
}

#[derive(Encode, Decode, Debug)]
pub struct DELParams<'a> {
    pub keys: Vec<Cow<'a, str>>,
}

#[derive(Encode, Decode, Debug)]
pub struct EXISTSParams<'a> {
    pub keys: Vec<Cow<'a, str>>,
}

#[derive(Encode, Decode, Debug)]
pub struct GETParams<'a> {
    pub key: Cow<'a, str>,
}

#[derive(Encode, Decode, Debug)]
pub struct KEYSParams<'a> {
    pub pattern: Cow<'a, str>,
}

#[derive(Encode, Decode, Debug)]
pub struct SETParams<'a> {
    pub key: Cow<'a, str>,
    pub value: Cow<'a, str>,
}

#[derive(Parser, Debug, Clone)]
#[command(version, about)]
pub struct CmdArgs {
    pub cmd_type: String,
    pub cmd_arg: String,
}

pub fn handle_cmd_input<'a>(args: &'a CmdArgs) -> Option<StoreCommand<'a>> {
    let cmd = args.cmd_type.to_uppercase();
    let mut cmd_args = args.cmd_arg.split_whitespace();

    match cmd.as_str() {
        "DEL" | "EXISTS" => {
            let keys: Vec<&str> = cmd_args.collect();
            if keys.is_empty() {
                eprintln!("Error: '{cmd}' requires at least one key");
                return None;
            }
            if cmd == "DEL" {
                Some(StoreCommand::del(keys))
            } else {
                Some(StoreCommand::exists(keys))
            }
        }

        "GET" | "KEYS" => {
            let first = match cmd_args.next() {
                Some(arg) => arg,
                None => {
                    eprintln!("Error: '{cmd}' requires exactly one argument");
                    return None;
                }
            };
            if cmd_args.next().is_some() {
                eprintln!("Error: '{cmd}' takes exactly one argument");
                return None;
            }
            if cmd == "GET" {
                Some(StoreCommand::get(first))
            } else {
                Some(StoreCommand::keys(first))
            }
        }

        "SET" => {
            let key = match cmd_args.next() {
                Some(k) => k,
                None => {
                    eprintln!("Error: '{cmd}' requires key and value");
                    return None;
                }
            };
            let value = match cmd_args.next() {
                Some(v) => v,
                None => {
                    eprintln!("Error: '{cmd}' requires value after key");
                    return None;
                }
            };
            if cmd_args.next().is_some() {
                eprintln!("Error: '{cmd}' takes exactly two arguments: key and value");
                return None;
            }
            Some(StoreCommand::set(key, value))
        }

        _ => {
            eprintln!("Error: unknown command '{cmd}'");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;

    #[test]
    fn store_command_del_multiple_keys() {
        let cmd = StoreCommand::del(vec!["key1", "key2"]);
        assert!(matches!(cmd, StoreCommand::DEL(_)));
        if let StoreCommand::DEL(params) = cmd {
            assert_eq!(
                params.keys,
                vec![Cow::Borrowed("key1"), Cow::Borrowed("key2")]
            );
        }
    }

    #[test]
    fn store_command_exists_from_strings() {
        let cmd = StoreCommand::exists(["a", "b"]);
        assert!(matches!(cmd, StoreCommand::EXISTS(_)));
        if let StoreCommand::EXISTS(params) = cmd {
            assert_eq!(params.keys.len(), 2);
        }
    }

    #[test]
    fn store_command_get() {
        let cmd = StoreCommand::get("user:123");
        assert!(matches!(cmd, StoreCommand::GET(_)));
        if let StoreCommand::GET(params) = cmd {
            assert_eq!(params.key, "user:123");
        }
    }

    #[test]
    fn store_command_keys() {
        let cmd = StoreCommand::keys("user:*");
        assert!(matches!(cmd, StoreCommand::KEYS(_)));
    }

    #[test]
    fn store_command_set() {
        let cmd = StoreCommand::set("theme", "dark");
        assert!(matches!(cmd, StoreCommand::SET(_)));
        if let StoreCommand::SET(params) = cmd {
            assert_eq!(params.key, "theme");
            assert_eq!(params.value, "dark");
        }
    }

    #[test]
    fn handle_cmd_input_del() {
        let args = CmdArgs {
            cmd_type: "del".to_string(),
            cmd_arg: "key1 key2".to_string(),
        };
        let cmd = handle_cmd_input(&args);
        assert!(cmd.is_some());
        assert!(matches!(cmd.unwrap(), StoreCommand::DEL(_)));
    }

    #[test]
    fn handle_cmd_input_exists() {
        let args = CmdArgs {
            cmd_type: "exists".to_string(),
            cmd_arg: "a b c".to_string(),
        };
        let cmd = handle_cmd_input(&args).unwrap();
        assert!(matches!(cmd, StoreCommand::EXISTS(_)));
    }

    #[test]
    fn handle_cmd_input_get_valid() {
        let args = CmdArgs {
            cmd_type: "GET".to_string(),
            cmd_arg: "mykey".to_string(),
        };
        let cmd = handle_cmd_input(&args).unwrap();
        assert!(matches!(cmd, StoreCommand::GET(_)));
    }

    #[test]
    fn handle_cmd_input_get_missing_arg() {
        let args = CmdArgs {
            cmd_type: "GET".to_string(),
            cmd_arg: "".to_string(),
        };
        assert!(handle_cmd_input(&args).is_none());
    }

    #[test]
    fn handle_cmd_input_set_valid() {
        let args = CmdArgs {
            cmd_type: "set".to_string(),
            cmd_arg: "color red".to_string(),
        };
        let cmd = handle_cmd_input(&args).unwrap();
        assert!(matches!(cmd, StoreCommand::SET(_)));
    }

    #[test]
    fn handle_cmd_input_set_missing_value() {
        let args = CmdArgs {
            cmd_type: "set".to_string(),
            cmd_arg: "color".to_string(),
        };
        assert!(handle_cmd_input(&args).is_none());
    }

    #[test]
    fn handle_cmd_input_unknown_command() {
        let args = CmdArgs {
            cmd_type: "unknown".to_string(),
            cmd_arg: "arg".to_string(),
        };
        assert!(handle_cmd_input(&args).is_none());
    }

    #[test]
    fn handle_cmd_input_case_insensitive() {
        let args = CmdArgs {
            cmd_type: "SeT".to_string(),
            cmd_arg: "k v".to_string(),
        };
        assert!(handle_cmd_input(&args).is_some());
    }
}
