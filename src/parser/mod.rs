//! Functions and structures for parsing a valid PostgreSQL URI into an AST.
//!
//! This is useful if you want to see the raw parts of a URI, or how
//! pg-connection-string is parsing it, but for most consumers the top-level
//! functions are a better entrypoint.
pub(crate) mod authority;
mod database;
mod query;
#[cfg(test)]
mod tests;

use crate::Parameter;
use anyhow::{format_err, Result};
use authority::{authority, Authority};
use database::database;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, one_of, space0},
    combinator::{all_consuming, map, opt, recognize},
    error::{context, VerboseError},
    multi::{many1, separated_list1},
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};
use query::query;
use tracing::debug;

type Res<I, O> = IResult<I, O, VerboseError<I>>;

/// Struct representing the AST for a Postgres URI.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct ConnectionUri {
    pub authority: Option<Authority>,
    pub database: Option<String>,
    pub parameters: Option<Vec<Parameter>>,
    pub fragment: Option<String>,
}

pub(crate) fn multi_connection_string(i: &str, sep: &str) -> Result<Vec<ConnectionUri>> {
    let (rem, res) = all_consuming(separated_list1(tag(sep), connection_string))(i)
        .map_err(|e| format_err!("error consuming multi connection string: {}", e))?;
    // eprintln!("input: {i:?}; rem: {rem:?}");
    debug!("input: {i:?}; rem: {rem:?}");

    Ok(res)
}

pub(crate) fn consuming_connection_string(i: &str) -> Result<ConnectionUri> {
    let (_, res) = all_consuming(connection_string)(i)
        .map_err(|e| format_err!("error consuming connection string: {}", e))?;

    Ok(res)
}

fn connection_string(i: &str) -> Res<&str, ConnectionUri> {
    map(
        preceded(
            context("scheme", scheme),
            tuple((
                opt(authority),
                context("database", database),
                context("query", query),
                opt(fragment),
            )),
        ),
        |(authority, database, parameters, fragment)| ConnectionUri {
            authority,
            database,
            parameters,
            fragment: fragment.map(std::convert::Into::into),
        },
    )(i)
}

/// Parse the `fragment` component from the URI.
fn fragment(i: &str) -> Res<&str, &str> {
    delimited(
        context("fragment #", tag("#")),
        context(
            "fragment body",
            recognize(many1(alt((alphanumeric1, recognize(one_of("-_")))))),
        ),
        space0,
    )(i)
}

/// Parse the scheme from the URI. This should always be either `postgres` or
/// `postgresql`.
fn scheme(i: &str) -> Res<&str, &str> {
    terminated(
        alt((postgresql, postgres)),
        context("scheme :// tag", tag("://")),
    )(i)
}

/// Recognize the tag `postgres`.
fn postgres(i: &str) -> Res<&str, &str> {
    tag("postgres")(i)
}

/// Recognize the tag `postgresql`.`test_from_st`
fn postgresql(i: &str) -> Res<&str, &str> {
    tag("postgresql")(i)
}
