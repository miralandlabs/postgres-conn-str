use super::*;

#[test]
#[allow(clippy::too_many_lines)]
fn test_authority() {
    // Good cases.
    for (input, rem, output) in vec![
        (
            "jack:secret@myhost:123",
            "",
            Authority {
                userspec: Some(UserSpec {
                    user: "jack".to_string(),
                    password: Some("secret".to_string()),
                }),
                hostspec: vec![HostSpec {
                    host: Some("myhost".parse().unwrap()),
                    port: Some(123),
                }],
            },
        ),
        (
            "jack:secret@myhost:123/mydb",
            "/mydb",
            Authority {
                userspec: Some(UserSpec {
                    user: "jack".to_string(),
                    password: Some("secret".to_string()),
                }),
                hostspec: vec![HostSpec {
                    host: Some("myhost".parse().unwrap()),
                    port: Some(123),
                }],
            },
        ),
        (
            "jack:secret@myhost1:123,myhost2:456/mydb",
            "/mydb",
            Authority {
                userspec: Some(UserSpec {
                    user: "jack".to_string(),
                    password: Some("secret".to_string()),
                }),
                hostspec: vec![
                    HostSpec {
                        host: Some("myhost1".parse().unwrap()),
                        port: Some(123),
                    },
                    HostSpec {
                        host: Some("myhost2".parse().unwrap()),
                        port: Some(456),
                    },
                ],
            },
        ),
        (
            "jack:secret@myhost",
            "",
            Authority {
                userspec: Some(UserSpec {
                    user: "jack".to_string(),
                    password: Some("secret".to_string()),
                }),
                hostspec: vec![HostSpec {
                    host: Some("myhost".parse().unwrap()),
                    port: None,
                }],
            },
        ),
        (
            "jack@myhost",
            "",
            Authority {
                userspec: Some(UserSpec {
                    user: "jack".to_string(),
                    password: None,
                }),
                hostspec: vec![HostSpec {
                    host: Some("myhost".parse().unwrap()),
                    port: None,
                }],
            },
        ),
        (
            "myhost",
            "",
            Authority {
                userspec: None,
                hostspec: vec![HostSpec {
                    host: Some("myhost".parse().unwrap()),
                    port: None,
                }],
            },
        ),
        (
            "jack@/mydb",
            "/mydb",
            Authority {
                userspec: Some(UserSpec {
                    user: "jack".to_string(),
                    password: None,
                }),
                hostspec: vec![],
            },
        ),
    ] {
        assert_eq!(authority(input), Ok((rem, output)), "input: {input:?}",);
    }

    for input in ["", ",jack@myhost", "/db"] {
        assert!(authority(input).is_err(), "input: {input:?}");
    }
}
