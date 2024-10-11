#![deny(unsafe_code)]

//! `pg-connection-string` parses URIs in the ways that psql (and generally,
//! libpq) will accept them. This is a more convenient and robust alternative to
//! crates like `uri`.
//!
//! As outlined in the
//! [Postgres docs](https://www.postgresql.org/docs/14/libpq-connect.html), the
//! general form for a connection URI is:
//!
//! ```txt
//! postgresql://[userspec@][hostspec][/dbname][?paramspec]
//!
//! where userspec is:
//!
//! user[:password]
//!
//! and hostspec is:
//!
//! [host][:port][,...]
//!
//! and paramspec is:
//!
//! name=value[&...]
//! ```
//!
//! The URI scheme designator can be either `postgresql://` or `postgres://`.
//! Each of the remaining URI parts is optional.

#[cfg(feature = "serde")]
mod de;
pub(crate) mod parser;
#[cfg(feature = "serde")]
mod ser;
#[cfg(test)]
mod tests;

use parser::{
    authority::{userinfo::UserSpec, Authority},
    ConnectionUri,
};
use std::{fmt::Display, path::PathBuf, str::FromStr};
use tracing::{debug, trace};

pub use parser::authority::host::Host;

/// A query parameter attached to the connection string.
///
/// This can be used to pass various configuration options to `libpq`, or to
/// override other parts of the URI.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Parameter {
    /// The key portion of the key=value pair.
    pub keyword: String,

    /// The value portion of the key=value pair.
    pub value: String,
}

/// Representation of the connection string.
///
/// This provides useful methods to access various parts of the connection
/// string, taking into account PostgreSQL's idiosyncrasies (such as being able
/// to pass most of the URI either in-place, or as query params).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ConnectionString {
    pub user: Option<String>,
    pub password: Option<String>,
    pub hostspecs: Vec<HostSpec>,
    pub database: Option<String>,
    pub parameters: Vec<Parameter>,
    pub fragment: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostSpec {
    pub host: Host,
    pub port: Option<u16>,
}

impl Display for ConnectionString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "postgresql://",)?;

        if let Some(user) = &self.user {
            write!(f, "{user}",)?;

            if let Some(password) = &self.password {
                write!(f, ":{password}",)?;
            }

            write!(f, "@")?;
        }

        for (n, HostSpec { host, port }) in self.hostspecs.iter().enumerate() {
            if let Host::Path(_) = host {
                continue;
            }

            write!(f, "{host}")?;

            if let Some(p) = port {
                write!(f, ":{p}")?;
            }

            if n + 1 < self.hostspecs.len() {
                write!(f, ",")?;
            }
        }

        if let Some(database) = &self.database {
            write!(f, "/{database}")?;
        }

        for (n, Parameter { keyword, value }) in self.parameters.iter().enumerate() {
            if n == 0 {
                write!(f, "?")?;
            }
            write!(f, "{keyword}={value}")?;

            if n + 1 < self.parameters.len() {
                write!(f, "&")?;
            }
        }

        // Write an host params to the end.
        for HostSpec { host, .. } in &self.hostspecs {
            match host {
                Host::Path(path) => {
                    write!(f, "{}", if self.parameters.is_empty() { "?" } else { "&" })?;

                    write!(f, "host={}", path.to_str().unwrap_or("invalid"))?;
                }
                _ => continue,
            }
        }

        if let Some(frag) = &self.fragment {
            write!(f, "#{frag}")?;
        }

        Ok(())
    }
}

impl TryFrom<ConnectionUri> for ConnectionString {
    type Error = anyhow::Error;

    fn try_from(mut uri: ConnectionUri) -> Result<Self, Self::Error> {
        // Extract any additional host from the query params before setting them
        // on the URIs.
        let mut addtl_hosts = vec![];
        if let Some(params) = &mut uri.parameters {
            if let Some(pos) = params.iter().position(|p| p.keyword == "host") {
                let param = params.remove(pos);

                addtl_hosts.push(param);
            }
        }

        // Set up a base object that has all the unchanging parameters about the
        // URL (properties that can't be specified multiple times).
        let mut out = ConnectionString {
            database: uri.database,
            parameters: uri.parameters.unwrap_or(vec![]),
            fragment: uri.fragment,
            ..ConnectionString::default()
        };
        trace!(?out, "populated unchanging pieces");

        // If there's an authority section, we'll add that to the output object.
        if let Some(Authority { userspec, hostspec }) = uri.authority {
            trace!(?userspec, ?hostspec, "found authority");

            // If a user/password were passed, set them.
            if let Some(UserSpec { user, password }) = userspec {
                trace!(?user, ?password, "found userspec");

                out.user = Some(user);
                out.password = password;
            }

            // If there's a hostspec, set that.
            for spec in hostspec {
                trace!(?spec, "adding hostspec");

                if let Some(host) = spec.host {
                    out.hostspecs.push(HostSpec {
                        host,
                        port: spec.port,
                    });
                }
            }
        }

        for Parameter { value, .. } in addtl_hosts {
            let host = if value.starts_with('/') {
                Host::Path(PathBuf::from(value))
            } else {
                value.parse()?
            };

            out.hostspecs.push(HostSpec { host, port: None });
        }

        Ok(out)
    }
}

/// Parse a PostgreSQL connection string.
impl FromStr for ConnectionString {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parsed = parser::consuming_connection_string(s)?;
        debug!(?parsed);

        parsed.try_into()
    }
}

/// Parse a delimited list of PostgreSQL connection strings into a list of
/// `ConnectionString`s.
///
/// This function can be useful if you want to take a string input list of
/// connection strings.
///
/// # Example
///
/// ```
/// use pg_connection_string::{HostSpec, from_multi_str, ConnectionString, Parameter};
///
/// assert_eq!(
///     from_multi_str("postgres://jack@besthost:34/mydb?host=foohost", ",").unwrap(),
///     [
///         ConnectionString {
///             user: Some("jack".to_string()),
///             password: None,
///             hostspecs: vec![
///                 HostSpec {
///                     host: "besthost".to_string(),
///                     port: Some(34),
///                 },
///                 HostSpec {
///                     host: "foohost".to_string(),
///                     port: None,
///                 },
///             ],
///             database: Some("mydb".to_string()),
///             parameters: vec![],
///             fragment: None,
///         },
///     ]
/// );
/// ```
pub fn from_multi_str(i: &str, sep: &str) -> anyhow::Result<Vec<ConnectionString>> {
    let parsed = parser::multi_connection_string(i, sep)?;
    // eprintln!("{parsed:?}");
    debug!("{parsed:?}");

    let mut out: Vec<ConnectionString> = vec![];

    for c in parsed {
        out.push(TryFrom::try_from(c)?);
    }

    Ok(out)
}
