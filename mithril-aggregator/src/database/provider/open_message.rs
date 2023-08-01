use mithril_common::{
    entities::{Epoch, ProtocolMessage, SignedEntityType, SingleSignatures},
    sqlite::{HydrationError, Projection, Provider, SourceAlias, SqLiteEntity, WhereCondition},
    StdResult,
};

use chrono::{DateTime, Utc};
use sqlite::{Connection, Row, Value};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

/// ## OpenMessage
///
/// An open message is a message open for signatures. Every signer may send a
/// single signature for this message from which a multi signature will be
/// generated if possible.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenMessageRecord {
    /// OpenMessage unique identifier
    pub open_message_id: Uuid,

    /// Epoch
    pub epoch: Epoch,

    /// Type of message
    pub signed_entity_type: SignedEntityType,

    /// Message used by the Mithril Protocol
    pub protocol_message: ProtocolMessage,

    /// Has this open message been converted into a certificate?
    pub is_certified: bool,

    /// Message creation datetime, it is set by the database.
    pub created_at: DateTime<Utc>,
}

impl OpenMessageRecord {
    #[cfg(test)]
    /// Create a dumb OpenMessage instance mainly for test purposes
    pub fn dummy() -> Self {
        let beacon = mithril_common::test_utils::fake_data::beacon();
        let epoch = beacon.epoch;
        let signed_entity_type = SignedEntityType::CardanoImmutableFilesFull(beacon);

        Self {
            open_message_id: Uuid::parse_str("193d1442-e89b-43cf-9519-04d8db9a12ff").unwrap(),
            epoch,
            signed_entity_type,
            protocol_message: ProtocolMessage::new(),
            is_certified: false,
            created_at: Utc::now(),
        }
    }
}

impl From<OpenMessageWithSingleSignaturesRecord> for OpenMessageRecord {
    fn from(value: OpenMessageWithSingleSignaturesRecord) -> Self {
        Self {
            open_message_id: value.open_message_id,
            epoch: value.epoch,
            signed_entity_type: value.signed_entity_type,
            protocol_message: value.protocol_message,
            is_certified: value.is_certified,
            created_at: value.created_at,
        }
    }
}

impl SqLiteEntity for OpenMessageRecord {
    fn hydrate(row: Row) -> Result<Self, HydrationError>
    where
        Self: Sized,
    {
        let open_message_id = row.read::<&str, _>(0);
        let open_message_id = Uuid::parse_str(open_message_id).map_err(|e| {
            HydrationError::InvalidData(format!(
                "Invalid UUID in open_message.open_message_id: '{open_message_id}'. Error: {e}"
            ))
        })?;
        let protocol_message = row.read::<&str, _>(4);
        let protocol_message = serde_json::from_str(protocol_message).map_err(|e| {
            HydrationError::InvalidData(format!(
                "Invalid protocol message JSON representation '{protocol_message}'. Error: {e}"
            ))
        })?;
        let epoch_setting_id = row.read::<i64, _>(1);
        let epoch_val = u64::try_from(epoch_setting_id)
            .map_err(|e| panic!("Integer field open_message.epoch_setting_id (value={epoch_setting_id}) is incompatible with u64 Epoch representation. Error = {e}"))?;

        // TODO: We need to check first that the cell can be read as a string first
        // (e.g. when beacon json is '{"network": "dev", "epoch": 1, "immutable_file_number": 2}').
        // If it fails, we fallback on readign the cell as an integer (e.g. when beacon json is '5').
        // Maybe there is a better way of doing this.
        let beacon_str = match row.try_read::<&str, _>(2) {
            Ok(value) => value.to_string(),
            Err(_) => (row.read::<i64, _>(2)).to_string(),
        };

        let signed_entity_type_id = usize::try_from(row.read::<i64, _>(3)).map_err(|e| {
            panic!(
                "Integer field open_message.signed_entity_type_id cannot be turned into usize: {e}"
            )
        })?;
        let signed_entity_type = SignedEntityType::hydrate(signed_entity_type_id, &beacon_str)?;
        let is_certified = row.read::<i64, _>(5) != 0;
        let datetime = &row.read::<&str, _>(6);
        let created_at =
            DateTime::parse_from_rfc3339(datetime).map_err(|e| {
                HydrationError::InvalidData(format!(
                    "Could not turn open_message.created_at field value '{datetime}' to rfc3339 Datetime. Error: {e}"
                ))
            })?.with_timezone(&Utc);

        let open_message = Self {
            open_message_id,
            epoch: Epoch(epoch_val),
            signed_entity_type,
            protocol_message,
            is_certified,
            created_at,
        };

        Ok(open_message)
    }

    fn get_projection() -> Projection {
        Projection::from(&[
            (
                "open_message_id",
                "{:open_message:}.open_message_id",
                "text",
            ),
            (
                "epoch_setting_id",
                "{:open_message:}.epoch_setting_id",
                "int",
            ),
            ("beacon", "{:open_message:}.beacon", "text"),
            (
                "signed_entity_type_id",
                "{:open_message:}.signed_entity_type_id",
                "int",
            ),
            (
                "protocol_message",
                "{:open_message:}.protocol_message",
                "text",
            ),
            ("is_certified", "{:open_message:}.is_certified", "bool"),
            ("created_at", "{:open_message:}.created_at", "text"),
        ])
    }
}

struct OpenMessageProvider<'client> {
    connection: &'client Connection,
}

impl<'client> OpenMessageProvider<'client> {
    pub fn new(connection: &'client Connection) -> Self {
        Self { connection }
    }

    fn get_epoch_condition(&self, epoch: Epoch) -> WhereCondition {
        WhereCondition::new("epoch_setting_id = ?*", vec![Value::Integer(*epoch as i64)])
    }

    fn get_signed_entity_type_condition(
        &self,
        signed_entity_type: &SignedEntityType,
    ) -> WhereCondition {
        WhereCondition::new(
            "signed_entity_type_id = ?* and beacon = ?*",
            vec![
                Value::Integer(signed_entity_type.index() as i64),
                // TODO!: Remove this ugly unwrap, should this method returns a result ?
                Value::String(signed_entity_type.get_json_beacon().unwrap()),
            ],
        )
    }

    // Useful in test and probably in the future.
    #[allow(dead_code)]
    fn get_open_message_id_condition(&self, open_message_id: &str) -> WhereCondition {
        WhereCondition::new(
            "open_message_id = ?*",
            vec![Value::String(open_message_id.to_owned())],
        )
    }
}

impl<'client> Provider<'client> for OpenMessageProvider<'client> {
    type Entity = OpenMessageRecord;

    fn get_connection(&'client self) -> &'client Connection {
        self.connection
    }

    fn get_definition(&self, condition: &str) -> String {
        let aliases = SourceAlias::new(&[
            ("{:open_message:}", "open_message"),
            ("{:single_signature:}", "single_signature"),
        ]);
        let projection = Self::Entity::get_projection().expand(aliases);

        format!("select {projection} from open_message where {condition} order by created_at desc")
    }
}

struct InsertOpenMessageProvider<'client> {
    connection: &'client Connection,
}

impl<'client> InsertOpenMessageProvider<'client> {
    pub fn new(connection: &'client Connection) -> Self {
        Self { connection }
    }

    fn get_insert_condition(
        &self,
        epoch: Epoch,
        signed_entity_type: &SignedEntityType,
        protocol_message: &ProtocolMessage,
    ) -> StdResult<WhereCondition> {
        let expression = "(open_message_id, epoch_setting_id, beacon, signed_entity_type_id, protocol_message, created_at) values (?*, ?*, ?*, ?*, ?*, ?*)";
        let beacon_str = signed_entity_type.get_json_beacon()?;
        let parameters = vec![
            Value::String(Uuid::new_v4().to_string()),
            Value::Integer(epoch.try_into()?),
            Value::String(beacon_str),
            Value::Integer(signed_entity_type.index() as i64),
            Value::String(serde_json::to_string(protocol_message)?),
            Value::String(Utc::now().to_rfc3339()),
        ];

        Ok(WhereCondition::new(expression, parameters))
    }
}

impl<'client> Provider<'client> for InsertOpenMessageProvider<'client> {
    type Entity = OpenMessageRecord;

    fn get_connection(&'client self) -> &'client Connection {
        self.connection
    }

    fn get_definition(&self, condition: &str) -> String {
        let aliases = SourceAlias::new(&[("{:open_message:}", "open_message")]);
        let projection = Self::Entity::get_projection().expand(aliases);

        format!("insert into open_message {condition} returning {projection}")
    }
}

struct UpdateOpenMessageProvider<'client> {
    connection: &'client Connection,
}
impl<'client> UpdateOpenMessageProvider<'client> {
    pub fn new(connection: &'client Connection) -> Self {
        Self { connection }
    }

    fn get_update_condition(&self, open_message: &OpenMessageRecord) -> StdResult<WhereCondition> {
        let expression = "epoch_setting_id = ?*, beacon = ?*, \
signed_entity_type_id = ?*, protocol_message = ?*, is_certified = ?* \
where open_message_id = ?*";
        let beacon_str = open_message.signed_entity_type.get_json_beacon()?;
        let parameters = vec![
            Value::Integer(open_message.epoch.try_into()?),
            Value::String(beacon_str),
            Value::Integer(open_message.signed_entity_type.index() as i64),
            Value::String(serde_json::to_string(&open_message.protocol_message)?),
            Value::Integer(open_message.is_certified as i64),
            Value::String(open_message.open_message_id.to_string()),
        ];

        Ok(WhereCondition::new(expression, parameters))
    }
}

impl<'client> Provider<'client> for UpdateOpenMessageProvider<'client> {
    type Entity = OpenMessageRecord;

    fn get_connection(&'client self) -> &'client Connection {
        self.connection
    }

    fn get_definition(&self, condition: &str) -> String {
        let aliases = SourceAlias::new(&[("{:open_message:}", "open_message")]);
        let projection = Self::Entity::get_projection().expand(aliases);

        format!("update open_message set {condition} returning {projection}")
    }
}

struct DeleteOpenMessageProvider<'client> {
    connection: &'client Connection,
}

impl<'client> DeleteOpenMessageProvider<'client> {
    pub fn new(connection: &'client Connection) -> Self {
        Self { connection }
    }

    fn get_epoch_condition(&self, epoch: Epoch) -> WhereCondition {
        WhereCondition::new("epoch_setting_id < ?*", vec![Value::Integer(*epoch as i64)])
    }
}

impl<'client> Provider<'client> for DeleteOpenMessageProvider<'client> {
    type Entity = OpenMessageRecord;

    fn get_connection(&'client self) -> &'client Connection {
        self.connection
    }

    fn get_definition(&self, condition: &str) -> String {
        let aliases = SourceAlias::new(&[("{:open_message:}", "open_message")]);
        let projection = Self::Entity::get_projection().expand(aliases);

        format!("delete from open_message where {condition} returning {projection}")
    }
}

/// Open Message with associated single signatures if any.
#[derive(Debug, Clone)]
pub struct OpenMessageWithSingleSignaturesRecord {
    /// OpenMessage unique identifier
    pub open_message_id: Uuid,

    /// Epoch
    pub epoch: Epoch,

    /// Type of message
    pub signed_entity_type: SignedEntityType,

    /// Message used by the Mithril Protocol
    pub protocol_message: ProtocolMessage,

    /// Has this message been converted into a Certificate?
    pub is_certified: bool,

    /// associated single signatures
    pub single_signatures: Vec<SingleSignatures>,

    /// Message creation datetime, it is set by the database.
    pub created_at: DateTime<Utc>,
}

impl SqLiteEntity for OpenMessageWithSingleSignaturesRecord {
    fn hydrate(row: Row) -> Result<Self, HydrationError>
    where
        Self: Sized,
    {
        let single_signatures = &row.read::<&str, _>(7);
        let single_signatures: Vec<SingleSignatures> = serde_json::from_str(single_signatures)
            .map_err(|e| {
                HydrationError::InvalidData(format!(
                    "Could not parse single signatures JSON: '{single_signatures}'. Error: {e}"
                ))
            })?;

        let open_message = OpenMessageRecord::hydrate(row)?;

        let open_message = Self {
            open_message_id: open_message.open_message_id,
            epoch: open_message.epoch,
            signed_entity_type: open_message.signed_entity_type,
            protocol_message: open_message.protocol_message,
            is_certified: open_message.is_certified,
            single_signatures,
            created_at: open_message.created_at,
        };

        Ok(open_message)
    }

    fn get_projection() -> Projection {
        Projection::from(&[
            (
                "open_message_id",
                "{:open_message:}.open_message_id",
                "text",
            ),
            (
                "epoch_setting_id",
                "{:open_message:}.epoch_setting_id",
                "int",
            ),
            ("beacon", "{:open_message:}.beacon", "text"),
            (
                "signed_entity_type_id",
                "{:open_message:}.signed_entity_type_id",
                "int",
            ),
            (
                "protocol_message",
                "{:open_message:}.protocol_message",
                "text",
            ),
            ("is_certified", "{:open_message:}.is_certified", "bool"),
            ("created_at", "{:open_message:}.created_at", "text"),
            (
                "single_signatures",
                "case when {:single_signature:}.signer_id is null then json('[]') \
else json_group_array( \
    json_object( \
        'party_id', {:single_signature:}.signer_id, \
        'signature', {:single_signature:}.signature, \
        'indexes', json({:single_signature:}.lottery_indexes) \
    ) \
) end",
                "text",
            ),
        ])
    }
}

struct OpenMessageWithSingleSignaturesProvider<'client> {
    connection: &'client Connection,
}

impl<'client> OpenMessageWithSingleSignaturesProvider<'client> {
    pub fn new(connection: &'client Connection) -> Self {
        Self { connection }
    }

    fn get_epoch_condition(&self, epoch: Epoch) -> WhereCondition {
        WhereCondition::new("epoch_setting_id = ?*", vec![Value::Integer(*epoch as i64)])
    }

    fn get_signed_entity_type_condition(
        &self,
        signed_entity_type: &SignedEntityType,
    ) -> WhereCondition {
        WhereCondition::new(
            "signed_entity_type_id = ?* and beacon = ?*",
            vec![
                Value::Integer(signed_entity_type.index() as i64),
                Value::String(signed_entity_type.get_json_beacon().unwrap()),
            ],
        )
    }
}

impl<'client> Provider<'client> for OpenMessageWithSingleSignaturesProvider<'client> {
    type Entity = OpenMessageWithSingleSignaturesRecord;

    fn get_connection(&'client self) -> &'client Connection {
        self.connection
    }

    fn get_definition(&self, condition: &str) -> String {
        let aliases = SourceAlias::new(&[
            ("{:open_message:}", "open_message"),
            ("{:single_signature:}", "single_signature"),
        ]);
        let projection = Self::Entity::get_projection().expand(aliases);

        format!(
            r#"
select {projection}
from open_message
    left outer join single_signature
        on open_message.open_message_id = single_signature.open_message_id 
where {condition}
group by open_message.open_message_id
order by open_message.created_at desc, open_message.rowid desc
"#
        )
    }
}

/// ## Open message repository
///
/// This is a business oriented layer to perform actions on the database through
/// providers.
pub struct OpenMessageRepository {
    connection: Arc<Mutex<Connection>>,
}

impl OpenMessageRepository {
    /// Instanciate service
    pub fn new(connection: Arc<Mutex<Connection>>) -> Self {
        Self { connection }
    }

    /// Return the latest [OpenMessageRecord] for the given Epoch and [SignedEntityType].
    pub async fn get_open_message(
        &self,
        signed_entity_type: &SignedEntityType,
    ) -> StdResult<Option<OpenMessageRecord>> {
        let lock = self.connection.lock().await;
        let provider = OpenMessageProvider::new(&lock);
        let filters = provider
            .get_epoch_condition(signed_entity_type.get_epoch())
            .and_where(provider.get_signed_entity_type_condition(signed_entity_type));
        let mut messages = provider.find(filters)?;

        Ok(messages.next())
    }

    /// Return an open message with its associated single signatures for the given Epoch and [SignedEntityType].
    pub async fn get_open_message_with_single_signatures(
        &self,
        signed_entity_type: &SignedEntityType,
    ) -> StdResult<Option<OpenMessageWithSingleSignaturesRecord>> {
        let lock = self.connection.lock().await;
        let provider = OpenMessageWithSingleSignaturesProvider::new(&lock);
        let filters = provider
            .get_epoch_condition(signed_entity_type.get_epoch())
            .and_where(provider.get_signed_entity_type_condition(signed_entity_type));
        let mut messages = provider.find(filters)?;

        Ok(messages.next())
    }

    /// Create a new [OpenMessageRecord] in the database.
    pub async fn create_open_message(
        &self,
        epoch: Epoch,
        signed_entity_type: &SignedEntityType,
        protocol_message: &ProtocolMessage,
    ) -> StdResult<OpenMessageRecord> {
        let lock = self.connection.lock().await;
        let provider = InsertOpenMessageProvider::new(&lock);
        let filters = provider.get_insert_condition(epoch, signed_entity_type, protocol_message)?;
        let mut cursor = provider.find(filters)?;

        cursor
            .next()
            .ok_or_else(|| panic!("Inserting an open_message should not return nothing."))
    }

    /// Updates an [OpenMessageRecord] in the database.
    pub async fn update_open_message(
        &self,
        open_message: &OpenMessageRecord,
    ) -> StdResult<OpenMessageRecord> {
        let lock = self.connection.lock().await;
        let provider = UpdateOpenMessageProvider::new(&lock);
        let filters = provider.get_update_condition(open_message)?;
        let mut cursor = provider.find(filters)?;

        cursor
            .next()
            .ok_or_else(|| panic!("Updating an open_message should not return nothing."))
    }

    /// Remove all the [OpenMessageRecord] for the strictly previous epochs of the given epoch in the database.
    /// It returns the number of messages removed.
    pub async fn clean_epoch(&self, epoch: Epoch) -> StdResult<usize> {
        let lock = self.connection.lock().await;
        let provider = DeleteOpenMessageProvider::new(&lock);
        let filters = provider.get_epoch_condition(epoch);
        let cursor = provider.find(filters)?;

        Ok(cursor.count())
    }
}

#[cfg(test)]
mod tests {
    use mithril_common::{entities::Beacon, sqlite::SourceAlias};

    use crate::database::provider::{
        apply_all_migrations_to_db, disable_foreign_key_support, SingleSignatureRecord,
    };
    use crate::{dependency_injection::DependenciesBuilder, Configuration};

    use crate::database::provider::test_helper::{
        insert_single_signatures_in_db, setup_single_signature_records,
    };

    use super::*;

    async fn get_connection() -> Arc<Mutex<Connection>> {
        let config = Configuration::new_sample();
        let mut builder = DependenciesBuilder::new(config);
        let connection = builder.get_sqlite_connection().await.unwrap();
        {
            let lock = connection.lock().await;
            lock.execute(
                r#"insert into epoch_setting(epoch_setting_id, protocol_parameters)
values (1, '{"k": 100, "m": 5, "phi": 0.65 }'), (2, '{"k": 100, "m": 5, "phi": 0.65 }');"#,
            )
            .unwrap();
        }

        connection
    }

    fn insert_golden_open_message_with_signature(connection: &Connection) {
        connection
            .execute(
                r#"
                insert into open_message values(
                    'd9498619-c12d-4379-ba76-c63035afd03c',
                    275,
                    275,
                    0,
                    '2023-07-27T00:02:44.505640275+00:00',
                    '{ "message_parts": {
                        "next_aggregate_verification_key":"7b226d745f636f6d6d69746d656e74223a7b22726f6f74223a5b3131312c3230352c3133392c3131322c32382c392c3233382c3134382c3133342c302c3230372c3233302c3234312c3130352c3135372c3131302c3232362c3131342c32362c35332c3136362c3235342c3230382c3132372c3231362c3230362c3230302c34382c35352c32312c3231372c31335d2c226e725f6c6561766573223a332c22686173686572223a6e756c6c7d2c22746f74616c5f7374616b65223a32383439323639303636317d"
                    }}',
                    1
                );

                insert into single_signature values(
                    'd9498619-c12d-4379-ba76-c63035afd03c',
                    'pool1r0tln8nct3mpyvehgy6uu3cdlmjnmtr2fxjcqnfl6v0qg0we42e',
                    274,
                    '[15,49,52,56,84,85,109,138,171,174,194,209,221,222,224,247,257,258,261,272,299,317,336,346,347,351,394,408,431,453,457,480,481,504,525,535,553,571,572,573,588,591,594,598,603,628,635,637,645,652,663,694,696,700,710,714,727,731,738,745,747,757,763,811,825,831,853,855,891,896,901,917,980,986,989,1010,1025,1077,1082,1092,1096,1140,1146,1147,1171,1192,1197,1246,1248,1261,1270,1277,1280,1290,1304,1349,1360,1363,1374,1409,1416,1418,1425,1432,1437,1440,1476,1481,1491,1499,1505,1527,1528,1544,1552,1559,1561,1571,1574,1596,1628,1638,1659,1680,1733,1736,1761,1782,1807,1868,1873,1877,1915,1923,1926,1927,1968,1969,1999,2078,2084,2125,2154,2156,2200,2207,2214,2231,2245,2247,2248,2280,2337,2364,2404,2442,2452,2461,2472,2484,2500,2511,2544,2565,2574,2578,2597,2600,2607,2611,2612,2620,2626,2638,2651,2668,2689,2698,2717,2764,2767,2843,2847,2855,2867,2869,2873,2906,2911,2918,2919,2921,2933,2936,2950,2953,2958,2960,2967,2973,2994,3002,3004,3030,3031,3037,3049,3056,3132,3136,3140,3194,3218,3240,3261,3283,3294,3298,3318,3320,3324,3358,3361,3368,3373,3378,3398,3415,3417,3428,3450,3458,3462,3467,3501,3512,3539,3545,3558,3568,3573,3581,3600,3618,3628,3634,3635,3638,3646,3667,3677,3698,3699,3701,3706,3708,3728,3741,3744,3748,3762,3771,3779,3792,3806,3807,3821,3823,3827,3828,3842,3849,3850,3854,3856,3861,3907,3925,3938,3942,3950,3985,3998,4015,4018,4021,4077,4092,4094,4103,4115,4165,4174,4188,4190,4199,4216,4220,4223,4252,4280,4314,4315,4338,4340,4353,4363,4367,4400,4403,4407,4419,4423,4427,4429,4450,4472,4486,4489,4497,4525,4537,4542,4550,4578,4598,4601,4613,4618,4621,4623,4640,4648,4656,4660,4661,4702,4710,4715,4737,4748,4753,4754,4766,4776,4779,4784,4794,4801,4803,4834,4854,4855,4861,4871,4873,4878,4887,4915,4920,4923,4945,4950,4951,4960,4962,4980,4993,4999,5028,5067,5068,5081,5091,5125,5129,5132,5133,5142,5176,5194,5223,5239,5256,5267,5292,5300,5337,5343,5354,5357,5366,5375,5376,5386,5405,5409,5416,5454,5457,5458,5465,5467,5471,5483,5490,5504,5540,5552,5565,5582,5617,5646,5659,5660,5666,5678,5685,5696,5706,5716,5722,5746,5752,5753,5760,5762,5782,5798,5799,5804,5810,5816,5817,5844,5857,5864,5873,5894,5970,5974,5994,6002,6006,6025,6026,6031,6047,6052,6065,6077,6084,6085,6098,6108,6115,6123,6137,6146,6171,6195,6206,6219,6229,6261,6263,6266,6274,6281,6301,6308,6312,6339,6360,6378,6422,6425,6449,6462,6477,6499,6508,6545,6546,6549,6551,6554,6563,6587,6589,6593,6599,6609,6610,6625,6636,6642,6644,6649,6653,6669,6673,6683,6697,6710,6712,6714,6717,6732,6766,6813,6864,6896,6908,6919,6943,6947,6965,6968,6969,6987,7000,7001,7022,7035,7037,7046,7047,7059,7074,7136,7146,7147,7161,7174,7191,7193,7221,7222,7225,7227,7255,7263,7281,7294,7313,7330,7349,7375,7387,7427,7442,7452,7466,7472,7482,7483,7488,7540,7586,7602,7624,7636,7657,7675,7678,7683,7691,7696,7713,7726,7737,7740,7781,7800,7809,7826,7827,7833,7836,7863,7868,7878,7886,7895,7923,7942,7945,7993,8007,8023,8029,8040,8051,8056,8079,8092,8094,8099,8120,8137,8152,8175,8191,8213,8219,8271,8280,8281,8293,8296,8300,8301,8304,8312,8326,8329,8336,8346,8347,8352,8363,8395,8397,8403,8405,8413,8426,8437,8441,8442,8458,8488,8519,8527,8534,8543,8643,8663,8669,8691,8730,8748,8756,8757,8760,8763,8772,8800,8806,8825,8837,8850,8853,8857,8864,8887,8903,8924,8970,8988,9015,9051,9084,9102,9111,9121,9122,9147,9171,9177,9178,9183,9194,9210,9246,9253,9266,9279,9292,9338,9339,9344,9348,9359,9374,9378,9404,9410,9418,9464,9468,9472,9479,9489,9494,9497,9549,9604,9613,9644,9663,9684,9686,9691,9696,9707,9717,9718,9773,9779,9794,9795,9796,9824,9871,9876,9881,9883,9886,9899,9920,9921,9922,9929,9930,9955,9956,9961,9982,9988,9991,10008,10025,10036,10038,10061,10064,10069,10070,10087,10090,10098,10119,10122,10124,10126,10139,10158,10164,10187,10203,10205,10242,10259,10269,10270,10285,10318,10324,10360,10381,10382,10407,10420,10438,10469,10481,10504,10508,10510,10590,10595,10608,10614,10626,10632,10662,10679,10685,10697,10705,10716,10719,10743,10790,10801,10815,10830,10844,10847,10856,10860,10877,10919,10930,10933,10938,10940,10942,10945,10950,10967,10985,10995,11021,11029,11032,11039,11131,11158,11170,11192,11205,11209,11220,11270,11283,11299,11328,11352,11358,11373,11376,11391,11421,11422,11431,11438,11449,11457,11474,11497,11506,11512,11542,11548,11563,11581,11591,11592,11593,11602,11657,11659,11673,11695,11706,11712,11717,11729,11744,11767,11777,11779,11793,11804,11824,11826,11843,11880,11884,11887,11924,11934,11936,11940,11966,11978,11989,11998,12026,12030,12037,12059,12063,12076,12087,12105,12145,12160,12161,12165,12170,12204,12236,12254,12258,12259,12303,12305,12313,12327,12334,12339,12355,12360,12367,12391,12415,12427,12463,12464,12532,12554,12568,12572,12595,12631,12637,12672,12678,12679,12701,12702,12705,12723,12725,12735,12753,12756,12776,12781,12805,12811,12831,12849,12855,12863,12873,12880,12885,12892,12896,12898,12904,12916,12932,12944,12946,12952,12953,12955,12965,12990,13002,13007,13047,13071,13079,13090,13102,13144,13159,13161,13173,13174,13188,13208,13216,13227,13246,13249,13268,13293,13296,13319,13323,13340,13349,13356,13378,13379,13388,13398,13432,13433,13467,13519,13524,13533,13566,13572,13596,13619,13641,13647,13656,13659,13671,13685,13693,13703,13752,13787,13793,13798,13801,13805,13807,13808,13820,13830,13841,13845,13857,13862,13870,13898,13908,13910,13935,13939,13942,13949,13952,13958,13968,13972,14003,14007,14037,14046,14051,14066,14075,14125,14127,14144,14149,14151,14163,14196,14202,14223,14243,14247,14248,14252,14255,14290,14293,14299,14362,14382,14392,14411,14429,14459,14467,14485,14502,14509,14540,14562,14570,14605,14619,14631,14640,14655,14681,14684,14698,14703,14704,14722,14735,14739,14765,14774,14814,14836,14842,14866,14873,14880,14884,14892,14897,14948,14966,14978,14984,14989,14999,15016,15025,15031,15041,15066,15079,15120,15124,15144,15173,15183,15186,15196,15212,15218,15230,15234,15244,15245,15254,15272,15273,15283,15291,15303,15320,15355,15369,15378,15403,15407,15412,15413,15426,15444,15498,15505,15509,15526,15528,15559,15564,15615,15619,15621,15626,15629,15650,15651,15663,15667,15685,15702,15712,15726,15733,15734,15743,15762,15794,15809,15820,15828,15872,15887,15889,15896,15897,15964,15982,15992,16002,16008,16020,16022,16023,16042,16054,16071,16082,16099,16116,16132,16140,16142,16164,16181,16196,16201,16204,16214,16230,16234,16235,16238,16239,16257,16275,16309,16364,16367,16379,16398,16423,16451,16454,16468,16471,16543,16547,16548,16557,16565,16571,16573,16580,16606,16613,16629,16636,16655,16656,16660,16679,16680,16685,16729,16735,16738,16739,16745,16767,16800,16810,16812,16850,16866,16893,16904,16927,16958,16961,16967,16975,16983,16989,17000,17060,17066,17088,17097,17102,17109,17114,17124,17139,17140,17144,17148,17167,17174,17195,17204,17220,17224,17234,17246,17251,17273,17292,17294,17305,17308,17314,17320,17321,17382,17388,17417,17427,17432,17445,17449,17465,17468,17473,17490,17502,17507,17519,17523,17577,17597,17610,17612,17622,17634,17639,17645,17671,17677,17713,17719,17728,17743,17748,17780,17795,17807,17808,17810,17817,17819,17825,17834,17837,17854,17882,17888,17895,17898,17899,17902,17929,17931,17933,17937,17938,17942,17952,17958,17959,17980,17986,17997,18004,18024,18026,18041,18043,18050,18076,18117,18122,18140,18154,18157,18163,18169,18178,18181,18191,18211,18214,18218,18247,18263,18265,18271,18274,18278,18320,18350,18360,18381,18397,18412,18428,18470,18476,18478,18484,18507,18522,18536,18546,18551,18552,18567,18573,18580,18582,18593,18602,18609,18616,18631,18632,18652,18665,18680,18685,18710,18721,18735,18745,18748,18759,18783,18786,18787,18790,18796,18802,18805,18810,18846,18858,18872,18873,18888,18905,18910,18919,18933,18936,18941,18944,18953,18981,18989,18999,19039,19077,19122,19153,19154,19156,19163,19169,19197,19198,19199,19229,19244,19245,19304,19306,19322,19343,19346,19348,19350,19352,19372,19379,19397,19405,19417,19452,19461,19468,19477,19499,19551,19574,19586,19595,19614,19635,19673,19683,19706,19718,19722,19764,19807,19849,19851,19885,19911,19937,19963,19964,19984,19987,19995,19996,20005,20010,20021,20053,20057,20095,20100,20101,20138,20143,20149,20151,20155,20159,20176,20186,20193,20195,20211,20215,20258,20270,20297,20305,20311,20336,20351,20370,20380,20390,20407,20413,20417,20439,20442,20444,20453,20455,20468,20498,20510,20518,20530,20532,20539,20552,20553,20584,20588,20600,20602,20638,20656,20675,20677,20693,20698,20728,20735,20762,20807,20808,20832,20847,20862,20894,20897,20903,20924,20938,20952,20960]',
                    '7b227369676d61223a5b3133392c3135332c36382c3133352c3134382c3138302c3133352c35392c3136302c3135302c3133302c3233362c3139332c3138392c3131382c3232342c3137382c3235322c3133312c3138382c32372c37362c3138332c3134322c3230342c34332c34362c3130342c3230372c36332c3135382c3137392c3231382c3135332c3232312c3233392c3234312c37322c3235342c362c3136302c3234382c3232332c3132382c3138322c3234372c3135342c3235325d2c22696e6465786573223a5b31352c34392c35322c35362c38342c38352c3130392c3133382c3137312c3137342c3139342c3230392c3232312c3232322c3232342c3234372c3235372c3235382c3236312c3237322c3239392c3331372c3333362c3334362c3334372c3335312c3339342c3430382c3433312c3435332c3435372c3438302c3438312c3530342c3532352c3533352c3535332c3537312c3537322c3537332c3538382c3539312c3539342c3539382c3630332c3632382c3633352c3633372c3634352c3635322c3636332c3639342c3639362c3730302c3731302c3731342c3732372c3733312c3733382c3734352c3734372c3735372c3736332c3831312c3832352c3833312c3835332c3835352c3839312c3839362c3930312c3931372c3938302c3938362c3938392c313031302c313032352c313037372c313038322c313039322c313039362c313134302c313134362c313134372c313137312c313139322c313139372c313234362c313234382c313236312c313237302c313237372c313238302c313239302c313330342c313334392c313336302c313336332c313337342c313430392c313431362c313431382c313432352c313433322c313433372c313434302c313437362c313438312c313439312c313439392c313530352c313532372c313532382c313534342c313535322c313535392c313536312c313537312c313537342c313539362c313632382c313633382c313635392c313638302c313733332c313733362c313736312c313738322c313830372c313836382c313837332c313837372c313931352c313932332c313932362c313932372c313936382c313936392c313939392c323037382c323038342c323132352c323135342c323135362c323230302c323230372c323231342c323233312c323234352c323234372c323234382c323238302c323333372c323336342c323430342c323434322c323435322c323436312c323437322c323438342c323530302c323531312c323534342c323536352c323537342c323537382c323539372c323630302c323630372c323631312c323631322c323632302c323632362c323633382c323635312c323636382c323638392c323639382c323731372c323736342c323736372c323834332c323834372c323835352c323836372c323836392c323837332c323930362c323931312c323931382c323931392c323932312c323933332c323933362c323935302c323935332c323935382c323936302c323936372c323937332c323939342c333030322c333030342c333033302c333033312c333033372c333034392c333035362c333133322c333133362c333134302c333139342c333231382c333234302c333236312c333238332c333239342c333239382c333331382c333332302c333332342c333335382c333336312c333336382c333337332c333337382c333339382c333431352c333431372c333432382c333435302c333435382c333436322c333436372c333530312c333531322c333533392c333534352c333535382c333536382c333537332c333538312c333630302c333631382c333632382c333633342c333633352c333633382c333634362c333636372c333637372c333639382c333639392c333730312c333730362c333730382c333732382c333734312c333734342c333734382c333736322c333737312c333737392c333739322c333830362c333830372c333832312c333832332c333832372c333832382c333834322c333834392c333835302c333835342c333835362c333836312c333930372c333932352c333933382c333934322c333935302c333938352c333939382c343031352c343031382c343032312c343037372c343039322c343039342c343130332c343131352c343136352c343137342c343138382c343139302c343139392c343231362c343232302c343232332c343235322c343238302c343331342c343331352c343333382c343334302c343335332c343336332c343336372c343430302c343430332c343430372c343431392c343432332c343432372c343432392c343435302c343437322c343438362c343438392c343439372c343532352c343533372c343534322c343535302c343537382c343539382c343630312c343631332c343631382c343632312c343632332c343634302c343634382c343635362c343636302c343636312c343730322c343731302c343731352c343733372c343734382c343735332c343735342c343736362c343737362c343737392c343738342c343739342c343830312c343830332c343833342c343835342c343835352c343836312c343837312c343837332c343837382c343838372c343931352c343932302c343932332c343934352c343935302c343935312c343936302c343936322c343938302c343939332c343939392c353032382c353036372c353036382c353038312c353039312c353132352c353132392c353133322c353133332c353134322c353137362c353139342c353232332c353233392c353235362c353236372c353239322c353330302c353333372c353334332c353335342c353335372c353336362c353337352c353337362c353338362c353430352c353430392c353431362c353435342c353435372c353435382c353436352c353436372c353437312c353438332c353439302c353530342c353534302c353535322c353536352c353538322c353631372c353634362c353635392c353636302c353636362c353637382c353638352c353639362c353730362c353731362c353732322c353734362c353735322c353735332c353736302c353736322c353738322c353739382c353739392c353830342c353831302c353831362c353831372c353834342c353835372c353836342c353837332c353839342c353937302c353937342c353939342c363030322c363030362c363032352c363032362c363033312c363034372c363035322c363036352c363037372c363038342c363038352c363039382c363130382c363131352c363132332c363133372c363134362c363137312c363139352c363230362c363231392c363232392c363236312c363236332c363236362c363237342c363238312c363330312c363330382c363331322c363333392c363336302c363337382c363432322c363432352c363434392c363436322c363437372c363439392c363530382c363534352c363534362c363534392c363535312c363535342c363536332c363538372c363538392c363539332c363539392c363630392c363631302c363632352c363633362c363634322c363634342c363634392c363635332c363636392c363637332c363638332c363639372c363731302c363731322c363731342c363731372c363733322c363736362c363831332c363836342c363839362c363930382c363931392c363934332c363934372c363936352c363936382c363936392c363938372c373030302c373030312c373032322c373033352c373033372c373034362c373034372c373035392c373037342c373133362c373134362c373134372c373136312c373137342c373139312c373139332c373232312c373232322c373232352c373232372c373235352c373236332c373238312c373239342c373331332c373333302c373334392c373337352c373338372c373432372c373434322c373435322c373436362c373437322c373438322c373438332c373438382c373534302c373538362c373630322c373632342c373633362c373635372c373637352c373637382c373638332c373639312c373639362c373731332c373732362c373733372c373734302c373738312c373830302c373830392c373832362c373832372c373833332c373833362c373836332c373836382c373837382c373838362c373839352c373932332c373934322c373934352c373939332c383030372c383032332c383032392c383034302c383035312c383035362c383037392c383039322c383039342c383039392c383132302c383133372c383135322c383137352c383139312c383231332c383231392c383237312c383238302c383238312c383239332c383239362c383330302c383330312c383330342c383331322c383332362c383332392c383333362c383334362c383334372c383335322c383336332c383339352c383339372c383430332c383430352c383431332c383432362c383433372c383434312c383434322c383435382c383438382c383531392c383532372c383533342c383534332c383634332c383636332c383636392c383639312c383733302c383734382c383735362c383735372c383736302c383736332c383737322c383830302c383830362c383832352c383833372c383835302c383835332c383835372c383836342c383838372c383930332c383932342c383937302c383938382c393031352c393035312c393038342c393130322c393131312c393132312c393132322c393134372c393137312c393137372c393137382c393138332c393139342c393231302c393234362c393235332c393236362c393237392c393239322c393333382c393333392c393334342c393334382c393335392c393337342c393337382c393430342c393431302c393431382c393436342c393436382c393437322c393437392c393438392c393439342c393439372c393534392c393630342c393631332c393634342c393636332c393638342c393638362c393639312c393639362c393730372c393731372c393731382c393737332c393737392c393739342c393739352c393739362c393832342c393837312c393837362c393838312c393838332c393838362c393839392c393932302c393932312c393932322c393932392c393933302c393935352c393935362c393936312c393938322c393938382c393939312c31303030382c31303032352c31303033362c31303033382c31303036312c31303036342c31303036392c31303037302c31303038372c31303039302c31303039382c31303131392c31303132322c31303132342c31303132362c31303133392c31303135382c31303136342c31303138372c31303230332c31303230352c31303234322c31303235392c31303236392c31303237302c31303238352c31303331382c31303332342c31303336302c31303338312c31303338322c31303430372c31303432302c31303433382c31303436392c31303438312c31303530342c31303530382c31303531302c31303539302c31303539352c31303630382c31303631342c31303632362c31303633322c31303636322c31303637392c31303638352c31303639372c31303730352c31303731362c31303731392c31303734332c31303739302c31303830312c31303831352c31303833302c31303834342c31303834372c31303835362c31303836302c31303837372c31303931392c31303933302c31303933332c31303933382c31303934302c31303934322c31303934352c31303935302c31303936372c31303938352c31303939352c31313032312c31313032392c31313033322c31313033392c31313133312c31313135382c31313137302c31313139322c31313230352c31313230392c31313232302c31313237302c31313238332c31313239392c31313332382c31313335322c31313335382c31313337332c31313337362c31313339312c31313432312c31313432322c31313433312c31313433382c31313434392c31313435372c31313437342c31313439372c31313530362c31313531322c31313534322c31313534382c31313536332c31313538312c31313539312c31313539322c31313539332c31313630322c31313635372c31313635392c31313637332c31313639352c31313730362c31313731322c31313731372c31313732392c31313734342c31313736372c31313737372c31313737392c31313739332c31313830342c31313832342c31313832362c31313834332c31313838302c31313838342c31313838372c31313932342c31313933342c31313933362c31313934302c31313936362c31313937382c31313938392c31313939382c31323032362c31323033302c31323033372c31323035392c31323036332c31323037362c31323038372c31323130352c31323134352c31323136302c31323136312c31323136352c31323137302c31323230342c31323233362c31323235342c31323235382c31323235392c31323330332c31323330352c31323331332c31323332372c31323333342c31323333392c31323335352c31323336302c31323336372c31323339312c31323431352c31323432372c31323436332c31323436342c31323533322c31323535342c31323536382c31323537322c31323539352c31323633312c31323633372c31323637322c31323637382c31323637392c31323730312c31323730322c31323730352c31323732332c31323732352c31323733352c31323735332c31323735362c31323737362c31323738312c31323830352c31323831312c31323833312c31323834392c31323835352c31323836332c31323837332c31323838302c31323838352c31323839322c31323839362c31323839382c31323930342c31323931362c31323933322c31323934342c31323934362c31323935322c31323935332c31323935352c31323936352c31323939302c31333030322c31333030372c31333034372c31333037312c31333037392c31333039302c31333130322c31333134342c31333135392c31333136312c31333137332c31333137342c31333138382c31333230382c31333231362c31333232372c31333234362c31333234392c31333236382c31333239332c31333239362c31333331392c31333332332c31333334302c31333334392c31333335362c31333337382c31333337392c31333338382c31333339382c31333433322c31333433332c31333436372c31333531392c31333532342c31333533332c31333536362c31333537322c31333539362c31333631392c31333634312c31333634372c31333635362c31333635392c31333637312c31333638352c31333639332c31333730332c31333735322c31333738372c31333739332c31333739382c31333830312c31333830352c31333830372c31333830382c31333832302c31333833302c31333834312c31333834352c31333835372c31333836322c31333837302c31333839382c31333930382c31333931302c31333933352c31333933392c31333934322c31333934392c31333935322c31333935382c31333936382c31333937322c31343030332c31343030372c31343033372c31343034362c31343035312c31343036362c31343037352c31343132352c31343132372c31343134342c31343134392c31343135312c31343136332c31343139362c31343230322c31343232332c31343234332c31343234372c31343234382c31343235322c31343235352c31343239302c31343239332c31343239392c31343336322c31343338322c31343339322c31343431312c31343432392c31343435392c31343436372c31343438352c31343530322c31343530392c31343534302c31343536322c31343537302c31343630352c31343631392c31343633312c31343634302c31343635352c31343638312c31343638342c31343639382c31343730332c31343730342c31343732322c31343733352c31343733392c31343736352c31343737342c31343831342c31343833362c31343834322c31343836362c31343837332c31343838302c31343838342c31343839322c31343839372c31343934382c31343936362c31343937382c31343938342c31343938392c31343939392c31353031362c31353032352c31353033312c31353034312c31353036362c31353037392c31353132302c31353132342c31353134342c31353137332c31353138332c31353138362c31353139362c31353231322c31353231382c31353233302c31353233342c31353234342c31353234352c31353235342c31353237322c31353237332c31353238332c31353239312c31353330332c31353332302c31353335352c31353336392c31353337382c31353430332c31353430372c31353431322c31353431332c31353432362c31353434342c31353439382c31353530352c31353530392c31353532362c31353532382c31353535392c31353536342c31353631352c31353631392c31353632312c31353632362c31353632392c31353635302c31353635312c31353636332c31353636372c31353638352c31353730322c31353731322c31353732362c31353733332c31353733342c31353734332c31353736322c31353739342c31353830392c31353832302c31353832382c31353837322c31353838372c31353838392c31353839362c31353839372c31353936342c31353938322c31353939322c31363030322c31363030382c31363032302c31363032322c31363032332c31363034322c31363035342c31363037312c31363038322c31363039392c31363131362c31363133322c31363134302c31363134322c31363136342c31363138312c31363139362c31363230312c31363230342c31363231342c31363233302c31363233342c31363233352c31363233382c31363233392c31363235372c31363237352c31363330392c31363336342c31363336372c31363337392c31363339382c31363432332c31363435312c31363435342c31363436382c31363437312c31363534332c31363534372c31363534382c31363535372c31363536352c31363537312c31363537332c31363538302c31363630362c31363631332c31363632392c31363633362c31363635352c31363635362c31363636302c31363637392c31363638302c31363638352c31363732392c31363733352c31363733382c31363733392c31363734352c31363736372c31363830302c31363831302c31363831322c31363835302c31363836362c31363839332c31363930342c31363932372c31363935382c31363936312c31363936372c31363937352c31363938332c31363938392c31373030302c31373036302c31373036362c31373038382c31373039372c31373130322c31373130392c31373131342c31373132342c31373133392c31373134302c31373134342c31373134382c31373136372c31373137342c31373139352c31373230342c31373232302c31373232342c31373233342c31373234362c31373235312c31373237332c31373239322c31373239342c31373330352c31373330382c31373331342c31373332302c31373332312c31373338322c31373338382c31373431372c31373432372c31373433322c31373434352c31373434392c31373436352c31373436382c31373437332c31373439302c31373530322c31373530372c31373531392c31373532332c31373537372c31373539372c31373631302c31373631322c31373632322c31373633342c31373633392c31373634352c31373637312c31373637372c31373731332c31373731392c31373732382c31373734332c31373734382c31373738302c31373739352c31373830372c31373830382c31373831302c31373831372c31373831392c31373832352c31373833342c31373833372c31373835342c31373838322c31373838382c31373839352c31373839382c31373839392c31373930322c31373932392c31373933312c31373933332c31373933372c31373933382c31373934322c31373935322c31373935382c31373935392c31373938302c31373938362c31373939372c31383030342c31383032342c31383032362c31383034312c31383034332c31383035302c31383037362c31383131372c31383132322c31383134302c31383135342c31383135372c31383136332c31383136392c31383137382c31383138312c31383139312c31383231312c31383231342c31383231382c31383234372c31383236332c31383236352c31383237312c31383237342c31383237382c31383332302c31383335302c31383336302c31383338312c31383339372c31383431322c31383432382c31383437302c31383437362c31383437382c31383438342c31383530372c31383532322c31383533362c31383534362c31383535312c31383535322c31383536372c31383537332c31383538302c31383538322c31383539332c31383630322c31383630392c31383631362c31383633312c31383633322c31383635322c31383636352c31383638302c31383638352c31383731302c31383732312c31383733352c31383734352c31383734382c31383735392c31383738332c31383738362c31383738372c31383739302c31383739362c31383830322c31383830352c31383831302c31383834362c31383835382c31383837322c31383837332c31383838382c31383930352c31383931302c31383931392c31383933332c31383933362c31383934312c31383934342c31383935332c31383938312c31383938392c31383939392c31393033392c31393037372c31393132322c31393135332c31393135342c31393135362c31393136332c31393136392c31393139372c31393139382c31393139392c31393232392c31393234342c31393234352c31393330342c31393330362c31393332322c31393334332c31393334362c31393334382c31393335302c31393335322c31393337322c31393337392c31393339372c31393430352c31393431372c31393435322c31393436312c31393436382c31393437372c31393439392c31393535312c31393537342c31393538362c31393539352c31393631342c31393633352c31393637332c31393638332c31393730362c31393731382c31393732322c31393736342c31393830372c31393834392c31393835312c31393838352c31393931312c31393933372c31393936332c31393936342c31393938342c31393938372c31393939352c31393939362c32303030352c32303031302c32303032312c32303035332c32303035372c32303039352c32303130302c32303130312c32303133382c32303134332c32303134392c32303135312c32303135352c32303135392c32303137362c32303138362c32303139332c32303139352c32303231312c32303231352c32303235382c32303237302c32303239372c32303330352c32303331312c32303333362c32303335312c32303337302c32303338302c32303339302c32303430372c32303431332c32303431372c32303433392c32303434322c32303434342c32303435332c32303435352c32303436382c32303439382c32303531302c32303531382c32303533302c32303533322c32303533392c32303535322c32303535332c32303538342c32303538382c32303630302c32303630322c32303633382c32303635362c32303637352c32303637372c32303639332c32303639382c32303732382c32303733352c32303736322c32303830372c32303830382c32303833322c32303834372c32303836322c32303839342c32303839372c32303930332c32303932342c32303933382c32303935322c32303936305d2c227369676e65725f696e646578223a327d',
                    '2023-07-27T00:06:20.710956040+00:00'
                );

            "#,
            )
            .unwrap();
    }

    #[tokio::test]
    async fn test_golden_master() {
        let connection = Connection::open(":memory:").unwrap();
        apply_all_migrations_to_db(&connection).unwrap();
        disable_foreign_key_support(&connection).unwrap();
        insert_golden_open_message_with_signature(&connection);

        let repository = OpenMessageRepository::new(Arc::new(Mutex::new(connection)));
        repository
            .get_open_message(&SignedEntityType::MithrilStakeDistribution(Epoch(275)))
            .await
            .expect("Getting Golden open message should not fail")
            .expect("A open message should exist for this signed entity type");

        repository
            .get_open_message_with_single_signatures(&SignedEntityType::MithrilStakeDistribution(
                Epoch(275),
            ))
            .await
            .expect("Getting Golden open message should not fail")
            .expect(
                "A open message with single signatures should exist for this signed entity type",
            );
    }

    #[test]
    fn open_message_with_single_signature_projection() {
        let projection = OpenMessageWithSingleSignaturesRecord::get_projection();
        let aliases = SourceAlias::new(&[
            ("{:open_message:}", "open_message"),
            ("{:single_signature:}", "single_signature"),
        ]);

        assert_eq!(
            "open_message.open_message_id as open_message_id, \
open_message.epoch_setting_id as epoch_setting_id, open_message.beacon as beacon, \
open_message.signed_entity_type_id as signed_entity_type_id, \
open_message.protocol_message as protocol_message, open_message.is_certified as is_certified, \
open_message.created_at as created_at, \
case when single_signature.signer_id is null then json('[]') \
else json_group_array( \
    json_object( \
        'party_id', single_signature.signer_id, \
        'signature', single_signature.signature, \
        'indexes', json(single_signature.lottery_indexes) \
    ) \
) end as single_signatures"
                .to_string(),
            projection.expand(aliases)
        )
    }

    #[test]
    fn open_message_projection() {
        let projection = OpenMessageRecord::get_projection();
        let aliases = SourceAlias::new(&[("{:open_message:}", "open_message")]);

        assert_eq!(
            "open_message.open_message_id as open_message_id, open_message.epoch_setting_id as epoch_setting_id, open_message.beacon as beacon, open_message.signed_entity_type_id as signed_entity_type_id, open_message.protocol_message as protocol_message, open_message.is_certified as is_certified, open_message.created_at as created_at".to_string(),
            projection.expand(aliases)
        )
    }

    #[test]
    fn provider_epoch_condition() {
        let connection = Connection::open(":memory:").unwrap();
        let provider = OpenMessageProvider::new(&connection);
        let (expr, params) = provider.get_epoch_condition(Epoch(12)).expand();

        assert_eq!("epoch_setting_id = ?1".to_string(), expr);
        assert_eq!(vec![Value::Integer(12)], params,);
    }

    #[test]
    fn provider_message_type_condition() {
        let connection = Connection::open(":memory:").unwrap();
        let provider = OpenMessageProvider::new(&connection);
        let beacon = Beacon {
            network: "whatever".to_string(),
            epoch: Epoch(4),
            immutable_file_number: 400,
        };
        let (expr, params) = provider
            .get_signed_entity_type_condition(&SignedEntityType::CardanoImmutableFilesFull(
                beacon.clone(),
            ))
            .expand();

        assert_eq!(
            "signed_entity_type_id = ?1 and beacon = ?2".to_string(),
            expr
        );
        assert_eq!(
            vec![
                Value::Integer(2),
                Value::String(serde_json::to_string(&beacon).unwrap())
            ],
            params,
        );
    }

    #[test]
    fn provider_message_id_condition() {
        let connection = Connection::open(":memory:").unwrap();
        let provider = OpenMessageProvider::new(&connection);
        let (expr, params) = provider
            .get_open_message_id_condition("cecd7983-8b3a-42b1-b778-6d75e87828ee")
            .expand();

        assert_eq!("open_message_id = ?1".to_string(), expr);
        assert_eq!(
            vec![Value::String(
                "cecd7983-8b3a-42b1-b778-6d75e87828ee".to_string()
            )],
            params,
        );
    }

    #[test]
    fn insert_provider_condition() {
        let connection = Connection::open(":memory:").unwrap();
        let provider = InsertOpenMessageProvider::new(&connection);
        let epoch = Epoch(12);
        let (expr, params) = provider
            .get_insert_condition(
                epoch,
                &SignedEntityType::CardanoImmutableFilesFull(Beacon::new(
                    "testnet".to_string(),
                    2,
                    4,
                )),
                &ProtocolMessage::new(),
            )
            .unwrap()
            .expand();

        assert_eq!("(open_message_id, epoch_setting_id, beacon, signed_entity_type_id, protocol_message, created_at) values (?1, ?2, ?3, ?4, ?5, ?6)".to_string(), expr);
        assert_eq!(Value::Integer(12), params[1]);
        assert_eq!(
            Value::String(
                r#"{"network":"testnet","epoch":2,"immutable_file_number":4}"#.to_string()
            ),
            params[2]
        );
        assert_eq!(Value::Integer(2), params[3]);
        assert_eq!(
            Value::String(serde_json::to_string(&ProtocolMessage::new()).unwrap()),
            params[4]
        );
    }

    #[test]
    fn update_provider_condition() {
        let connection = Connection::open(":memory:").unwrap();
        let provider = UpdateOpenMessageProvider::new(&connection);
        let open_message = OpenMessageRecord {
            open_message_id: Uuid::new_v4(),
            epoch: Epoch(12),
            signed_entity_type: SignedEntityType::dummy(),
            protocol_message: ProtocolMessage::new(),
            is_certified: true,
            created_at: DateTime::<Utc>::default(),
        };
        let (expr, params) = provider
            .get_update_condition(&open_message)
            .unwrap()
            .expand();

        assert_eq!(
            "epoch_setting_id = ?1, beacon = ?2, signed_entity_type_id = ?3, protocol_message = ?4, is_certified = ?5 where open_message_id = ?6"
                .to_string(),
            expr
        );
        assert_eq!(
            vec![
                Value::Integer(*open_message.epoch as i64),
                Value::String(open_message.signed_entity_type.get_json_beacon().unwrap()),
                Value::Integer(open_message.signed_entity_type.index() as i64),
                Value::String(serde_json::to_string(&open_message.protocol_message).unwrap()),
                Value::Integer(open_message.is_certified as i64),
                Value::String(open_message.open_message_id.to_string()),
            ],
            params
        );
    }

    #[test]
    fn delete_provider_epoch_condition() {
        let connection = Connection::open(":memory:").unwrap();
        let provider = DeleteOpenMessageProvider::new(&connection);
        let (expr, params) = provider.get_epoch_condition(Epoch(12)).expand();

        assert_eq!("epoch_setting_id < ?1".to_string(), expr);
        assert_eq!(vec![Value::Integer(12)], params,);
    }

    #[tokio::test]
    async fn repository_get_open_message() {
        let connection = get_connection().await;
        let repository = OpenMessageRepository::new(connection.clone());
        let beacon = Beacon::new("devnet".to_string(), 1, 1);

        let signed_entity_type = SignedEntityType::MithrilStakeDistribution(beacon.epoch);
        repository
            .create_open_message(beacon.epoch, &signed_entity_type, &ProtocolMessage::new())
            .await
            .unwrap();
        let open_message_result = repository
            .get_open_message(&signed_entity_type)
            .await
            .unwrap();
        assert!(open_message_result.is_some());

        let signed_entity_type = SignedEntityType::CardanoImmutableFilesFull(beacon.clone());
        repository
            .create_open_message(beacon.epoch, &signed_entity_type, &ProtocolMessage::new())
            .await
            .unwrap();
        let open_message_result = repository
            .get_open_message(&signed_entity_type)
            .await
            .unwrap();
        assert!(open_message_result.is_some());
    }

    #[tokio::test]
    async fn repository_create_open_message() {
        let connection = get_connection().await;
        let repository = OpenMessageRepository::new(connection.clone());
        let epoch = Epoch(1);
        let open_message = repository
            .create_open_message(
                epoch,
                &SignedEntityType::CardanoImmutableFilesFull(Beacon::default()),
                &ProtocolMessage::new(),
            )
            .await
            .unwrap();

        assert_eq!(Epoch(1), open_message.epoch);
        assert_eq!(
            SignedEntityType::CardanoImmutableFilesFull(Beacon::default()),
            open_message.signed_entity_type
        );

        let message = {
            let lock = connection.lock().await;
            let provider = OpenMessageProvider::new(&lock);
            let mut cursor = provider
                .find(WhereCondition::new(
                    "open_message_id = ?*",
                    vec![Value::String(open_message.open_message_id.to_string())],
                ))
                .unwrap();

            cursor.next().unwrap_or_else(|| {
                panic!(
                    "OpenMessage ID='{}' should exist in the database.",
                    open_message.open_message_id
                )
            })
        };

        assert_eq!(open_message.protocol_message, message.protocol_message);
        assert_eq!(open_message.epoch, message.epoch);
    }

    #[tokio::test]
    async fn repository_update_open_message() {
        let connection = get_connection().await;
        let repository = OpenMessageRepository::new(connection.clone());
        let epoch = Epoch(1);
        let open_message = repository
            .create_open_message(
                epoch,
                &SignedEntityType::CardanoImmutableFilesFull(Beacon::default()),
                &ProtocolMessage::new(),
            )
            .await
            .unwrap();

        let mut open_message_updated = open_message;
        open_message_updated.is_certified = true;
        let open_message_saved = repository
            .update_open_message(&open_message_updated)
            .await
            .unwrap();

        assert_eq!(open_message_updated, open_message_saved);
    }

    #[tokio::test]
    async fn repository_clean_open_message() {
        let connection = get_connection().await;
        let repository = OpenMessageRepository::new(connection.clone());
        let beacon = Beacon {
            epoch: Epoch(1),
            ..Beacon::default()
        };
        let _ = repository
            .create_open_message(
                beacon.epoch,
                &SignedEntityType::CardanoImmutableFilesFull(beacon.clone()),
                &ProtocolMessage::new(),
            )
            .await
            .unwrap();
        let _ = repository
            .create_open_message(
                beacon.epoch,
                &SignedEntityType::CardanoImmutableFilesFull(Beacon {
                    epoch: Epoch(2),
                    ..beacon
                }),
                &ProtocolMessage::new(),
            )
            .await
            .unwrap();
        let count = repository.clean_epoch(Epoch(2)).await.unwrap();

        assert_eq!(2, count);
    }

    #[tokio::test]
    async fn repository_get_open_message_with_single_signatures_when_signatures_exist() {
        let connection = Connection::open(":memory:").unwrap();
        apply_all_migrations_to_db(&connection).unwrap();
        disable_foreign_key_support(&connection).unwrap();
        let connection = Arc::new(Mutex::new(connection));
        let repository = OpenMessageRepository::new(connection.clone());

        let open_message = repository
            .create_open_message(
                Epoch(1),
                &SignedEntityType::MithrilStakeDistribution(Epoch(1)),
                &ProtocolMessage::default(),
            )
            .await
            .unwrap();
        let single_signature_records: Vec<SingleSignatureRecord> =
            setup_single_signature_records(1, 1, 4)
                .into_iter()
                .map(|s| SingleSignatureRecord {
                    open_message_id: open_message.open_message_id,
                    ..s
                })
                .collect();
        {
            let conn = connection.lock().await;
            insert_single_signatures_in_db(&conn, single_signature_records).unwrap();
        }

        let open_message_with_single_signatures = repository
            .get_open_message_with_single_signatures(&open_message.signed_entity_type)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            4,
            open_message_with_single_signatures.single_signatures.len()
        )
    }

    #[tokio::test]
    async fn repository_get_open_message_with_single_signatures_when_signatures_not_exist() {
        let connection = Connection::open(":memory:").unwrap();
        apply_all_migrations_to_db(&connection).unwrap();
        disable_foreign_key_support(&connection).unwrap();
        let repository = OpenMessageRepository::new(Arc::new(Mutex::new(connection)));

        let open_message = OpenMessageRecord::dummy();
        repository
            .create_open_message(
                open_message.epoch,
                &open_message.signed_entity_type,
                &open_message.protocol_message,
            )
            .await
            .unwrap();

        let open_message_with_single_signatures = repository
            .get_open_message_with_single_signatures(&open_message.signed_entity_type)
            .await
            .unwrap()
            .unwrap();
        assert!(open_message_with_single_signatures
            .single_signatures
            .is_empty())
    }
}
