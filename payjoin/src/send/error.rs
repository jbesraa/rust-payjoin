use std::fmt::{self, Display};

use bitcoin::locktime::absolute::LockTime;
use bitcoin::Sequence;

use crate::input_type::{InputType, InputTypeError};

/// Error that may occur when the response from receiver is malformed.
///
/// This is currently opaque type because we aren't sure which variants will stay.
/// You can only display it.
#[derive(Debug)]
pub struct ValidationError {
    internal: InternalValidationError,
}

#[derive(Debug)]
pub(crate) enum InternalValidationError {
    Parse,
    Io(std::io::Error),
    InvalidInputType(InputTypeError),
    InvalidProposedInput(crate::psbt::PrevTxOutError),
    VersionsDontMatch {
        proposed: i32,
        original: i32,
    },
    LockTimesDontMatch {
        proposed: LockTime,
        original: LockTime,
    },
    SenderTxinSequenceChanged {
        proposed: Sequence,
        original: Sequence,
    },
    SenderTxinContainsNonWitnessUtxo,
    SenderTxinContainsWitnessUtxo,
    SenderTxinContainsFinalScriptSig,
    SenderTxinContainsFinalScriptWitness,
    TxInContainsKeyPaths,
    ContainsPartialSigs,
    ReceiverTxinNotFinalized,
    ReceiverTxinMissingUtxoInfo,
    MixedSequence,
    MixedInputTypes {
        proposed: InputType,
        original: InputType,
    },
    MissingOrShuffledInputs,
    TxOutContainsKeyPaths,
    FeeContributionExceedsMaximum,
    DisallowedOutputSubstitution,
    OutputValueDecreased,
    MissingOrShuffledOutputs,
    Inflation,
    AbsoluteFeeDecreased,
    PayeeTookContributedFee,
    FeeContributionPaysOutputSizeIncrease,
    FeeRateBelowMinimum,
    #[cfg(feature = "v2")]
    V2(crate::v2::Error),
    #[cfg(feature = "v2")]
    Psbt(bitcoin::psbt::Error),
}

impl From<InternalValidationError> for ValidationError {
    fn from(value: InternalValidationError) -> Self { ValidationError { internal: value } }
}
impl From<InputTypeError> for InternalValidationError {
    fn from(value: InputTypeError) -> Self { InternalValidationError::InvalidInputType(value) }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use InternalValidationError::*;

        match &self.internal {
            Parse => write!(f, "couldn't decode as PSBT or JSON",),
            Io(e) => write!(f, "couldn't read PSBT: {}", e),
            InvalidInputType(e) => write!(f, "invalid transaction input type: {}", e),
            InvalidProposedInput(e) => write!(f, "invalid proposed transaction input: {}", e),
            VersionsDontMatch { proposed, original, } => write!(f, "proposed transaction version {} doesn't match the original {}", proposed, original),
            LockTimesDontMatch { proposed, original, } => write!(f, "proposed transaction lock time {} doesn't match the original {}", proposed, original),
            SenderTxinSequenceChanged { proposed, original, } => write!(f, "proposed transaction sequence number {} doesn't match the original {}", proposed, original),
            SenderTxinContainsNonWitnessUtxo => write!(f, "an input in proposed transaction belonging to the sender contains non-witness UTXO information"),
            SenderTxinContainsWitnessUtxo => write!(f, "an input in proposed transaction belonging to the sender contains witness UTXO information"),
            SenderTxinContainsFinalScriptSig => write!(f, "an input in proposed transaction belonging to the sender contains finalized non-witness signature"),
            SenderTxinContainsFinalScriptWitness => write!(f, "an input in proposed transaction belonging to the sender contains finalized witness signature"),
            TxInContainsKeyPaths => write!(f, "proposed transaction inputs contain key paths"),
            ContainsPartialSigs => write!(f, "an input in proposed transaction belonging to the sender contains partial signatures"),
            ReceiverTxinNotFinalized => write!(f, "an input in proposed transaction belonging to the receiver is not finalized"),
            ReceiverTxinMissingUtxoInfo => write!(f, "an input in proposed transaction belonging to the receiver is missing UTXO information"),
            MixedSequence => write!(f, "inputs of proposed transaction contain mixed sequence numbers"),
            MixedInputTypes { proposed, original, } => write!(f, "proposed transaction contains input of type {:?} while original contains inputs of type {:?}", proposed, original),
            MissingOrShuffledInputs => write!(f, "proposed transaction is missing inputs of the sender or they are shuffled"),
            TxOutContainsKeyPaths => write!(f, "proposed transaction outputs contain key paths"),
            FeeContributionExceedsMaximum => write!(f, "fee contribution exceeds allowed maximum"),
            DisallowedOutputSubstitution => write!(f, "the receiver change output despite it being disallowed"),
            OutputValueDecreased => write!(f, "the amount in our non-fee output was decreased"),
            MissingOrShuffledOutputs => write!(f, "proposed transaction is missing outputs of the sender or they are shuffled"),
            Inflation => write!(f, "proposed transaction is attempting inflation"),
            AbsoluteFeeDecreased => write!(f, "abslute fee of proposed transaction is lower than original"),
            PayeeTookContributedFee => write!(f, "payee tried to take fee contribution for himself"),
            FeeContributionPaysOutputSizeIncrease => write!(f, "fee contribution pays for additional outputs"),
            FeeRateBelowMinimum =>  write!(f, "the fee rate of proposed transaction is below minimum"),
            #[cfg(feature = "v2")]
            V2(e) => write!(f, "v2 error: {}", e),
            #[cfg(feature = "v2")]
            Psbt(e) => write!(f, "psbt error: {}", e),
        }
    }
}

impl std::error::Error for ValidationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use InternalValidationError::*;

        match &self.internal {
            Parse => None,
            Io(error) => Some(error),
            InvalidInputType(error) => Some(error),
            InvalidProposedInput(error) => Some(error),
            VersionsDontMatch { proposed: _, original: _ } => None,
            LockTimesDontMatch { proposed: _, original: _ } => None,
            SenderTxinSequenceChanged { proposed: _, original: _ } => None,
            SenderTxinContainsNonWitnessUtxo => None,
            SenderTxinContainsWitnessUtxo => None,
            SenderTxinContainsFinalScriptSig => None,
            SenderTxinContainsFinalScriptWitness => None,
            TxInContainsKeyPaths => None,
            ContainsPartialSigs => None,
            ReceiverTxinNotFinalized => None,
            ReceiverTxinMissingUtxoInfo => None,
            MixedSequence => None,
            MixedInputTypes { .. } => None,
            MissingOrShuffledInputs => None,
            TxOutContainsKeyPaths => None,
            FeeContributionExceedsMaximum => None,
            DisallowedOutputSubstitution => None,
            OutputValueDecreased => None,
            MissingOrShuffledOutputs => None,
            Inflation => None,
            AbsoluteFeeDecreased => None,
            PayeeTookContributedFee => None,
            FeeContributionPaysOutputSizeIncrease => None,
            FeeRateBelowMinimum => None,
            #[cfg(feature = "v2")]
            V2(error) => Some(error),
            #[cfg(feature = "v2")]
            Psbt(error) => Some(error),
        }
    }
}

/// Error returned when request could not be created.
///
/// This error can currently only happen due to programmer mistake.
/// `unwrap()`ing it is thus considered OK in Rust but you may achieve nicer message by displaying
/// it.
#[derive(Debug)]
pub struct CreateRequestError(InternalCreateRequestError);

#[derive(Debug)]
pub(crate) enum InternalCreateRequestError {
    InvalidOriginalInput(crate::psbt::PsbtInputsError),
    InconsistentOriginalPsbt(crate::psbt::InconsistentPsbt),
    NoInputs,
    PayeeValueNotEqual,
    NoOutputs,
    MultiplePayeeOutputs,
    MissingPayeeOutput,
    FeeOutputValueLowerThanFeeContribution,
    AmbiguousChangeOutput,
    ChangeIndexOutOfBounds,
    ChangeIndexPointsAtPayee,
    Url(url::ParseError),
    UriDoesNotSupportPayjoin,
    PrevTxOut(crate::psbt::PrevTxOutError),
    InputType(crate::input_type::InputTypeError),
    #[cfg(feature = "v2")]
    V2(crate::v2::Error),
    #[cfg(feature = "v2")]
    SubdirectoryNotBase64(bitcoin::base64::DecodeError),
    #[cfg(feature = "v2")]
    SubdirectoryInvalidPubkey(bitcoin::secp256k1::Error),
    #[cfg(feature = "v2")]
    MissingOhttpConfig,
}

impl fmt::Display for CreateRequestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use InternalCreateRequestError::*;

        match &self.0 {
            InvalidOriginalInput(e) => write!(f, "an input in the original transaction is invalid: {:#?}", e),
            InconsistentOriginalPsbt(e) => write!(f, "the original transaction is inconsistent: {:#?}", e),
            NoInputs => write!(f, "the original transaction has no inputs"),
            PayeeValueNotEqual => write!(f, "the value in original transaction doesn't equal value requested in the payment link"),
            NoOutputs => write!(f, "the original transaction has no outputs"),
            MultiplePayeeOutputs => write!(f, "the original transaction has more than one output belonging to the payee"),
            MissingPayeeOutput => write!(f, "the output belonging to payee is missing from the original transaction"),
            FeeOutputValueLowerThanFeeContribution => write!(f, "the value of fee output is lower than maximum allowed contribution"),
            AmbiguousChangeOutput => write!(f, "can not determine which output is change because there's more than two outputs"),
            ChangeIndexOutOfBounds => write!(f, "fee output index is points out of bounds"),
            ChangeIndexPointsAtPayee => write!(f, "fee output index is points at output belonging to the payee"),
            Url(e) => write!(f, "cannot parse url: {:#?}", e),
            UriDoesNotSupportPayjoin => write!(f, "the URI does not support payjoin"),
            PrevTxOut(e) => write!(f, "invalid previous transaction output: {}", e),
            InputType(e) => write!(f, "invalid input type: {}", e),
            #[cfg(feature = "v2")]
            V2(e) => write!(f, "v2 error: {}", e),
            #[cfg(feature = "v2")]
            SubdirectoryNotBase64(e) => write!(f, "subdirectory is not valid base64 error: {}", e),
            #[cfg(feature = "v2")]
            SubdirectoryInvalidPubkey(e) => write!(f, "subdirectory does not represent a valid pubkey: {}", e),
            #[cfg(feature = "v2")]
            MissingOhttpConfig => write!(f, "no ohttp configuration with which to make a v2 request available"),
        }
    }
}

impl std::error::Error for CreateRequestError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use InternalCreateRequestError::*;

        match &self.0 {
            InvalidOriginalInput(error) => Some(error),
            InconsistentOriginalPsbt(error) => Some(error),
            NoInputs => None,
            PayeeValueNotEqual => None,
            NoOutputs => None,
            MultiplePayeeOutputs => None,
            MissingPayeeOutput => None,
            FeeOutputValueLowerThanFeeContribution => None,
            AmbiguousChangeOutput => None,
            ChangeIndexOutOfBounds => None,
            ChangeIndexPointsAtPayee => None,
            Url(error) => Some(error),
            UriDoesNotSupportPayjoin => None,
            PrevTxOut(error) => Some(error),
            InputType(error) => Some(error),
            #[cfg(feature = "v2")]
            V2(error) => Some(error),
            #[cfg(feature = "v2")]
            SubdirectoryNotBase64(error) => Some(error),
            #[cfg(feature = "v2")]
            SubdirectoryInvalidPubkey(error) => Some(error),
            #[cfg(feature = "v2")]
            MissingOhttpConfig => None,
        }
    }
}

impl From<InternalCreateRequestError> for CreateRequestError {
    fn from(value: InternalCreateRequestError) -> Self { CreateRequestError(value) }
}

/// Represent an error returned by the receiver.
pub enum ResponseError {
    /// `WellKnown` errors following the BIP78 spec
    /// https://github.com/bitcoin/bips/blob/master/bip-0078.mediawiki#user-content-Receivers_well_known_errors
    /// These errors are displayed to end users.
    ///
    /// The `WellKnownError` represents `errorCode` and `message`.
    WellKnown(WellKnownError),
    /// `Unrecognized` errors are errors that are not well known and are only displayed in debug logs.
    /// They are not displayed to end users.
    ///
    /// The first `String` is `errorCode`
    /// The second `String` is `message`.
    Unrecognized(String, String),
    /// `Validation` errors are errors that are caused by malformed responses.
    /// They are only displayed in debug logs.
    Validation(ValidationError),
}

impl ResponseError {
    fn from_json(json: serde_json::Value) -> Self {
        // we try to find the errorCode field and
        // if it exists we try to parse it as a well known error
        // if its an unknown error we return the error code and message
        // from original response
        // if errorCode field doesn't exist we return parse error
        let message = json
            .as_object()
            .and_then(|v| v.get("message"))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        if let Some(error_code) =
            json.as_object().and_then(|v| v.get("errorCode")).and_then(|v| v.as_str())
        {
            match error_code {
                "version-unsupported" => {
                    let supported = json
                        .as_object()
                        .and_then(|v| v.get("supported"))
                        .and_then(|v| v.as_array())
                        .map(|array| array.iter().filter_map(|v| v.as_u64()).collect::<Vec<u64>>())
                        .unwrap_or_default();
                    WellKnownError::VersionUnsupported(message, supported).into()
                }
                "unavailable" => WellKnownError::Unavailable(message).into(),
                "not-enough-money" => WellKnownError::NotEnoughMoney(message).into(),
                "original-psbt-rejected" => WellKnownError::OriginalPsbtRejected(message).into(),
                _ => Self::Unrecognized(error_code.to_string(), message),
            }
        } else {
            InternalValidationError::Parse.into()
        }
    }

    /// Parse a response from the receiver.
    ///
    /// response must be valid JSON string.
    pub fn parse(response: &str) -> Self {
        match serde_json::from_str(response) {
            Ok(json) => Self::from_json(json),
            Err(_) => InternalValidationError::Parse.into(),
        }
    }

    /// Whether the error provided by the payjoin receiver is suggesting a specific version of
    /// payjoin.
    ///
    /// This is useful for when you support multiple versions of payjoin. For example, if you
    /// support v1 and v2(BiP78 && BiP77 respectively), you can check if the error is suggesting a
    /// specific version and if it does, you can retry the request based on the asked version.
    pub fn requires_version(&self, v: &u64) -> bool {
        match self {
            Self::WellKnown(e) => match e {
                WellKnownError::VersionUnsupported(_, supported) => supported.contains(v),
                _ => false,
            },
            _ => false,
        }
    }
}

impl std::error::Error for ResponseError {}

impl From<WellKnownError> for ResponseError {
    fn from(value: WellKnownError) -> Self { Self::WellKnown(value) }
}

impl From<InternalValidationError> for ResponseError {
    fn from(value: InternalValidationError) -> Self {
        Self::Validation(ValidationError { internal: value })
    }
}

// It is imperative to carefully display pre-defined messages to end users and the details in debug.
impl Display for ResponseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::WellKnown(e) => e.fmt(f),
            // Don't display unknowns to end users, only debug logs
            Self::Unrecognized(_, _) => write!(f, "The receiver sent an unrecognized error."),
            Self::Validation(e) => write!(f, "The receiver sent an invalid response: {}", e),
        }
    }
}

impl fmt::Debug for ResponseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::WellKnown(e) => write!(
                f,
                r#"Well known error: {{ "errorCode": "{}",
                "message": "{}" }}"#,
                e.error_code(),
                e.message()
            ),
            Self::Unrecognized(code, msg) => write!(
                f,
                r#"Unrecognized error: {{ "errorCode": "{}", "message": "{}" }}"#,
                code, msg
            ),
            Self::Validation(e) => write!(f, "Validation({:?})", e),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WellKnownError {
    Unavailable(String),
    NotEnoughMoney(String),
    VersionUnsupported(String, Vec<u64>),
    OriginalPsbtRejected(String),
}

impl WellKnownError {
    pub fn error_code(&self) -> &str {
        match self {
            WellKnownError::Unavailable(_) => "unavailable",
            WellKnownError::NotEnoughMoney(_) => "not-enough-money",
            WellKnownError::VersionUnsupported(_, _) => "version-unsupported",
            WellKnownError::OriginalPsbtRejected(_) => "original-psbt-rejected",
        }
    }
    pub fn message(&self) -> &str {
        match self {
            WellKnownError::Unavailable(m) => m,
            WellKnownError::NotEnoughMoney(m) => m,
            WellKnownError::VersionUnsupported(m, _) => m,
            WellKnownError::OriginalPsbtRejected(m) => m,
        }
    }
}

impl Display for WellKnownError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Unavailable(_) => write!(f, "The payjoin endpoint is not available for now."),
            Self::NotEnoughMoney(_) => write!(f, "The receiver added some inputs but could not bump the fee of the payjoin proposal."),
            Self::VersionUnsupported(_, v) => write!(f, "This version of payjoin is not supported. Use version {:?}.", v),
            Self::OriginalPsbtRejected(_) => write!(f, "The receiver rejected the original PSBT."),
        }
    }
}

#[cfg(test)]
mod tests {
    use bitcoind::bitcoincore_rpc::jsonrpc::serde_json::json;

    use super::*;

    #[test]
    fn test_parse_json() {
        let known_str_error = r#"{"errorCode":"version-unsupported", "message":"custom message here", "supported": [1, 2]}"#;
        let error = ResponseError::parse(known_str_error);
        match error {
            ResponseError::WellKnown(e) => {
                assert_eq!(e.error_code(), "version-unsupported");
                assert_eq!(e.message(), "custom message here");
                assert_eq!(
                    e.to_string(),
                    "This version of payjoin is not supported. Use version [1, 2]."
                );
            }
            _ => panic!("Expected WellKnown error"),
        };
        let unrecognized_error = r#"{"errorCode":"random", "message":"random"}"#;
        assert_eq!(
            ResponseError::parse(unrecognized_error).to_string(),
            "The receiver sent an unrecognized error."
        );
        let invalid_json_error = json!({
            "err": "random",
            "message": "This version of payjoin is not supported."
        });
        assert_eq!(
            ResponseError::from_json(invalid_json_error).to_string(),
            "The receiver sent an invalid response: couldn't decode as PSBT or JSON"
        );
    }
}
