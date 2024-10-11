/*!
If you'd like to go insane, go read [https://paquier.xyz/postgresql-2/postgres-10-multi-host-connstr](https://paquier.xyz/postgresql-2/postgres-10-multi-host-connstr).
*/

use super::Res;
use anyhow::anyhow;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, digit1, hex_digit1, one_of, u16},
    combinator::{map, map_res, opt, recognize},
    multi::{count, many0, many1, separated_list0},
    sequence::{delimited, pair, preceded, terminated, tuple},
    Finish,
};
use std::{
    fmt::Display,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    path::PathBuf,
    str::FromStr,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct HostSpec {
    pub host: Option<Host>,
    pub port: Option<u16>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Host {
    Path(PathBuf),
    Name(String),
    Ip(IpAddr),
}

impl Display for Host {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Host::Path(path) => {
                write!(f, "{}", path.to_str().unwrap_or("<invalid>"))
            }
            Host::Name(name) => write!(f, "{name}"),
            Host::Ip(IpAddr::V4(ip)) => write!(f, "{ip}"),
            Host::Ip(IpAddr::V6(ip)) => write!(f, "[{ip}]"),
        }
    }
}

impl FromStr for Host {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        host(s)
            .finish()
            .map(|(_, host)| host)
            .map_err(|e| anyhow!(e.to_string()))
    }
}

/// Get a list of host:port pairs.
pub(crate) fn hostspecs(i: &str) -> Res<&str, Vec<HostSpec>> {
    // We deconstruct the result a bit to remove any pairs that are both None.
    many0(hostspec)(i)
}

/// Get a list of host:port pairs.
pub(crate) fn hostspec(i: &str) -> Res<&str, HostSpec> {
    // We deconstruct the result a bit to remove any pairs that are both None.
    terminated(
        alt((
            map(pair(host, port), |(host, port)| HostSpec {
                host: Some(host),
                port: Some(port),
            }),
            map(pair(host, opt(port)), |(host, port)| HostSpec {
                host: Some(host),
                port,
            }),
            map(pair(opt(host), port), |(host, port)| HostSpec {
                host: host.map(std::convert::Into::into),
                port: Some(port),
            }),
        )),
        opt(tag(",")),
    )(i)
}

/// Parse the `port` component of a URI.
fn port(i: &str) -> Res<&str, u16> {
    preceded(tag(":"), u16)(i)
}

/// Parse the `host` component of a URI.
fn host(i: &str) -> Res<&str, Host> {
    alt((
        map(ipv4, |ip| Host::Ip(IpAddr::V4(ip))),
        map(ipv6, |ip| Host::Ip(IpAddr::V6(ip))),
        map(
            recognize(many1(alt((alphanumeric1, recognize(one_of("._-")))))),
            |s: &str| Host::Name(s.to_string()),
        ),
    ))(i)
}

/// Recognize an ipv6 address.
fn ipv6(i: &str) -> Res<&str, Ipv6Addr> {
    delimited(
        tag("["),
        map_res(
            recognize(tuple((
                separated_list0(tag(":"), hex_digit1),
                tag("::"),
                hex_digit1,
            ))),
            Ipv6Addr::from_str,
        ),
        tag("]"),
    )(i)
}

/// Recognize an ipv4 address.
fn ipv4(i: &str) -> Res<&str, Ipv4Addr> {
    map_res(
        recognize(pair(count(pair(digit1, tag(".")), 3), digit1)),
        Ipv4Addr::from_str,
    )(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port() {
        // Good cases.
        for (input, rem, output) in vec![(":123", "", 123)] {
            assert_eq!(
                port(input).unwrap(),
                (rem, output),
                "input: {:?}; rem: {:?}",
                input,
                rem
            );
        }

        for input in vec![",123", ",:123", ":port"] {
            assert!(port(input).is_err());
        }
    }

    #[test]
    fn test_host() {
        // Good cases.
        for (input, rem, output) in vec![
            ("myhost", "", Host::Name("myhost".into())),
            ("myhost.mydomain", "", Host::Name("myhost.mydomain".into())),
            (
                "subdomain.myhost.com",
                "",
                Host::Name("subdomain.myhost.com".into()),
            ),
            (
                "sub-domain.my_host.com",
                "",
                Host::Name("sub-domain.my_host.com".into()),
            ),
            (
                "sub-domain.my_host.com/db",
                "/db",
                Host::Name("sub-domain.my_host.com".into()),
            ),
            (
                "[2001:db8::1234]/database",
                "/database",
                Host::Ip(IpAddr::from_str("2001:db8::1234").unwrap()),
            ),
        ] {
            assert_eq!(
                host(input),
                Ok((rem, output)),
                "input: {input:?}; rem: {rem:?}",
            );
        }

        // Bad cases.
        for input in [",host", "/db"] {
            assert!(host(input).is_err());
        }
    }

    #[test]
    fn test_ipv6() {
        // Good cases.
        for (input, rem, output) in [(
            "[2001:db8::1234]",
            "",
            Ipv6Addr::from_str("2001:db8::1234").unwrap(),
        )] {
            assert_eq!(
                ipv6(input),
                Ok((rem, output)),
                "input: {input:?}; rem: {rem:?}",
            );
        }
    }

    #[test]
    fn test_ipv4() {
        // Good cases.
        for (input, rem, output) in vec![("192.168.0.1", "", Ipv4Addr::new(192, 168, 0, 1))] {
            assert_eq!(
                ipv4(input).unwrap(),
                (rem, output),
                "input: {:?}; rem: {:?}",
                input,
                rem
            );
        }
    }

    macro_rules! host {
        ($s:expr) => {
            Some(host($s).unwrap().1)
        };
    }

    #[test]
    fn test_hostspec() {
        // Good cases.
        for (input, rem, output) in vec![(
            "myhost:123",
            "",
            HostSpec {
                host: host!("myhost"),
                port: Some(123),
            },
        )] {
            assert_eq!(
                hostspec(input).unwrap(),
                (rem, output),
                "input: {input:?}; rem: {rem:?}",
            );
        }

        for input in &["", ",", ",host:123"] {
            assert!(hostspec(input).is_err());
        }
    }
    #[test]
    fn test_hostspecs() {
        // Good cases.
        for (input, rem, output) in vec![
            ("", "", vec![]),
            (", ", ", ", vec![]),
            (",host:123", ",host:123", vec![]),
            (
                "myhost:123",
                "",
                vec![HostSpec {
                    host: host!("myhost"),
                    port: Some(123),
                }],
            ),
            (
                "myhost:123,secondhost:65535",
                "",
                vec![
                    HostSpec {
                        host: host!("myhost"),
                        port: Some(123),
                    },
                    HostSpec {
                        host: host!("secondhost"),
                        port: Some(65535),
                    },
                ],
            ),
        ] {
            assert_eq!(
                hostspecs(input).unwrap(),
                (rem, output),
                "input: {input:?}; rem: {rem:?}",
            );
        }

        for _input in &[",", ",host:123"] {}
    }
}
