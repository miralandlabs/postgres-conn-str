pub(crate) mod host;
#[cfg(test)]
mod tests;
pub(crate) mod userinfo;

use super::Res;
use host::{hostspecs, HostSpec};
use nom::{
    bytes::complete::tag,
    combinator::{fail, map, opt},
    sequence::{terminated, tuple},
};
use userinfo::{userinfo, UserSpec};

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Authority {
    pub userspec: Option<UserSpec>,
    pub hostspec: Vec<HostSpec>,
}

/// Parse the `authority` from the connection string. Unlike a standard URI,
/// this will accept a comma-separated list of host:port pairs.
pub(crate) fn authority(i: &str) -> Res<&str, Authority> {
    // We don't want to return an empty Vec of host:port pairs if there aren't any,
    // so we'll match on the result to replace an empty vec with None.
    //
    // TODO: Do this better directly in nom.
    match map(
        tuple((opt(terminated(userinfo, tag("@"))), hostspecs)),
        |(userspec, hostspec)| Authority { userspec, hostspec },
    )(i)
    {
        Ok((rem, auth)) if auth.userspec.is_none() && auth.hostspec.is_empty() => fail(rem),
        u => u,
    }
}
