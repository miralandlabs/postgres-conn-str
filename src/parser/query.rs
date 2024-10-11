use super::Res;
use crate::Parameter;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, one_of},
    combinator::{map, opt, recognize},
    multi::{many1, separated_list0},
    sequence::{pair, preceded, separated_pair},
};

/// Parse the `query` component from the URI.
pub(crate) fn query(i: &str) -> Res<&str, Option<Vec<Parameter>>> {
    // We use a match statement to return None if the query list is empty.
    // This will happen when there's a dangling `?`, but no actual query params.
    //
    // TODO: Handle this more elegantly directly in nom.
    match opt(preceded(tag("?"), separated_list0(tag("&"), keyval)))(i) {
        // If it's an error, just pass it back up and out.
        Err(e) => Err(e),
        // If we found query params, but the list is empty (which will happen
        // with a trailing ? but no actual params), override it to just return
        // None.
        Ok((s, Some(q))) if q.is_empty() => Ok((s, None)),
        // Otherwise, let the response go through.
        Ok(r) => Ok(r),
    }
}

/// Parse a key=val pair from a list of query params.
fn keyval(i: &str) -> Res<&str, Parameter> {
    map(separated_pair(key, tag("="), value), |(keyword, value)| {
        Parameter {
            keyword: keyword.into(),
            value: value.into(),
        }
    })(i)
}

/// Parse a `key` tag from the query params.
fn key(i: &str) -> Res<&str, &str> {
    recognize(many1(alt((alphanumeric1, recognize(one_of("_-"))))))(i)
}

/// Parse a `value` tag from the query params.
fn value(i: &str) -> Res<&str, &str> {
    recognize(many1(alt((
        urlencoded1,
        alphanumeric1,
        recognize(one_of("_-/")),
    ))))(i)
}

fn urlencoded1(i: &str) -> Res<&str, &str> {
    preceded(tag("%"), recognize(pair(hex, hex)))(i)
}

fn hex(i: &str) -> Res<&str, &str> {
    recognize(one_of("0123456789abcdef0123456789ABCDEF"))(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value() {
        for (input, rem, expect, comment) in vec![
            (
                "%20synchronous_commit%3Doff",
                "",
                "%20synchronous_commit%3Doff",
                "escaped query values uppercase",
            ),
            (
                "%20synchronous_commit%3doff",
                "",
                "%20synchronous_commit%3doff",
                "escaped query values lowercase",
            ),
            (
                "-c%20synchronous_commit%3Doff",
                "",
                "-c%20synchronous_commit%3Doff",
                "option flags with escapes",
            ),
            ("/var/lib/postgresql", "", "/var/lib/postgresql", "paths"),
        ] {
            assert_eq!(
                value(input).unwrap(),
                (rem, expect.into()),
                "input: {:?} (testing {})",
                input,
                comment
            );
        }
    }

    #[test]
    fn test_query() {
        assert_eq!(
            query("?connect_timeout=10&application_name=myapp"),
            Ok((
                "",
                Some(vec![
                    Parameter {
                        keyword: "connect_timeout".to_string(),
                        value: "10".to_string()
                    },
                    Parameter {
                        keyword: "application_name".to_string(),
                        value: "myapp".to_string()
                    },
                ]),
            ))
        );
    }

    #[test]
    fn test_urlencoded1() {
        for (input, rem, expect) in vec![
            ("%20", "", "20".into()),
            ("%3D", "", "3D".into()),
            ("%3d", "", "3d".into()),
            ("%3D1", "1", "3D".into()),
        ] {
            assert_eq!(
                urlencoded1(input).unwrap(),
                (rem, expect),
                "input: {:?}",
                input,
            );
        }
    }
}
