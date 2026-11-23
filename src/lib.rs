use std::io::{Read, Write};

mod model;
mod processor;
mod text;

pub fn generate_report<R: Read, W: Write>(input: R, output: W) -> anyhow::Result<()> {
    let transactions = text::read_transactions(input)?;
    let report = processor::process_transactions(transactions)?;
    text::write_report(&report, output)?;
    Ok(())
}
