use std::iter::repeat;

use sqlite::Value;

use mithril_common::entities::{ImmutableFileNumber, TransactionHash};
use mithril_common::StdResult;
use mithril_persistence::sqlite::{
    Provider, SourceAlias, SqLiteEntity, SqliteConnection, WhereCondition,
};

use crate::database::record::CardanoTransactionRecord;

/// Simple queries to retrieve [CardanoTransaction] from the sqlite database.
pub(crate) struct CardanoTransactionProvider<'client> {
    connection: &'client SqliteConnection,
}

impl<'client> CardanoTransactionProvider<'client> {
    /// Create a new instance
    pub fn new(connection: &'client SqliteConnection) -> Self {
        Self { connection }
    }

    // Useful in test and probably in the future.
    pub(crate) fn get_transaction_hash_condition(
        &self,
        transaction_hash: &TransactionHash,
    ) -> WhereCondition {
        WhereCondition::new(
            "transaction_hash = ?*",
            vec![Value::String(transaction_hash.to_owned())],
        )
    }

    pub(crate) fn get_transaction_up_to_beacon_condition(
        &self,
        beacon: ImmutableFileNumber,
    ) -> WhereCondition {
        WhereCondition::new(
            "immutable_file_number <= ?*",
            vec![Value::Integer(beacon as i64)],
        )
    }
}

impl<'client> Provider<'client> for CardanoTransactionProvider<'client> {
    type Entity = CardanoTransactionRecord;

    fn get_connection(&'client self) -> &'client SqliteConnection {
        self.connection
    }

    fn get_definition(&self, condition: &str) -> String {
        let aliases = SourceAlias::new(&[("{:cardano_tx:}", "cardano_tx")]);
        let projection = Self::Entity::get_projection().expand(aliases);

        format!("select {projection} from cardano_tx where {condition} order by rowid")
    }
}

/// Query to insert [CardanoTransactionRecord] in the sqlite database
pub(crate) struct InsertCardanoTransactionProvider<'client> {
    connection: &'client SqliteConnection,
}

impl<'client> InsertCardanoTransactionProvider<'client> {
    /// Create a new instance
    pub fn new(connection: &'client SqliteConnection) -> Self {
        Self { connection }
    }

    /// Condition to insert one record.
    pub fn get_insert_condition(
        &self,
        record: &CardanoTransactionRecord,
    ) -> StdResult<WhereCondition> {
        self.get_insert_many_condition(vec![record.clone()])
    }

    /// Condition to insert multiples records.
    pub fn get_insert_many_condition(
        &self,
        transactions_records: Vec<CardanoTransactionRecord>,
    ) -> StdResult<WhereCondition> {
        let columns =
            "(transaction_hash, block_number, slot_number, block_hash, immutable_file_number)";
        let values_columns: Vec<&str> = repeat("(?*, ?*, ?*, ?*, ?*)")
            .take(transactions_records.len())
            .collect();

        let values: StdResult<Vec<Value>> =
            transactions_records
                .into_iter()
                .try_fold(vec![], |mut vec, record| {
                    vec.append(&mut vec![
                        Value::String(record.transaction_hash),
                        Value::Integer(record.block_number.try_into()?),
                        Value::Integer(record.slot_number.try_into()?),
                        Value::String(record.block_hash.clone()),
                        Value::Integer(record.immutable_file_number.try_into()?),
                    ]);
                    Ok(vec)
                });

        Ok(WhereCondition::new(
            format!("{columns} values {}", values_columns.join(", ")).as_str(),
            values?,
        ))
    }
}

impl<'client> Provider<'client> for InsertCardanoTransactionProvider<'client> {
    type Entity = CardanoTransactionRecord;

    fn get_connection(&'client self) -> &'client SqliteConnection {
        self.connection
    }

    fn get_definition(&self, condition: &str) -> String {
        let aliases = SourceAlias::new(&[("{:cardano_tx:}", "cardano_tx")]);
        let projection = Self::Entity::get_projection().expand(aliases);

        format!("insert or ignore into cardano_tx {condition} returning {projection}")
    }
}
