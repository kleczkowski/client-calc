# client-calc

`client-calc` reads a transaction CSV and writes one balance row per client.

## Run and test

```sh
cargo build
cargo run -- input.csv > output.csv
cargo test
```

The CLI takes one input path and writes the report to standard output. The
[`tests/fixtures`](tests/fixtures) directory contains a sample input and its
expected report. Tests cover balance changes, disputes, account freezes, and
invalid CSV rows.

## Processing rules

Each client has one account. The program processes CSV rows in file order and
keeps balances and booked transactions in memory. A later dispute can refer to
any earlier booked transaction by ID. Memory therefore grows with the number of
clients and booked transactions. The program writes the report after reading the
whole input successfully.

Withdrawals require enough available funds. Dispute actions apply only to the
client who owns the referenced transaction. Resolve and chargeback require an
open dispute. A chargeback freezes the account, and later transactions for that
client are ignored.

## Types and validation

`Client` and `TransactionId` are new-types. `TransactionKind` makes
wrong states irrepresentable (like having disputes with amounts).

`Amount` is a fixed-point decimal with four fractional digits. 
Balance arithmetic assumes that intermediate and final values fit in that range.

## License

This project is licensed under the [MIT license](LICENSE).
