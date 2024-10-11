/*!
Deserialize connection strings with Serde.
*/

use serde::{de::Visitor, Deserialize};
use std::{fmt::Display, str::FromStr};

use crate::ConnectionString;

#[derive(Clone, Debug, PartialEq)]
pub struct Error(String);

impl std::error::Error for Error {}

impl serde::de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Default)]
struct ConnectionStringVisitor;

impl<'de> Visitor<'de> for ConnectionStringVisitor {
    type Value = ConnectionString;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("privilege")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match ConnectionString::from_str(v) {
            Ok(s) => Ok(s),
            Err(e) => Err(E::custom(e.to_string())),
        }
    }
}

impl<'de> Deserialize<'de> for ConnectionString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(ConnectionStringVisitor)
    }
}

#[cfg(test)]
mod tests {
    use crate::HostSpec;

    use super::*;

    #[derive(Debug, Deserialize, PartialEq)]
    struct SingleConn {
        conn: ConnectionString,
    }

    #[test]
    fn test_from_yml() {
        let input = "conn: postgres://localhost/mydb";

        assert_eq!(
            serde_yaml::from_str::<SingleConn>(input).unwrap(),
            SingleConn {
                conn: ConnectionString {
                    user: None,
                    password: None,
                    hostspecs: vec![HostSpec {
                        host: "localhost".to_string(),
                        port: None,
                    }],
                    database: Some("mydb".to_string()),
                    parameters: vec![],
                    fragment: None,
                }
            }
        );
    }
}
