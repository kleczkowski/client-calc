use anyhow::{Context, Result, bail};
use std::{env, fs::File, io, path::PathBuf};

fn main() -> Result<()> {
    let mut args = env::args_os().skip(1);
    let (Some(path), None) = (args.next(), args.next()) else {
        bail!("Usage: client-calc <transactions.csv>");
    };
    let path = PathBuf::from(path);
    let input = File::open(&path).with_context(|| format!("could not open {}", path.display()))?;
    client_calc::generate_report(input, io::stdout().lock())
}
