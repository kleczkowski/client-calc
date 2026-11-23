use crate::model::{ClientReport, Transaction};
use anyhow::ensure;
use std::io::{Read, Write};

pub fn read_transactions<R: Read>(
    reader: R,
) -> anyhow::Result<impl Iterator<Item = anyhow::Result<Transaction>>> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(reader);

    let headers = reader.headers()?.clone();
    Ok(reader
        .into_records()
        .enumerate()
        .map(move |(index, record)| {
            let row = index + 1;
            let record = record?;
            ensure!(
                record.len() <= headers.len(),
                "too many fields in transaction row {row}"
            );
            Ok(record.deserialize(Some(&headers))?)
        }))
}

pub fn write_report<W: Write>(report: &[ClientReport], writer: W) -> anyhow::Result<()> {
    let mut writer = csv::WriterBuilder::new()
        .has_headers(true)
        .from_writer(writer);

    report
        .iter()
        .try_for_each(|report| writer.serialize(report))?;

    writer.flush()?;
    anyhow::Ok(())
}
