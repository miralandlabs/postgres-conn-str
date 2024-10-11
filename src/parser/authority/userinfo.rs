use super::Res;
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    character::complete::{alphanumeric1, one_of},
    combinator::{map, opt, recognize},
    multi::many1,
    sequence::{preceded, tuple},
};

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct UserSpec {
    pub user: String,
    pub password: Option<String>,
}

/// Parse the `userinfo` component of a URI.
pub(crate) fn userinfo(i: &str) -> Res<&str, UserSpec> {
    map(tuple((user, opt(password))), |(user, password)| UserSpec {
        user: user.into(),
        password: password.map(std::convert::Into::into),
    })(i)
}

/// Parse the `user` component of the `userinfo component of a uri.
fn user(i: &str) -> Res<&str, &str> {
    recognize(many1(alt((alphanumeric1, recognize(one_of("-_"))))))(i)
}

/// Parse the `password` component of the `userinfo component of a uri.
fn password(i: &str) -> Res<&str, &str> {
    preceded(tag(":"), is_not("@"))(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_userinfo() {
        // Good cases.
        assert_eq!(
            userinfo("jack:secret"),
            Ok((
                "",
                UserSpec {
                    user: "jack".to_string(),
                    password: Some("secret".to_string())
                }
            ))
        );
        assert_eq!(
            userinfo("jack:secret@"),
            Ok((
                "@",
                UserSpec {
                    user: "jack".to_string(),
                    password: Some("secret".to_string())
                }
            ))
        );
        assert_eq!(
            userinfo("jack"),
            Ok((
                "",
                UserSpec {
                    user: "jack".to_string(),
                    password: None
                }
            ))
        );
        assert_eq!(
            userinfo("jack@"),
            Ok((
                "@",
                UserSpec {
                    user: "jack".to_string(),
                    password: None
                }
            ))
        );
        assert_eq!(
            userinfo("jack:"),
            Ok((
                ":",
                UserSpec {
                    user: "jack".to_string(),
                    password: None
                }
            ))
        );

        for input in vec![",jack"] {
            assert!(userinfo(input).is_err(), "input: {:?}", input);
        }
    }

    #[test]
    fn test_password() {
        // Good cases.
        assert_eq!(password(":secret@"), Ok(("@", "secret")));

        for input in vec!["secret", "secret@", ":@", ":"] {
            assert!(password(input).is_err(), "input: {}", input);
        }
    }

    #[test]
    fn test_user() {
        // Good cases.
        assert_eq!(user("jack:"), Ok((":", "jack")));
        assert_eq!(user("jack"), Ok(("", "jack")));

        // Bad cases.
        assert!(user(":jack").is_err());
        assert!(user("@jack").is_err());
    }
}
