use super::authority::{host::HostSpec, userinfo::UserSpec, Authority};
use super::*;
use crate::{ConnectionString, Parameter};
use std::str::FromStr;

#[test]
fn test_multi_connection_string() {
    for (input, sep, output) in vec![
        ("postgres://", ",", vec![ConnectionUri::default()]),
        (
            "postgres://,postgres://",
            ",",
            vec![ConnectionUri::default(), ConnectionUri::default()],
        ),
        (
            "postgres://jack:secret@myhost:5432/somedb?a=b&c=d#frag,postgres://",
            ",",
            vec![
                ConnectionUri {
                    authority: Some(Authority {
                        userspec: Some(UserSpec {
                            user: "jack".to_string(),
                            password: Some("secret".to_string()),
                        }),
                        hostspec: vec![HostSpec {
                            host: Some("myhost".parse().unwrap()),
                            port: Some(5432),
                        }],
                    }),
                    database: Some("somedb".to_string()),
                    parameters: Some(vec![
                        Parameter {
                            keyword: "a".to_string(),
                            value: "b".to_string(),
                        },
                        Parameter {
                            keyword: "c".to_string(),
                            value: "d".to_string(),
                        },
                    ]),
                    fragment: Some("frag".to_string()),
                },
                ConnectionUri::default(),
            ],
        ),
    ] {
        let res = multi_connection_string(input, sep);
        assert!(
            res.is_ok(),
            "expect Ok result: input: {:?}; sep: {:?}",
            input,
            sep
        );
        assert_eq!(res.unwrap(), output, "input: {}", input);
    }

    assert!(multi_connection_string(
        "postgres://jack:secret@myhost:5432/somedb?a=b&c=d#frag, ",
        ","
    )
    .is_err());
    assert!(multi_connection_string("postgres://,", ",").is_err());
    assert!(multi_connection_string("postgres://,postgres://,", ",").is_err());
}

#[test]
fn test_consuming_connection_string() {
    for (input, output) in vec![
        ("postgres://", ConnectionUri::default()),
        (
            "postgres://jack:secret@myhost:5432/somedb?a=b&c=d#frag",
            ConnectionUri {
                authority: Some(Authority {
                    userspec: Some(UserSpec {
                        user: "jack".to_string(),
                        password: Some("secret".to_string()),
                    }),
                    hostspec: vec![HostSpec {
                        host: Some("myhost".parse().unwrap()),
                        port: Some(5432),
                    }],
                }),
                database: Some("somedb".to_string()),
                parameters: Some(vec![
                    Parameter {
                        keyword: "a".to_string(),
                        value: "b".to_string(),
                    },
                    Parameter {
                        keyword: "c".to_string(),
                        value: "d".to_string(),
                    },
                ]),
                fragment: Some("frag".to_string()),
            },
        ),
    ] {
        assert_eq!(
            consuming_connection_string(input).unwrap(),
            output,
            "input: {}",
            input
        );
    }

    for input in vec![
        "postgres://,",
        "postgres://jack:secret@myhost:5432/somedb?a=b&c=d#frag another",
    ] {
        assert!(
            consuming_connection_string(input).is_err(),
            "input: {:?}",
            input
        );
    }
}

#[test]
fn test_connection_string() {
    for (input, rem, output) in vec![
        (
            "postgres://,",
            ",",
            ConnectionUri {
                authority: None,
                database: None,
                parameters: None,
                fragment: None,
            },
        ),
        (
            "postgres://",
            "",
            ConnectionUri {
                authority: None,
                database: None,
                parameters: None,
                fragment: None,
            },
        ),
        (
            "postgres://jack:secret@myhost:5432/somedb?a=b&c=d#frag another",
            "another",
            ConnectionUri {
                authority: Some(Authority {
                    userspec: Some(UserSpec {
                        user: "jack".to_string(),
                        password: Some("secret".to_string()),
                    }),
                    hostspec: vec![HostSpec {
                        host: Some("myhost".parse().unwrap()),
                        port: Some(5432),
                    }],
                }),
                database: Some("somedb".to_string()),
                parameters: Some(vec![
                    Parameter {
                        keyword: "a".to_string(),
                        value: "b".to_string(),
                    },
                    Parameter {
                        keyword: "c".to_string(),
                        value: "d".to_string(),
                    },
                ]),
                fragment: Some("frag".to_string()),
            },
        ),
        (
            "postgres://jack:secret@myhost:5432/somedb?a=b&c=d",
            "",
            ConnectionUri {
                authority: Some(Authority {
                    userspec: Some(UserSpec {
                        user: "jack".to_string(),
                        password: Some("secret".to_string()),
                    }),
                    hostspec: vec![HostSpec {
                        host: Some("myhost".parse().unwrap()),
                        port: Some(5432),
                    }],
                }),
                database: Some("somedb".to_string()),
                parameters: Some(vec![
                    Parameter {
                        keyword: "a".to_string(),
                        value: "b".to_string(),
                    },
                    Parameter {
                        keyword: "c".to_string(),
                        value: "d".to_string(),
                    },
                ]),
                fragment: None,
            },
        ),
        (
            "postgres://jack:secret@myhost/somedb?a=b&c=d",
            "",
            ConnectionUri {
                authority: Some(Authority {
                    userspec: Some(UserSpec {
                        user: "jack".to_string(),
                        password: Some("secret".to_string()),
                    }),
                    hostspec: vec![HostSpec {
                        host: Some("myhost".parse().unwrap()),
                        port: None,
                    }],
                }),
                database: Some("somedb".to_string()),
                parameters: Some(vec![
                    Parameter {
                        keyword: "a".to_string(),
                        value: "b".to_string(),
                    },
                    Parameter {
                        keyword: "c".to_string(),
                        value: "d".to_string(),
                    },
                ]),
                fragment: None,
            },
        ),
    ] {
        assert_eq!(
            connection_string(input).unwrap(),
            (rem, output),
            "input: {}",
            input
        );
    }
    // Valid connection string, with no query params specified.
    assert_eq!(
        connection_string("postgres://jack:secret@myhost:5432/somedb"),
        Ok((
            "",
            ConnectionUri {
                authority: Some(Authority {
                    userspec: Some(UserSpec {
                        user: "jack".to_string(),
                        password: Some("secret".to_string())
                    }),
                    hostspec: vec![HostSpec {
                        host: Some("myhost".parse().unwrap()),
                        port: Some(5432)
                    }],
                }),
                database: Some("somedb".to_string()),
                parameters: None,
                fragment: None,
            }
        ))
    );
    // Valid connection string, with no query params specified, but a
    // dangling question mark.
    assert_eq!(
        connection_string("postgres://jack:secret@myhost:5432/somedb?"),
        Ok((
            "",
            ConnectionUri {
                authority: Some(Authority {
                    userspec: Some(UserSpec {
                        user: "jack".to_string(),
                        password: Some("secret".to_string())
                    }),
                    hostspec: vec![HostSpec {
                        host: Some("myhost".parse().unwrap()),
                        port: Some(5432)
                    }],
                }),
                database: Some("somedb".to_string()),
                ..Default::default()
            }
        ))
    );
    // Valid connection string, with no database specified (with and without
    // trailing slash).
    assert_eq!(
        connection_string("postgres://jack:secret@myhost:5432"),
        Ok((
            "",
            ConnectionUri {
                authority: Some(Authority {
                    userspec: Some(UserSpec {
                        user: "jack".to_string(),
                        password: Some("secret".to_string())
                    }),
                    hostspec: vec![HostSpec {
                        host: Some("myhost".parse().unwrap()),
                        port: Some(5432)
                    }],
                }),
                ..Default::default()
            }
        ))
    );
    assert_eq!(
        connection_string("postgres://jack:secret@myhost:5432/"),
        Ok((
            "",
            ConnectionUri {
                authority: Some(Authority {
                    userspec: Some(UserSpec {
                        user: "jack".to_string(),
                        password: Some("secret".to_string())
                    }),
                    hostspec: vec![HostSpec {
                        host: Some("myhost".parse().unwrap()),
                        port: Some(5432)
                    }],
                }),
                ..Default::default()
            }
        ))
    );
    // Valid connection string, with no port specified.
    assert_eq!(
        connection_string("postgres://jack:secret@myhost"),
        Ok((
            "",
            ConnectionUri {
                authority: Some(Authority {
                    userspec: Some(UserSpec {
                        user: "jack".to_string(),
                        password: Some("secret".to_string())
                    }),
                    hostspec: vec![HostSpec {
                        host: Some("myhost".parse().unwrap()),
                        port: None,
                    }],
                }),
                ..Default::default()
            }
        ))
    );
    // Valid connection string, with only user and host specified.
    assert_eq!(
        connection_string("postgres://jack@myhost"),
        Ok((
            "",
            ConnectionUri {
                authority: Some(Authority {
                    userspec: Some(UserSpec {
                        user: "jack".to_string(),
                        password: None,
                    }),
                    hostspec: vec![HostSpec {
                        host: Some("myhost".parse().unwrap()),
                        port: None,
                    }],
                }),
                ..Default::default()
            }
        ))
    );
    // Trailing semicolon with no port is ignored.
    assert_eq!(
        connection_string("postgres://jack:secret@myhost:"),
        Ok((
            ":",
            ConnectionUri {
                authority: Some(Authority {
                    userspec: Some(UserSpec {
                        user: "jack".to_string(),
                        password: Some("secret".to_string()),
                    }),
                    hostspec: vec![HostSpec {
                        host: Some("myhost".parse().unwrap()),
                        port: None,
                    }],
                }),
                ..Default::default()
            }
        ))
    );

    // Doc tests to move back up to the docs section once the syntax is
    // cleaned up.
    assert_eq!(
        connection_string("postgres://"),
        Ok(("", ConnectionUri::default()))
    );
    assert_eq!(
        connection_string("postgres://localhost"),
        Ok((
            "",
            ConnectionUri {
                authority: Some(Authority {
                    userspec: None,
                    hostspec: vec![HostSpec {
                        host: Some("localhost".parse().unwrap()),
                        port: None,
                    }],
                }),
                ..Default::default()
            }
        ))
    );
    assert_eq!(
        connection_string("postgres://localhost:5433"),
        Ok((
            "",
            ConnectionUri {
                authority: Some(Authority {
                    userspec: None,
                    hostspec: vec![HostSpec {
                        host: Some("localhost".parse().unwrap()),
                        port: Some(5433),
                    }],
                }),
                ..Default::default()
            }
        ))
    );
    assert_eq!(
        connection_string("postgres://localhost/mydb"),
        Ok((
            "",
            ConnectionUri {
                authority: Some(Authority {
                    userspec: None,
                    hostspec: vec![HostSpec {
                        host: Some("localhost".parse().unwrap()),
                        port: None,
                    }],
                }),
                database: Some("mydb".to_string()),
                ..Default::default()
            }
        ))
    );
    assert_eq!(
        connection_string("postgres://user@localhost"),
        Ok((
            "",
            ConnectionUri {
                authority: Some(Authority {
                    userspec: Some(UserSpec {
                        user: "user".to_string(),
                        password: None
                    }),
                    hostspec: vec![HostSpec {
                        host: Some("localhost".parse().unwrap()),
                        port: None,
                    }],
                }),
                ..Default::default()
            }
        ))
    );
    // We don't support passing a password with no username.
    assert_eq!(
        connection_string("postgres://:user@localhost"),
        Ok((":user@localhost", ConnectionUri::default()))
    );
    assert_eq!(
        connection_string("postgres://user:secret@localhost"),
        Ok((
            "",
            ConnectionUri {
                authority: Some(Authority {
                    userspec: Some(UserSpec {
                        user: "user".to_string(),
                        password: Some("secret".to_string()),
                    }),
                    hostspec: vec![HostSpec {
                        host: Some("localhost".parse().unwrap()),
                        port: None,
                    }],
                }),
                ..Default::default()
            }
        ))
    );
    assert_eq!(
        connection_string(
            "postgresql://other@localhost/otherdb?connect_timeout=10&application_name=myapp"
        ),
        Ok((
            "",
            ConnectionUri {
                authority: Some(Authority {
                    userspec: Some(UserSpec {
                        user: "other".to_string(),
                        password: None
                    }),
                    hostspec: vec![HostSpec {
                        host: Some("localhost".parse().unwrap()),
                        port: None,
                    }],
                }),
                database: Some("otherdb".to_string()),
                parameters: Some(vec![
                    Parameter {
                        keyword: "connect_timeout".to_string(),
                        value: "10".to_string(),
                    },
                    Parameter {
                        keyword: "application_name".to_string(),
                        value: "myapp".to_string(),
                    },
                ]),
                ..Default::default()
            }
        ))
    );

    assert_eq!(
            connection_string(
                "postgresql://host1:123,host2:456/somedb?target_session_attrs=any&application_name=myapp"
            ),
            Ok((
                "",
                ConnectionUri {
                    authority: Some(Authority {
userspec: None,
                        hostspec: vec![HostSpec {
                            host: Some("host1".parse().unwrap()),
                            port: Some(123),
                        },HostSpec {
                            host: Some("host2".parse().unwrap()),
                            port: Some(456),
                        }],
                    }),
                    database: Some("somedb".to_string()),
                            parameters: Some(vec![
                                Parameter {
                                    keyword: "target_session_attrs".to_string(),
                                    value: "any".to_string(),
                                },
                                Parameter {
                                    keyword: "application_name".to_string(),
                                    value: "myapp".to_string(),
                                },
                            ]),
                    ..Default::default()
                }
            ))
        );
    assert_eq!(
        connection_string("postgresql://host1,host2"),
        Ok((
            "",
            ConnectionUri {
                authority: Some(Authority {
                    hostspec: vec![
                        HostSpec {
                            host: Some("host1".parse().unwrap()),
                            port: None,
                        },
                        HostSpec {
                            host: Some("host2".parse().unwrap()),
                            port: None,
                        }
                    ],
                    ..Default::default()
                }),
                ..Default::default()
            }
        ))
    );
    assert_eq!(
            connection_string(
                "postgresql://user:secret@host1:123,host2:456/somedb?target_session_attrs=any&application_name=myapp"
            ),
            Ok((
                "",
                ConnectionUri {
                    authority: Some(Authority {
                    userspec: Some(UserSpec {
                        user: "user".to_string(),
                        password: Some("secret".to_string()),
                    }),
                        hostspec: vec![HostSpec {
                            host: Some("host1".parse().unwrap()),
                            port: Some(123),
                        },HostSpec {
                            host: Some("host2".parse().unwrap()),
                            port: Some(456),
                        }],
                        }),
                    database: Some("somedb".to_string()),
                    parameters: Some(vec![
                        Parameter {
                            keyword: "target_session_attrs".to_string(),
                            value: "any".to_string(),
                        },
                        Parameter {
                            keyword: "application_name".to_string(),
                            value: "myapp".to_string(),
                        },
                    ]),
                    ..Default::default()
                }
            ))
        );
    assert_eq!(
        connection_string("postgresql://?host=myhost"),
        Ok((
            "",
            ConnectionUri {
                parameters: Some(vec![Parameter {
                    keyword: "host".to_string(),
                    value: "myhost".to_string(),
                },]),
                ..Default::default()
            }
        ))
    );
}

#[test]
fn test_scheme() {
    // Good cases.
    assert_eq!(scheme("postgres://"), Ok(("", "postgres")));
    assert_eq!(scheme("postgres://,"), Ok((",", "postgres")));
    assert_eq!(scheme("postgresql://"), Ok(("", "postgresql")));
    assert_eq!(scheme("postgresql://,"), Ok((",", "postgresql")));
    assert_eq!(scheme("postgres://foobar"), Ok(("foobar", "postgres")));
    assert_eq!(scheme("postgresql://foobaz"), Ok(("foobaz", "postgresql")));

    // Bad cases.
    assert!(scheme("postgres//").is_err());
    assert!(scheme("postgres:/").is_err());
    assert!(scheme("ostgresql://").is_err());
    assert!(scheme("postgre://").is_err());
}

#[test]
fn test_postgres() {
    assert_eq!(postgres("postgres"), Ok(("", "postgres")));
    assert_eq!(postgres("postgresql"), Ok(("ql", "postgres")));
    assert!(postgres("postgre").is_err());
    assert!(postgres("ostgres").is_err());
    assert!(postgres("postres").is_err());
}

#[test]
fn test_postgresql() {
    assert_eq!(postgresql("postgresql"), Ok(("", "postgresql")));
    assert_eq!(postgresql("postgresqlok"), Ok(("ok", "postgresql")));
    assert!(postgresql("postgresq").is_err());
    assert!(postgresql("ostgresql").is_err());
    assert!(postgresql("postresql").is_err());
}

#[test]
fn test_isomorphism() {
    // Expect parsing and then serializing to match the original input, except
    // for extra hosts that we hoist into the host list.
    for s in r#"postgresql://
postgresql://localhost
postgresql://localhost:5433
postgresql://localhost/mydb
postgresql://user@localhost
postgresql://user:secret@localhost
postgresql://other@localhost/otherdb?connect_timeout=10&application_name=myapp
postgresql://host1:123,host2:456/somedb?target_session_attrs=any&application_name=myapp
postgresql://user@localhost:5433/mydb?options=-c%20synchronous_commit%3Doff
postgresql://[2001:db8::1234]/database
postgresql:///dbname?host=/var/lib/postgresql
postgresql://%2Fvar%2Flib%2Fpostgresql/dbname
    "#
    .trim()
    .lines()
    {
        let res = ConnectionString::from_str(s).unwrap();
        let res = res.to_string();

        assert_eq!(res, s, "input: {s:?}");
    }
}

#[test]
fn test_formatting() {
    // Expect these sample URLs to be unchanged except for things like host
    // hoisting and trimming trailing slashes, etc.
    for (input, expect) in &[
        (
            "postgresql:///mydb?host=localhost&port=5433",
            "postgresql://localhost/mydb?port=5433",
        ),
        (
            "postgresql://host1:1,host2:2,host3:3/",
            "postgresql://host1:1,host2:2,host3:3",
        ),
    ] {
        let res = ConnectionString::from_str(input).unwrap();
        let res = res.to_string();

        assert_eq!(&res, expect, "input: {:?}", input);
    }
}
