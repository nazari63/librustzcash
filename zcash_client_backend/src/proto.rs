//! Generated code for handling light client protobuf structs.

use std::io;

use incrementalmerkletree::frontier::CommitmentTree;

use nonempty::NonEmpty;
use zcash_primitives::{
    block::{BlockHash, BlockHeader},
    consensus::{self, BlockHeight, Parameters},
    memo::{self, MemoBytes},
    merkle_tree::read_commitment_tree,
    sapling::{self, note::ExtractedNoteCommitment, Node, Nullifier, NOTE_COMMITMENT_TREE_DEPTH},
    transaction::{
        components::{amount::NonNegativeAmount, OutPoint},
        fees::StandardFeeRule,
        TxId,
    },
};

use zcash_note_encryption::{EphemeralKeyBytes, COMPACT_NOTE_SIZE};

use crate::{
    data_api::{
        wallet::input_selection::{Proposal, SaplingInputs},
        PoolType, SaplingInputSource, ShieldedProtocol, TransparentInputSource,
    },
    fees::{ChangeValue, TransactionBalance},
    zip321::{TransactionRequest, Zip321Error},
};

#[rustfmt::skip]
#[allow(unknown_lints)]
#[allow(clippy::derive_partial_eq_without_eq)]
pub mod compact_formats;

#[rustfmt::skip]
#[allow(unknown_lints)]
#[allow(clippy::derive_partial_eq_without_eq)]
pub mod proposal;

#[rustfmt::skip]
#[allow(unknown_lints)]
#[allow(clippy::derive_partial_eq_without_eq)]
pub mod service;

impl compact_formats::CompactBlock {
    /// Returns the [`BlockHash`] for this block.
    ///
    /// # Panics
    ///
    /// This function will panic if [`CompactBlock.header`] is not set and
    /// [`CompactBlock.hash`] is not exactly 32 bytes.
    ///
    /// [`CompactBlock.header`]: #structfield.header
    /// [`CompactBlock.hash`]: #structfield.hash
    pub fn hash(&self) -> BlockHash {
        if let Some(header) = self.header() {
            header.hash()
        } else {
            BlockHash::from_slice(&self.hash)
        }
    }

    /// Returns the [`BlockHash`] for this block's parent.
    ///
    /// # Panics
    ///
    /// This function will panic if [`CompactBlock.header`] is not set and
    /// [`CompactBlock.prevHash`] is not exactly 32 bytes.
    ///
    /// [`CompactBlock.header`]: #structfield.header
    /// [`CompactBlock.prevHash`]: #structfield.prevHash
    pub fn prev_hash(&self) -> BlockHash {
        if let Some(header) = self.header() {
            header.prev_block
        } else {
            BlockHash::from_slice(&self.prev_hash)
        }
    }

    /// Returns the [`BlockHeight`] value for this block
    ///
    /// # Panics
    ///
    /// This function will panic if [`CompactBlock.height`] is not
    /// representable within a u32.
    pub fn height(&self) -> BlockHeight {
        self.height.try_into().unwrap()
    }

    /// Returns the [`BlockHeader`] for this block if present.
    ///
    /// A convenience method that parses [`CompactBlock.header`] if present.
    ///
    /// [`CompactBlock.header`]: #structfield.header
    pub fn header(&self) -> Option<BlockHeader> {
        if self.header.is_empty() {
            None
        } else {
            BlockHeader::read(&self.header[..]).ok()
        }
    }
}

impl compact_formats::CompactTx {
    /// Returns the transaction Id
    pub fn txid(&self) -> TxId {
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&self.hash);
        TxId::from_bytes(hash)
    }
}

impl compact_formats::CompactSaplingOutput {
    /// Returns the note commitment for this output.
    ///
    /// A convenience method that parses [`CompactOutput.cmu`].
    ///
    /// [`CompactOutput.cmu`]: #structfield.cmu
    pub fn cmu(&self) -> Result<ExtractedNoteCommitment, ()> {
        let mut repr = [0; 32];
        repr.as_mut().copy_from_slice(&self.cmu[..]);
        Option::from(ExtractedNoteCommitment::from_bytes(&repr)).ok_or(())
    }

    /// Returns the ephemeral public key for this output.
    ///
    /// A convenience method that parses [`CompactOutput.epk`].
    ///
    /// [`CompactOutput.epk`]: #structfield.epk
    pub fn ephemeral_key(&self) -> Result<EphemeralKeyBytes, ()> {
        self.ephemeral_key[..]
            .try_into()
            .map(EphemeralKeyBytes)
            .map_err(|_| ())
    }
}

impl<Proof> From<&sapling::bundle::OutputDescription<Proof>>
    for compact_formats::CompactSaplingOutput
{
    fn from(
        out: &sapling::bundle::OutputDescription<Proof>,
    ) -> compact_formats::CompactSaplingOutput {
        compact_formats::CompactSaplingOutput {
            cmu: out.cmu().to_bytes().to_vec(),
            ephemeral_key: out.ephemeral_key().as_ref().to_vec(),
            ciphertext: out.enc_ciphertext()[..COMPACT_NOTE_SIZE].to_vec(),
        }
    }
}

impl TryFrom<compact_formats::CompactSaplingOutput>
    for sapling::note_encryption::CompactOutputDescription
{
    type Error = ();

    fn try_from(value: compact_formats::CompactSaplingOutput) -> Result<Self, Self::Error> {
        Ok(sapling::note_encryption::CompactOutputDescription {
            cmu: value.cmu()?,
            ephemeral_key: value.ephemeral_key()?,
            enc_ciphertext: value.ciphertext.try_into().map_err(|_| ())?,
        })
    }
}

impl compact_formats::CompactSaplingSpend {
    pub fn nf(&self) -> Result<Nullifier, ()> {
        Nullifier::from_slice(&self.nf).map_err(|_| ())
    }
}

impl<A: sapling::bundle::Authorization> From<&sapling::bundle::SpendDescription<A>>
    for compact_formats::CompactSaplingSpend
{
    fn from(spend: &sapling::bundle::SpendDescription<A>) -> compact_formats::CompactSaplingSpend {
        compact_formats::CompactSaplingSpend {
            nf: spend.nullifier().to_vec(),
        }
    }
}

impl<SpendAuth> From<&orchard::Action<SpendAuth>> for compact_formats::CompactOrchardAction {
    fn from(action: &orchard::Action<SpendAuth>) -> compact_formats::CompactOrchardAction {
        compact_formats::CompactOrchardAction {
            nullifier: action.nullifier().to_bytes().to_vec(),
            cmx: action.cmx().to_bytes().to_vec(),
            ephemeral_key: action.encrypted_note().epk_bytes.to_vec(),
            ciphertext: action.encrypted_note().enc_ciphertext[..COMPACT_NOTE_SIZE].to_vec(),
        }
    }
}

impl service::TreeState {
    /// Deserializes and returns the Sapling note commitment tree field of the tree state.
    pub fn sapling_tree(&self) -> io::Result<CommitmentTree<Node, NOTE_COMMITMENT_TREE_DEPTH>> {
        let sapling_tree_bytes = hex::decode(&self.sapling_tree).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Hex decoding of Sapling tree bytes failed: {:?}", e),
            )
        })?;
        read_commitment_tree::<Node, _, NOTE_COMMITMENT_TREE_DEPTH>(&sapling_tree_bytes[..])
    }
}

pub const PROPOSAL_SER_V1: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProposalError<DbError> {
    Zip321(Zip321Error),
    TxIdInvalid(Vec<u8>),
    InputRetrieval(DbError),
    InputNotFound(TxId, PoolType, u32),
    BalanceInvalid,
    MemoInvalid(memo::Error),
    VersionInvalid(u32),
    ZeroMinConf,
    FeeRuleNotSpecified,
}

impl<E> From<Zip321Error> for ProposalError<E> {
    fn from(value: Zip321Error) -> Self {
        Self::Zip321(value)
    }
}

impl proposal::ProposedInput {
    pub fn parse_txid(&self) -> Result<TxId, Vec<u8>> {
        Ok(TxId::from_bytes(self.txid.clone().try_into()?))
    }
}

impl proposal::Proposal {
    /// Serializes a [`Proposal`] based upon a supported [`StandardFeeRule`] to its protobuf
    /// representation.
    pub fn from_standard_proposal<P: Parameters, NoteRef>(
        params: &P,
        value: &Proposal<StandardFeeRule, NoteRef>,
    ) -> Option<Self> {
        let transaction_request = value.transaction_request().to_uri(params)?;

        let transparent_inputs = value
            .transparent_inputs()
            .iter()
            .map(|utxo| proposal::ProposedInput {
                txid: utxo.outpoint().hash().to_vec(),
                index: utxo.outpoint().n(),
                value: utxo.txout().value.into(),
            })
            .collect();

        let sapling_inputs = value
            .sapling_inputs()
            .map(|sapling_inputs| proposal::SaplingInputs {
                anchor_height: sapling_inputs.anchor_height().into(),
                inputs: sapling_inputs
                    .notes()
                    .iter()
                    .map(|rec_note| proposal::ProposedInput {
                        txid: rec_note.txid().as_ref().to_vec(),
                        index: rec_note.output_index().into(),
                        value: rec_note.value().into(),
                    })
                    .collect(),
            });

        let balance = Some(proposal::TransactionBalance {
            proposed_change: value
                .balance()
                .proposed_change()
                .iter()
                .map(|cv| match cv {
                    ChangeValue::Sapling { value, memo } => proposal::ChangeValue {
                        value: Some(proposal::change_value::Value::SaplingValue(
                            proposal::SaplingChange {
                                amount: (*value).into(),
                                memo: memo.as_ref().map(|memo_bytes| proposal::MemoBytes {
                                    value: memo_bytes.as_slice().to_vec(),
                                }),
                            },
                        )),
                    },
                })
                .collect(),
            fee_required: value.balance().fee_required().into(),
        });

        #[allow(deprecated)]
        Some(proposal::Proposal {
            proto_version: PROPOSAL_SER_V1,
            transaction_request,
            transparent_inputs,
            sapling_inputs,
            balance,
            fee_rule: match value.fee_rule() {
                StandardFeeRule::PreZip313 => proposal::FeeRule::PreZip313,
                StandardFeeRule::Zip313 => proposal::FeeRule::Zip313,
                StandardFeeRule::Zip317 => proposal::FeeRule::Zip317,
            }
            .into(),
            min_target_height: value.min_target_height().into(),
            is_shielding: value.is_shielding(),
        })
    }

    /// Attempts to parse a [`Proposal`] based upon a supported [`StandardFeeRule`] from its
    /// protobuf representation.
    pub fn try_into_standard_proposal<P: consensus::Parameters, DbT, DbError>(
        &self,
        params: &P,
        wallet_db: &DbT,
    ) -> Result<Proposal<StandardFeeRule, DbT::NoteRef>, ProposalError<DbError>>
    where
        DbT: TransparentInputSource<Error = DbError> + SaplingInputSource<Error = DbError>,
    {
        match self.proto_version {
            PROPOSAL_SER_V1 => {
                #[allow(deprecated)]
                let fee_rule = match self.fee_rule() {
                    proposal::FeeRule::PreZip313 => StandardFeeRule::PreZip313,
                    proposal::FeeRule::Zip313 => StandardFeeRule::Zip313,
                    proposal::FeeRule::Zip317 => StandardFeeRule::Zip317,
                    proposal::FeeRule::NotSpecified => {
                        return Err(ProposalError::FeeRuleNotSpecified);
                    }
                };

                let transaction_request =
                    TransactionRequest::from_uri(params, &self.transaction_request)?;
                let transparent_inputs = self
                    .transparent_inputs
                    .iter()
                    .map(|t_in| {
                        let txid = t_in.parse_txid().map_err(ProposalError::TxIdInvalid)?;
                        let outpoint = OutPoint::new(txid.into(), t_in.index);

                        wallet_db
                            .get_unspent_transparent_output(&outpoint)
                            .map_err(ProposalError::InputRetrieval)?
                            .ok_or_else(|| {
                                ProposalError::InputNotFound(
                                    txid,
                                    PoolType::Transparent,
                                    t_in.index,
                                )
                            })
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                let sapling_inputs = self.sapling_inputs.as_ref().and_then(|s_in| {
                    s_in.inputs
                        .iter()
                        .map(|s_in| {
                            let txid = s_in.parse_txid().map_err(ProposalError::TxIdInvalid)?;

                            wallet_db
                                .get_spendable_sapling_note(&txid, s_in.index)
                                .map_err(ProposalError::InputRetrieval)
                                .and_then(|opt| {
                                    opt.ok_or_else(|| {
                                        ProposalError::InputNotFound(
                                            txid,
                                            PoolType::Shielded(ShieldedProtocol::Sapling),
                                            s_in.index,
                                        )
                                    })
                                })
                        })
                        .collect::<Result<Vec<_>, _>>()
                        .map(|notes| {
                            NonEmpty::from_vec(notes).map(|notes| {
                                SaplingInputs::from_parts(s_in.anchor_height.into(), notes)
                            })
                        })
                        .transpose()
                });

                let balance = self.balance.as_ref().ok_or(ProposalError::BalanceInvalid)?;
                let balance = TransactionBalance::new(
                    balance
                        .proposed_change
                        .iter()
                        .filter_map(|cv| {
                            cv.value
                                .as_ref()
                                .map(|cv| -> Result<ChangeValue, ProposalError<_>> {
                                    match cv {
                                        proposal::change_value::Value::SaplingValue(sc) => {
                                            Ok(ChangeValue::sapling(
                                                NonNegativeAmount::from_u64(sc.amount)
                                                    .map_err(|_| ProposalError::BalanceInvalid)?,
                                                sc.memo
                                                    .as_ref()
                                                    .map(|bytes| {
                                                        MemoBytes::from_bytes(&bytes.value)
                                                            .map_err(ProposalError::MemoInvalid)
                                                    })
                                                    .transpose()?,
                                            ))
                                        }
                                    }
                                })
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                    NonNegativeAmount::from_u64(balance.fee_required)
                        .map_err(|_| ProposalError::BalanceInvalid)?,
                )
                .map_err(|_| ProposalError::BalanceInvalid)?;

                Proposal::from_parts(
                    transaction_request,
                    transparent_inputs,
                    sapling_inputs.transpose()?,
                    balance,
                    fee_rule,
                    self.min_target_height.into(),
                    self.is_shielding,
                )
                .map_err(|_| ProposalError::BalanceInvalid)
            }
            other => Err(ProposalError::VersionInvalid(other)),
        }
    }
}
