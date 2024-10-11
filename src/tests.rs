use std::path::PathBuf;

use super::*;

#[test]
fn test_from_str() {
    for (input, output) in vec![
        (
            "postgres://jack@besthost:34/mydb?host=foohost",
            ConnectionString {
                user: Some("jack".to_string()),
                password: None,
                hostspecs: vec![
                    HostSpec {
                        host: "besthost".parse().unwrap(),
                        port: Some(34),
                    },
                    HostSpec {
                        host: "foohost".parse().unwrap(),
                        port: None,
                    },
                ],
                database: Some("mydb".to_string()),
                parameters: vec![],
                fragment: None,
            },
        ),
        (
            "postgres://jack@besthost,otherhost:34/mydb?host=foohost&other=yes",
            ConnectionString {
                user: Some("jack".to_string()),
                password: None,
                hostspecs: vec![
                    HostSpec {
                        host: "besthost".parse().unwrap(),
                        port: None,
                    },
                    HostSpec {
                        host: "otherhost".parse().unwrap(),
                        port: Some(34),
                    },
                    HostSpec {
                        host: "foohost".parse().unwrap(),
                        port: None,
                    },
                ],
                database: Some("mydb".to_string()),
                parameters: vec![Parameter {
                    keyword: "other".to_string(),
                    value: "yes".to_string(),
                }],
                fragment: None,
            },
        ),
        (
            "postgres://jack@/postgres?host=/tmp/ephesock",
            ConnectionString {
                user: Some("jack".into()),
                password: None,
                hostspecs: vec![HostSpec {
                    host: Host::Path(PathBuf::from("/tmp/ephesock")),
                    port: None,
                }],
                database: Some("postgres".into()),
                parameters: vec![],
                fragment: None,
            },
        ),
    ] {
        assert_eq!(
            ConnectionString::from_str(input).unwrap(),
            output,
            "input: {input:?}",
        );
    }

    assert!(
        ConnectionString::from_str("").is_err(),
        "expect an error with an empty string"
    );
}

#[test]
fn test_from_multi_str() {
    for (input, sep, output) in vec![(
            "postgres://jack@besthost:34/mydb?host=foohost",
            ",",
            vec![
                ConnectionString {
                    user: Some("jack".to_string()),
                    password: None,
                    hostspecs: vec![
                    HostSpec { host: "besthost".parse().unwrap(), port: Some(34)},
                    HostSpec { host: "foohost".parse().unwrap(), port: None}
                ],
                    database: Some("mydb".to_string()),
                    parameters: vec![],
                    fragment: None,
                },
            ],
        ),(
            "postgres://jack@besthost:34/mydb?host=foohost,postgresql://jack@otherhost:543/mydb?hoster=foohost",
            ",",
            vec![
                ConnectionString {
                    user: Some("jack".to_string()),
                    password: None,
                    hostspecs: vec![
                        HostSpec { host: "besthost".parse().unwrap(), port: Some(34)},
                        HostSpec { host: "foohost".parse().unwrap(), port: None},
                    ],
                    database: Some("mydb".to_string()),
                    parameters: vec![],
                    fragment: None,
                },
                ConnectionString {
                    user: Some("jack".to_string()),
                    password: None,
                    hostspecs: vec![HostSpec { host: "otherhost".parse().unwrap(), port: Some(543)}],
                    database: Some("mydb".to_string()),
                    parameters: vec![
                        Parameter {
                            keyword: "hoster".into(),
                            value: "foohost".into()}
                    ],
                    fragment: None,
                }
            ],
        )] {
            assert_eq!(
                from_multi_str(input, sep).unwrap(),
                output,
                "input: {input:?}; sep: {sep:?}",
            );
        }
}
