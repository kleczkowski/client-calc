use primitive_fixed_point_decimal::ConstScaleFpdec;
use serde::{Deserialize, Deserializer, Serialize, de};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "TransactionRow", into = "TransactionRow")]
pub struct Transaction {
    pub kind: TransactionKind,
    pub client: Client,
    pub tx: TransactionId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TransactionKind {
    Deposit(Amount),
    Withdrawal(Amount),
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Serialize, Deserialize)]
struct TransactionRow {
    #[serde(rename = "type")]
    kind: TransactionRowKind,
    client: Client,
    tx: TransactionId,
    #[serde(default, deserialize_with = "deserialize_amount")]
    amount: Option<Amount>,
}

fn deserialize_amount<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Amount>, D::Error> {
    Option::<String>::deserialize(deserializer)?
        .map(|amount| amount.parse().map_err(de::Error::custom))
        .transpose()
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum TransactionRowKind {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

impl TryFrom<TransactionRow> for Transaction {
    type Error = &'static str;

    fn try_from(row: TransactionRow) -> Result<Self, Self::Error> {
        let required_amount = || {
            let amount = row.amount.ok_or("missing amount")?;
            if amount < Amount::ZERO {
                return Err("negative amount");
            }
            Ok(amount)
        };
        let kind = match row.kind {
            TransactionRowKind::Deposit => TransactionKind::Deposit(required_amount()?),
            TransactionRowKind::Withdrawal => TransactionKind::Withdrawal(required_amount()?),
            TransactionRowKind::Dispute => TransactionKind::Dispute,
            TransactionRowKind::Resolve => TransactionKind::Resolve,
            TransactionRowKind::Chargeback => TransactionKind::Chargeback,
        };
        Ok(Self {
            kind,
            client: row.client,
            tx: row.tx,
        })
    }
}

impl From<Transaction> for TransactionRow {
    fn from(transaction: Transaction) -> Self {
        let (kind, amount) = match transaction.kind {
            TransactionKind::Deposit(amount) => (TransactionRowKind::Deposit, Some(amount)),
            TransactionKind::Withdrawal(amount) => (TransactionRowKind::Withdrawal, Some(amount)),
            TransactionKind::Dispute => (TransactionRowKind::Dispute, None),
            TransactionKind::Resolve => (TransactionRowKind::Resolve, None),
            TransactionKind::Chargeback => (TransactionRowKind::Chargeback, None),
        };
        Self {
            kind,
            client: transaction.client,
            tx: transaction.tx,
            amount,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Client(u16);

#[cfg(test)]
impl Client {
    pub(crate) fn new(id: u16) -> Self {
        Self(id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TransactionId(u32);

#[cfg(test)]
impl TransactionId {
    pub(crate) fn new(id: u32) -> Self {
        Self(id)
    }
}

pub type Amount = ConstScaleFpdec<i128, 4>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ClientReport {
    pub client: Client,
    pub available: Amount,
    pub held: Amount,
    pub total: Amount,
    pub locked: bool,
}
