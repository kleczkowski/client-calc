use crate::model::{Amount, Client, ClientReport, Transaction, TransactionId, TransactionKind};
use std::collections::{HashMap, hash_map::Entry};

#[derive(Debug, Default)]
struct Status {
    available: Amount,
    held: Amount,
    locked: bool,
}

struct RecordedTransaction {
    client: Client,
    amount: Amount,
    disputed: bool,
}

pub fn process_transactions<E>(
    transactions: impl IntoIterator<Item = Result<Transaction, E>>,
) -> Result<Vec<ClientReport>, E> {
    let mut accounts: HashMap<Client, Status> = HashMap::new();
    let mut recorded: HashMap<TransactionId, RecordedTransaction> = HashMap::new();

    for transaction in transactions {
        let transaction = transaction?;
        let status = accounts.entry(transaction.client).or_default();
        if status.locked {
            continue;
        }

        match transaction.kind {
            TransactionKind::Deposit(amount) => {
                if let Entry::Vacant(entry) = recorded.entry(transaction.tx) {
                    status.available += amount;
                    entry.insert(RecordedTransaction {
                        client: transaction.client,
                        amount,
                        disputed: false,
                    });
                }
            }
            TransactionKind::Withdrawal(amount) => {
                if status.available >= amount
                    && let Entry::Vacant(entry) = recorded.entry(transaction.tx)
                {
                    status.available -= amount;
                    entry.insert(RecordedTransaction {
                        client: transaction.client,
                        amount,
                        disputed: false,
                    });
                }
            }
            TransactionKind::Dispute => {
                if let Some(original) = recorded.get_mut(&transaction.tx)
                    && original.client == transaction.client
                    && !original.disputed
                {
                    status.available -= original.amount;
                    status.held += original.amount;
                    original.disputed = true;
                }
            }
            TransactionKind::Resolve => {
                if let Some(original) = recorded.get_mut(&transaction.tx)
                    && original.client == transaction.client
                    && original.disputed
                {
                    status.held -= original.amount;
                    status.available += original.amount;
                    original.disputed = false;
                }
            }
            TransactionKind::Chargeback => {
                if let Some(original) = recorded.get_mut(&transaction.tx)
                    && original.client == transaction.client
                    && original.disputed
                {
                    status.held -= original.amount;
                    original.disputed = false;
                    status.locked = true;
                }
            }
        }
    }

    Ok(accounts
        .into_iter()
        .map(|(client, status)| ClientReport {
            client,
            available: status.available,
            held: status.held,
            total: status.available + status.held,
            locked: status.locked,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use crate::model::{Amount, Client, ClientReport, Transaction, TransactionId, TransactionKind};
    use TransactionKind::{Chargeback, Deposit, Dispute, Resolve, Withdrawal};
    use std::convert::Infallible;

    fn process_transactions(transactions: &[Transaction]) -> Vec<ClientReport> {
        super::process_transactions(transactions.iter().copied().map(Ok::<_, Infallible>)).unwrap()
    }

    fn amount(units: i64) -> Amount {
        Amount::from_mantissa(i128::from(units))
    }

    fn transaction(client: u16, tx: u32, kind: TransactionKind) -> Transaction {
        Transaction {
            client: Client::new(client),
            tx: TransactionId::new(tx),
            kind,
        }
    }

    fn balance(client: u16, total: i64, held: i64, locked: bool) -> ClientReport {
        ClientReport {
            client: Client::new(client),
            available: amount(total - held),
            held: amount(held),
            total: amount(total),
            locked,
        }
    }

    #[test]
    fn example_deposits_and_withdrawals() {
        let transactions = [
            transaction(1, 1, Deposit(amount(10_000))),
            transaction(2, 2, Deposit(amount(20_000))),
            transaction(1, 3, Deposit(amount(20_000))),
            transaction(1, 4, Withdrawal(amount(15_000))),
            transaction(2, 5, Withdrawal(amount(30_000))),
        ];

        let mut actual = process_transactions(&transactions);
        actual.sort_unstable_by_key(|row| row.client);

        assert_eq!(
            actual,
            vec![balance(1, 15_000, 0, false), balance(2, 20_000, 0, false)]
        );
    }

    #[test]
    fn empty_log_has_no_accounts() {
        assert!(process_transactions(&[]).is_empty());
    }

    #[test]
    fn withdrawal_uses_available_funds_and_failed_withdrawal_cannot_be_disputed() {
        let transactions = [
            transaction(1, 1, Deposit(amount(10_000))),
            transaction(1, 2, Deposit(amount(20_000))),
            transaction(1, 1, Dispute),
            transaction(1, 3, Withdrawal(amount(20_000))),
            transaction(1, 4, Withdrawal(amount(1))),
            transaction(1, 4, Dispute),
            transaction(1, 4, Chargeback),
        ];

        assert_eq!(
            process_transactions(&transactions),
            vec![balance(1, 10_000, 10_000, false)]
        );
    }

    #[test]
    fn missing_future_and_undisputed_references_are_ignored() {
        let transactions = [
            transaction(1, 99, Dispute),
            transaction(1, 99, Resolve),
            transaction(1, 99, Chargeback),
            transaction(1, 1, Dispute),
            transaction(1, 1, Deposit(amount(30_000))),
            transaction(1, 1, Resolve),
            transaction(1, 1, Chargeback),
        ];

        assert_eq!(
            process_transactions(&transactions),
            vec![balance(1, 30_000, 0, false)]
        );
    }

    #[test]
    fn dispute_actions_cannot_change_another_clients_transactions() {
        let transactions = [
            transaction(1, 1, Deposit(amount(10_000))),
            transaction(1, 2, Deposit(amount(20_000))),
            transaction(2, 3, Deposit(amount(5_000))),
            transaction(1, 1, Dispute),
            transaction(2, 2, Dispute),
            transaction(2, 1, Resolve),
            transaction(2, 1, Chargeback),
        ];

        let mut actual = process_transactions(&transactions);
        actual.sort_unstable_by_key(|row| row.client);

        assert_eq!(
            actual,
            vec![
                balance(1, 30_000, 10_000, false),
                balance(2, 5_000, 0, false)
            ]
        );
    }

    #[test]
    fn resolve_releases_a_hold_once_and_allows_a_new_dispute() {
        let transactions = [
            transaction(1, 1, Deposit(amount(10_000))),
            transaction(1, 1, Dispute),
            transaction(1, 1, Dispute),
            transaction(1, 1, Resolve),
            transaction(1, 1, Resolve),
            transaction(1, 1, Dispute),
        ];

        assert_eq!(
            process_transactions(&transactions[..3]),
            vec![balance(1, 10_000, 10_000, false)]
        );
        assert_eq!(
            process_transactions(&transactions[..5]),
            vec![balance(1, 10_000, 0, false)]
        );
        assert_eq!(
            process_transactions(&transactions),
            vec![balance(1, 10_000, 10_000, false)]
        );
    }

    #[test]
    fn chargeback_removes_only_its_hold_and_freezes_only_its_client() {
        let transactions = [
            transaction(1, 1, Deposit(amount(10_000))),
            transaction(1, 2, Deposit(amount(20_000))),
            transaction(1, 3, Deposit(amount(5_000))),
            transaction(1, 1, Dispute),
            transaction(1, 2, Dispute),
            transaction(1, 1, Chargeback),
            transaction(1, 1, Chargeback),
            transaction(1, 2, Resolve),
            transaction(1, 4, Deposit(amount(10_000))),
            transaction(1, 5, Withdrawal(amount(1))),
            transaction(2, 6, Deposit(amount(20_000))),
            transaction(2, 7, Withdrawal(amount(5_000))),
        ];

        let mut actual = process_transactions(&transactions);
        actual.sort_unstable_by_key(|row| row.client);

        assert_eq!(
            actual,
            vec![
                balance(1, 25_000, 20_000, true),
                balance(2, 15_000, 0, false)
            ]
        );
    }

    #[test]
    fn chargeback_of_spent_deposit_can_make_total_negative() {
        let transactions = [
            transaction(1, 1, Deposit(amount(10_000))),
            transaction(1, 2, Withdrawal(amount(10_000))),
            transaction(1, 1, Dispute),
            transaction(1, 1, Chargeback),
        ];

        assert_eq!(
            process_transactions(&transactions[..3]),
            vec![balance(1, 0, 10_000, false)]
        );
        assert_eq!(
            process_transactions(&transactions),
            vec![balance(1, -10_000, 0, true)]
        );
    }
}
