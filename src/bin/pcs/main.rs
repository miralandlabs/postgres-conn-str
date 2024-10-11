use anyhow::Result;
use clap::Parser;
use rand::seq::SliceRandom;
use tracing::{debug, instrument};

/// Parse one or more connection strings from an input string.
///
/// env!("CARGO_BIN_NAME") accepts either a single connection string, or a character-separated list
/// of strings (defaults to comma-separated).
#[derive(Debug, Default, Parser)]
struct Config {
    /// Increase log verbosity. Defaults to warn.
    ///
    /// To decrease logging verbosity, set the RUST_LOG=error.
    #[clap(short, long, parse(from_occurrences))]
    verbose: usize,

    /// Character used to separate connection strings.
    #[clap(short, long, default_value = ",")]
    separator: String,

    /// Limit the number of returned connection strings.
    ///
    /// Combine with --randomize to select a random connection string.
    #[clap(short, long)]
    limit: Option<usize>,

    /// Return connection string(s) in random order.
    ///
    /// Combine with --limit to select a single random connection string.
    #[clap(short, long)]
    randomize: bool,

    connection_string: String,
}

#[instrument]
fn main() -> Result<()> {
    // Parse the config from input args/environment.
    let config = Config::parse();

    // Set up logging. Defaults to WARN, increasing from there.
    jacklog::from_level(2 + config.verbose, None)?;

    debug!(connection_string = ?config.connection_string);
    let mut parsed =
        pg_connection_string::from_multi_str(&config.connection_string, &config.separator)?;
    debug!(?parsed);

    if config.randomize {
        let mut rng = rand::thread_rng();
        parsed.shuffle(&mut rng);
    }

    if let Some(limit) = config.limit {
        for (n, conn) in parsed.iter().enumerate() {
            if n >= limit {
                break;
            }

            println!("{:#?}", &conn);
        }

        return Ok(());
    }

    println!("{:#?}", &parsed);

    Ok(())
}
