#[cfg(any(test, feature = "test_tools"))]
use crate::test_utils::fake_keys;
use crate::{
    crypto_helper::{KESPeriod, ProtocolOpCert, ProtocolSignerVerificationKeySignature},
    entities::{
        HexEncodedOpCert, HexEncodedVerificationKey, HexEncodedVerificationKeySignature, PartyId,
        Signer, SignerWithStake, Stake,
    },
    StdError, StdResult,
};
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};

/// Signer with Stake Message
#[derive(Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SignerWithStakeMessagePart {
    /// The unique identifier of the signer
    ///
    /// Used only for testing when SPO pool id is not certified
    pub party_id: PartyId,

    /// The public key used to authenticate signer signature
    pub verification_key: HexEncodedVerificationKey,

    /// The encoded signer 'Mithril verification key' signature (signed by the
    /// Cardano node KES secret key).
    ///
    /// None is used only for testing when SPO pool id is not certified
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_key_signature: Option<HexEncodedVerificationKeySignature>,

    /// The encoded operational certificate of stake pool operator attached to
    /// the signer node.
    ///
    /// None is used only for testing when SPO pool id is not certified
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operational_certificate: Option<HexEncodedOpCert>,

    /// The KES period used to compute the verification key signature
    // TODO: This KES period should not be used as is and should probably be
    //       within an allowed range of KES periods for the epoch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kes_period: Option<KESPeriod>,

    /// The signer stake
    pub stake: Stake,
}

impl SignerWithStakeMessagePart {
    cfg_test_tools! {
        /// Return a dummy test entity (test-only).
        pub fn dummy() -> Self {
            Self {
                party_id: "pool1m8crhnqj5k2kyszf5j2scshupystyxc887zdfrpzh6ty6eun4fx".to_string(),
                verification_key: fake_keys::signer_verification_key()[0].to_string(),
                verification_key_signature: Some(
                    fake_keys::signer_verification_key_signature()[0].to_string(),
                ),
                operational_certificate: Some(fake_keys::operational_certificate()[0].to_string()),
                kes_period: Some(6),
                stake: 234,
            }
        }
    }

    /// Convert a set of signers into message parts
    pub fn from_signers(signers: Vec<SignerWithStake>) -> Vec<Self> {
        signers.into_iter().map(|signer| signer.into()).collect()
    }

    /// Convert a set of signer message parts into a set of signers with stake
    pub fn try_into_signers(messages: Vec<Self>) -> StdResult<Vec<SignerWithStake>> {
        messages
            .into_iter()
            .map(SignerWithStakeMessagePart::try_into)
            .collect()
    }
}

impl TryInto<SignerWithStake> for SignerWithStakeMessagePart {
    type Error = StdError;

    fn try_into(self) -> Result<SignerWithStake, Self::Error> {
        let verification_key_signature: Option<ProtocolSignerVerificationKeySignature> = self
            .verification_key_signature
            .map(|f| f.try_into())
            .transpose()
            .with_context(|| {
                format!(
                    "Error while parsing verification key signature message, party_id = '{}'",
                    self.party_id
                )
            })?;
        let operational_certificate: Option<ProtocolOpCert> = self
            .operational_certificate
            .map(|f| f.try_into())
            .transpose()
            .with_context(|| {
                format!(
                    "Error while parsing operational certificate message, party_id = '{}'.",
                    self.party_id
                )
            })?;
        let value = SignerWithStake {
            party_id: self.party_id,
            verification_key: self.verification_key.try_into()?,
            verification_key_signature,
            kes_period: self.kes_period,
            operational_certificate,
            stake: self.stake,
        };
        Ok(value)
    }
}

impl From<SignerWithStake> for SignerWithStakeMessagePart {
    fn from(value: SignerWithStake) -> Self {
        Self {
            party_id: value.party_id,
            verification_key: value.verification_key.try_into().unwrap(),
            verification_key_signature: value
                .verification_key_signature
                .map(|k| k.try_into().unwrap()),
            operational_certificate: value
                .operational_certificate
                .map(|op_cert| (op_cert.try_into().unwrap())),
            kes_period: value.kes_period,
            stake: value.stake,
        }
    }
}

impl Debug for SignerMessagePart {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let should_be_exhaustive = f.alternate();
        let mut debug = f.debug_struct("Signer");
        debug.field("party_id", &self.party_id);

        match should_be_exhaustive {
            true => debug
                .field(
                    "verification_key",
                    &format_args!("{:?}", self.verification_key),
                )
                .field(
                    "verification_key_signature",
                    &format_args!("{:?}", self.verification_key_signature),
                )
                .field(
                    "operational_certificate",
                    &format_args!("{:?}", self.operational_certificate),
                )
                .field("kes_period", &format_args!("{:?}", self.kes_period))
                .finish(),
            false => debug.finish_non_exhaustive(),
        }
    }
}

/// Signer Message
#[derive(Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SignerMessagePart {
    /// The unique identifier of the signer
    ///
    /// Used only for testing when SPO pool id is not certified
    pub party_id: PartyId,

    /// The public key used to authenticate signer signature
    pub verification_key: HexEncodedVerificationKey,

    /// The encoded signer 'Mithril verification key' signature (signed by the
    /// Cardano node KES secret key).
    ///
    /// None is used only for testing when SPO pool id is not certified
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_key_signature: Option<HexEncodedVerificationKeySignature>,

    /// The encoded operational certificate of stake pool operator attached to
    /// the signer node.
    ///
    /// None is used only for testing when SPO pool id is not certified
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operational_certificate: Option<HexEncodedOpCert>,

    /// The KES period used to compute the verification key signature
    // TODO: This KES period should not be used as is and should probably be
    //       within an allowed range of KES periods for the epoch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kes_period: Option<KESPeriod>,
}

impl SignerMessagePart {
    /// Convert a set of signer message parts into a set of signers
    pub fn try_into_signers(messages: Vec<Self>) -> StdResult<Vec<Signer>> {
        messages
            .into_iter()
            .map(SignerMessagePart::try_into)
            .collect()
    }

    /// Convert a set of signers into message parts
    pub fn from_signers(signers: Vec<Signer>) -> Vec<Self> {
        signers.into_iter().map(|signer| signer.into()).collect()
    }

    cfg_test_tools! {
        /// Return a dummy test entity (test-only).
        pub fn dummy() -> Self {
            Self {
                party_id: "pool1m8crhnqj5k2kyszf5j2scshupystyxc887zdfrpzh6ty6eun4fx".to_string(),
                verification_key: fake_keys::signer_verification_key()[0].to_string(),
                verification_key_signature: Some(
                    fake_keys::signer_verification_key_signature()[0].to_string(),
                ),
                operational_certificate: Some(fake_keys::operational_certificate()[0].to_string()),
                kes_period: Some(6),
            }
        }
    }
}

impl TryInto<Signer> for SignerMessagePart {
    type Error = StdError;

    fn try_into(self) -> Result<Signer, Self::Error> {
        let verification_key_signature: Option<ProtocolSignerVerificationKeySignature> = self
            .verification_key_signature
            .map(|f| f.try_into())
            .transpose()
            .with_context(|| {
                format!(
                    "Error while parsing verification key signature message, party_id = '{}'",
                    self.party_id
                )
            })?;
        let operational_certificate: Option<ProtocolOpCert> = self
            .operational_certificate
            .map(|f| f.try_into())
            .transpose()
            .with_context(|| {
                format!(
                    "Error while parsing operational certificate message, party_id = '{}'.",
                    self.party_id
                )
            })?;
        let value = Signer {
            party_id: self.party_id,
            verification_key: self.verification_key.try_into()?,
            verification_key_signature,
            kes_period: self.kes_period,
            operational_certificate,
        };
        Ok(value)
    }
}

impl From<Signer> for SignerMessagePart {
    fn from(value: Signer) -> Self {
        Self {
            party_id: value.party_id,
            verification_key: value.verification_key.try_into().unwrap(),
            verification_key_signature: value
                .verification_key_signature
                .map(|k| k.try_into().unwrap()),
            operational_certificate: value
                .operational_certificate
                .map(|op_cert| (op_cert.try_into().unwrap())),
            kes_period: value.kes_period,
        }
    }
}

impl Debug for SignerWithStakeMessagePart {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let should_be_exhaustive = f.alternate();
        let mut debug = f.debug_struct("Signer");
        debug
            .field("party_id", &self.party_id)
            .field("stake", &self.stake);

        match should_be_exhaustive {
            true => debug
                .field(
                    "verification_key",
                    &format_args!("{:?}", self.verification_key),
                )
                .field(
                    "verification_key_signature",
                    &format_args!("{:?}", self.verification_key_signature),
                )
                .field(
                    "operational_certificate",
                    &format_args!("{:?}", self.operational_certificate),
                )
                .field("kes_period", &format_args!("{:?}", self.kes_period))
                .finish(),
            false => debug.finish_non_exhaustive(),
        }
    }
}
