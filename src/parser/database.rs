use super::Res;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, one_of},
    combinator::{opt, recognize},
    multi::many1,
    sequence::preceded,
};

/// Parse the `path` component from the URI.
pub(crate) fn database(i: &str) -> Res<&str, Option<String>> {
    // We use a match statement to ensure we return `None` for the path even if
    // there's a trailing slash, because that doesn't make sense as a postgres
    // database name.
    //
    // TODO: Handle this better directly in nom.
    match opt(preceded(
        tag("/"),
        opt(recognize(many1(alt((
            alphanumeric1,
            recognize(one_of("_-")),
        ))))),
    ))(i)
    {
        Ok((r, Some(Some(p)))) if p.is_empty() => Ok((r, None)),
        Ok((r, Some(Some(p)))) => Ok((r, Some(p.to_string()))),
        Ok((r, None | Some(None))) => Ok((r, None)),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database() {
        // Good cases.
        assert_eq!(database("/mypath"), Ok(("", Some("mypath".to_string()))));
        assert_eq!(
            database("/db?host=/tmp"),
            Ok(("?host=/tmp", Some("db".to_string())))
        );
        assert_eq!(database("/"), Ok(("", None)));
        assert_eq!(database(""), Ok(("", None)));
        assert_eq!(
            database("/mypath/again"),
            Ok(("/again", Some("mypath".to_string())))
        );
    }
}
