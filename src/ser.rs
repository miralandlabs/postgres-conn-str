use crate::ConnectionString;
use serde::Serialize;

impl Serialize for ConnectionString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ConnectionString, HostSpec, Parameter};

    #[derive(Debug, Serialize, PartialEq)]
    struct SingleConn {
        conn: ConnectionString,
    }

    #[test]
    fn test_to_yml() {
        for (input, expect) in &[
            (
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
                    },
                },
                "---\nconn: \"postgresql://localhost/mydb\"\n",
            ),
            (
                SingleConn {
                    conn: ConnectionString {
                        user: Some("jackb".into()),
                        password: Some("supersecret".into()),
                        hostspecs: vec![
                            HostSpec {
                                host: "localhost".to_string(),
                                port: Some(456),
                            },
                            HostSpec {
                                host: "otherhost".to_string(),
                                port: Some(123),
                            },
                        ],
                        database: Some("mydb".to_string()),
                        parameters: vec![
                            Parameter {
                                keyword: "timeout".into(),
                                value: "60".into(),
                            }
                        ],
                        fragment: Some("tagyes".into()),
                    },
                },
                "---\nconn: \"postgresql://jackb:supersecret@localhost:456,otherhost:123/mydb?timeout=60#tagyes\"\n",
            ),
        ] {
            let res = serde_yaml::to_string(&input);
            assert_eq!(&res.unwrap(), expect, "input: {:?}", input);
        }
    }
}
