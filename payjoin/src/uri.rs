use std::borrow::Cow;
use std::convert::TryFrom;

use bip21::de::UriError;
use bip21::{DeserializationError, DeserializeParams};
use bitcoin::address::{Error, NetworkChecked, NetworkUnchecked, NetworkValidation};
use bitcoin::{Address, Amount, Network};
use url::Url;

/// Payjoin Uri represents a bip21 uri with additional
/// payjoin parameters.
pub struct PayjoinUri<
    'a,
    N: NetworkValidation,
    P: DeserializeParams<'a> + DeserializationError + Sized,
> {
    pub inner: bip21::Uri<'a, N, P>,
}

impl<'a, N: NetworkValidation, P: DeserializeParams<'a> + DeserializationError + Sized>
    From<bip21::Uri<'a, N, P>> for PayjoinUri<'a, N, P>
{
    fn from(value: bip21::Uri<'a, N, P>) -> Self { Self { inner: value } }
}

impl<'a> PayjoinUri<'a, NetworkUnchecked, Payjoin> {
    /// Marks network of this address as checked.
    pub fn assume_checked(self) -> PayjoinUri<'a, NetworkChecked, Payjoin> {
        PayjoinUri::new(
            self.inner.address.assume_checked(),
            self.inner.extras,
            self.inner.amount,
            self.inner.label,
            self.inner.message,
        )
        .into()
    }
    /// Checks whether network of this address is as required.
    pub fn require_network(
        self,
        network: Network,
    ) -> Result<PayjoinUri<'a, NetworkChecked, Payjoin>, Error> {
        Ok(PayjoinUri::new(
            self.inner.address.require_network(network)?,
            self.inner.extras,
            self.inner.amount,
            self.inner.label,
            self.inner.message,
        )
        .into())
    }
}

impl<'a, N: NetworkValidation> PayjoinUri<'a, N, Payjoin> {
    fn new(
        address: Address<N>,
        extras: Payjoin,
        amount: Option<Amount>,
        label: Option<bip21::Param<'a>>,
        message: Option<bip21::Param<'a>>,
    ) -> Self {
        let mut uri = bip21::Uri::with_extras(address, extras);
        uri.amount = amount;
        uri.label = label;
        uri.message = message;
        Self { inner: uri }
    }
}

impl From<bip21::de::Error<PjParseError>> for PjParseError {
    fn from(e: bip21::de::Error<PjParseError>) -> Self {
        match e {
            bip21::de::Error::Uri(e) => InternalPjParseError::UriParseError(e).into(),
            bip21::de::Error::Extras(e) => e.into(),
        }
    }
}

impl<'a> TryFrom<&'a str> for PayjoinUri<'a, NetworkUnchecked, Payjoin> {
    type Error = PjParseError;

    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        let uri = bip21::Uri::try_from(s).map_err(|e| Into::<PjParseError>::into(e))?;
        Ok(uri.into())
    }
}

impl ToString for PayjoinUri<'_, NetworkChecked, Payjoin> {
    fn to_string(&self) -> String { self.inner.to_string() }
}

#[derive(Clone)]
pub enum Payjoin {
    Supported(PayjoinParams),
    V2Only(PayjoinParams),
    Unsupported,
}

impl Payjoin {
    pub fn pj_is_supported(&self) -> bool {
        match self {
            Payjoin::Supported(_) => true,
            Payjoin::V2Only(_) => true,
            Payjoin::Unsupported => false,
        }
    }
    pub fn endpoint(&self) -> Option<&Url> {
        match self {
            Payjoin::Supported(params) => Some(&params.endpoint),
            Payjoin::V2Only(params) => Some(&params.endpoint),
            Payjoin::Unsupported => None,
        }
    }
    pub fn disable_output_substitution(&self) -> bool {
        match self {
            Payjoin::Supported(params) => params.disable_output_substitution,
            Payjoin::V2Only(params) => params.disable_output_substitution,
            Payjoin::Unsupported => false,
        }
    }
    #[cfg(feature = "v2")]
    pub fn ohttp_config(&self) -> Option<&ohttp::KeyConfig> {
        match self {
            Payjoin::Supported(params) => params.ohttp_config.as_ref(),
            Payjoin::V2Only(params) => params.ohttp_config.as_ref(),
            Payjoin::Unsupported => None,
        }
    }
}

#[derive(Clone)]
pub struct PayjoinParams {
    pub(crate) endpoint: Url,
    pub(crate) disable_output_substitution: bool,
    #[cfg(feature = "v2")]
    pub(crate) ohttp_config: Option<ohttp::KeyConfig>,
}

pub type Uri<'a, NetworkValidation> = bip21::Uri<'a, NetworkValidation, Payjoin>;
pub type PjUri<'a> = bip21::Uri<'a, NetworkChecked, PayjoinParams>;

mod sealed {
    use bitcoin::address::{NetworkChecked, NetworkUnchecked};

    pub trait UriExt: Sized {}

    impl<'a> UriExt for super::Uri<'a, NetworkChecked> {}
    impl<'a> UriExt for super::PjUri<'a> {}

    pub trait UriExtNetworkUnchecked: Sized {}

    impl<'a> UriExtNetworkUnchecked for super::Uri<'a, NetworkUnchecked> {}
}
pub trait UriExtNetworkUnchecked<'a>: sealed::UriExtNetworkUnchecked {
    fn require_network(self, network: Network) -> Result<Uri<'a, NetworkChecked>, Error>;

    fn assume_checked(self) -> Uri<'a, NetworkChecked>;
}

pub trait UriExt<'a>: sealed::UriExt {
    fn check_pj_supported(self) -> Result<PjUri<'a>, bip21::Uri<'a>>;
}

impl<'a> UriExtNetworkUnchecked<'a> for Uri<'a, NetworkUnchecked> {
    fn require_network(self, network: Network) -> Result<Uri<'a, NetworkChecked>, Error> {
        let checked_address = self.address.require_network(network)?;
        let mut uri = bip21::Uri::with_extras(checked_address, self.extras);
        uri.amount = self.amount;
        uri.label = self.label;
        uri.message = self.message;
        Ok(uri)
    }

    fn assume_checked(self) -> Uri<'a, NetworkChecked> {
        let checked_address = self.address.assume_checked();
        let mut uri = bip21::Uri::with_extras(checked_address, self.extras);
        uri.amount = self.amount;
        uri.label = self.label;
        uri.message = self.message;
        uri
    }
}

impl<'a> UriExt<'a> for Uri<'a, NetworkChecked> {
    fn check_pj_supported(self) -> Result<PjUri<'a>, bip21::Uri<'a>> {
        match self.extras {
            Payjoin::Supported(payjoin) | Payjoin::V2Only(payjoin) => {
                let mut uri = bip21::Uri::with_extras(self.address, payjoin);
                uri.amount = self.amount;
                uri.label = self.label;
                uri.message = self.message;

                Ok(uri)
            }
            Payjoin::Unsupported => {
                let mut uri = bip21::Uri::new(self.address);
                uri.amount = self.amount;
                uri.label = self.label;
                uri.message = self.message;

                Err(uri)
            }
        }
    }
}

impl PayjoinParams {
    pub fn is_output_substitution_disabled(&self) -> bool { self.disable_output_substitution }
}

impl bip21::de::DeserializationError for Payjoin {
    type Error = PjParseError;
}

impl<'a> bip21::de::DeserializeParams<'a> for Payjoin {
    type DeserializationState = DeserializationState;
}

#[derive(Default)]
pub struct DeserializationState {
    pj: Option<Url>,
    pjos: Option<bool>,
    #[cfg(feature = "v2")]
    ohttp: Option<ohttp::KeyConfig>,
}

#[derive(Debug)]
pub struct PjParseError(InternalPjParseError);

impl From<InternalPjParseError> for PjParseError {
    fn from(value: InternalPjParseError) -> Self { PjParseError(value) }
}

impl<'a> bip21::de::DeserializationState<'a> for DeserializationState {
    type Value = Payjoin;

    fn is_param_known(&self, param: &str) -> bool { matches!(param, "pj" | "pjos") }

    fn deserialize_temp(
        &mut self,
        key: &str,
        value: bip21::Param<'_>,
    ) -> std::result::Result<
        bip21::de::ParamKind,
        <Self::Value as bip21::DeserializationError>::Error,
    > {
        match key {
            #[cfg(feature = "v2")]
            "ohttp" if self.ohttp.is_none() => {
                let base64_config = Cow::try_from(value).map_err(InternalPjParseError::NotUtf8)?;
                let config_bytes =
                    bitcoin::base64::decode_config(&*base64_config, bitcoin::base64::URL_SAFE)
                        .map_err(InternalPjParseError::NotBase64)?;
                let config = ohttp::KeyConfig::decode(&config_bytes)
                    .map_err(InternalPjParseError::BadOhttp)?;
                self.ohttp = Some(config);
                Ok(bip21::de::ParamKind::Known)
            }
            #[cfg(feature = "v2")]
            "ohttp" => Err(PjParseError(InternalPjParseError::MultipleParams("ohttp"))),
            "pj" if self.pj.is_none() => {
                let endpoint = Cow::try_from(value).map_err(InternalPjParseError::NotUtf8)?;
                let url = Url::parse(&endpoint).map_err(InternalPjParseError::BadEndpoint)?;
                self.pj = Some(url);

                Ok(bip21::de::ParamKind::Known)
            }
            "pj" => Err(InternalPjParseError::MultipleParams("pj").into()),
            "pjos" if self.pjos.is_none() => {
                match &*Cow::try_from(value).map_err(|_| InternalPjParseError::BadPjOs)? {
                    "0" => self.pjos = Some(false),
                    "1" => self.pjos = Some(true),
                    _ => return Err(InternalPjParseError::BadPjOs.into()),
                }
                Ok(bip21::de::ParamKind::Known)
            }
            "pjos" => Err(InternalPjParseError::MultipleParams("pjos").into()),
            _ => Ok(bip21::de::ParamKind::Unknown),
        }
    }

    #[cfg(feature = "v2")]
    fn finalize(
        self,
    ) -> std::result::Result<Self::Value, <Self::Value as bip21::DeserializationError>::Error> {
        match (self.pj, self.pjos, self.ohttp) {
            (None, None, _) => Ok(Payjoin::Unsupported),
            (None, Some(_), _) => Err(PjParseError(InternalPjParseError::MissingEndpoint)),
            (Some(endpoint), pjos, None) => {
                if endpoint.scheme() == "https"
                    || endpoint.scheme() == "http"
                        && endpoint.domain().unwrap_or_default().ends_with(".onion")
                {
                    Ok(Payjoin::Supported(PayjoinParams {
                        endpoint,
                        disable_output_substitution: pjos.unwrap_or(false),
                        ohttp_config: None,
                    }))
                } else {
                    Err(PjParseError(InternalPjParseError::UnsecureEndpoint))
                }
            }
            (Some(endpoint), pjos, Some(ohttp)) => {
                if endpoint.scheme() == "https"
                    || endpoint.scheme() == "http"
                        && endpoint.domain().unwrap_or_default().ends_with(".onion")
                {
                    Ok(Payjoin::Supported(PayjoinParams {
                        endpoint,
                        disable_output_substitution: pjos.unwrap_or(false),
                        ohttp_config: Some(ohttp),
                    }))
                } else if endpoint.scheme() == "http" {
                    Ok(Payjoin::V2Only(PayjoinParams {
                        endpoint,
                        disable_output_substitution: pjos.unwrap_or(false),
                        ohttp_config: Some(ohttp),
                    }))
                } else {
                    Err(PjParseError(InternalPjParseError::UnsecureEndpoint))
                }
            }
        }
    }

    #[cfg(not(feature = "v2"))]
    fn finalize(
        self,
    ) -> std::result::Result<Self::Value, <Self::Value as bip21::DeserializationError>::Error> {
        match (self.pj, self.pjos) {
            (None, None) => Ok(Payjoin::Unsupported),
            (None, Some(_)) => Err(PjParseError(InternalPjParseError::MissingEndpoint)),
            (Some(endpoint), pjos) => {
                if endpoint.scheme() == "https"
                    || endpoint.scheme() == "http"
                        && endpoint.domain().unwrap_or_default().ends_with(".onion")
                {
                    Ok(Payjoin::Supported(PayjoinParams {
                        endpoint,
                        disable_output_substitution: pjos.unwrap_or(false),
                    }))
                } else {
                    Err(PjParseError(InternalPjParseError::UnsecureEndpoint))
                }
            }
        }
    }
}

impl std::fmt::Display for PjParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            InternalPjParseError::UriParseError(e) => write!(f, "Uri parse error: {}", e),
            InternalPjParseError::BadPjOs => write!(f, "Bad pjos parameter"),
            InternalPjParseError::MultipleParams(param) => {
                write!(f, "Multiple instances of parameter '{}'", param)
            }
            InternalPjParseError::MissingEndpoint => write!(f, "Missing payjoin endpoint"),
            InternalPjParseError::NotUtf8(_) => write!(f, "Endpoint is not valid UTF-8"),
            #[cfg(feature = "v2")]
            InternalPjParseError::NotBase64(_) => write!(f, "ohttp config is not valid base64"),
            InternalPjParseError::BadEndpoint(_) => write!(f, "Endpoint is not valid"),
            #[cfg(feature = "v2")]
            InternalPjParseError::BadOhttp(_) => write!(f, "ohttp config is not valid"),
            InternalPjParseError::UnsecureEndpoint => {
                write!(f, "Endpoint scheme is not secure (https or onion)")
            }
        }
    }
}

#[derive(Debug)]
enum InternalPjParseError {
    UriParseError(UriError),
    BadPjOs,
    MultipleParams(&'static str),
    MissingEndpoint,
    NotUtf8(core::str::Utf8Error),
    #[cfg(feature = "v2")]
    NotBase64(bitcoin::base64::DecodeError),
    BadEndpoint(url::ParseError),
    #[cfg(feature = "v2")]
    BadOhttp(ohttp::Error),
    UnsecureEndpoint,
}

impl<'a> bip21::SerializeParams for &'a Payjoin {
    type Key = &'static str;
    type Value = String;
    type Iterator = std::vec::IntoIter<(Self::Key, Self::Value)>;

    fn serialize_params(self) -> Self::Iterator {
        match self {
            Payjoin::Supported(params) => params.serialize_params(),
            Payjoin::V2Only(params) => params.serialize_params(),
            Payjoin::Unsupported => vec![].into_iter(),
        }
    }
}

impl<'a> bip21::SerializeParams for &'a PayjoinParams {
    type Key = &'static str;
    type Value = String;
    type Iterator = std::vec::IntoIter<(Self::Key, Self::Value)>;

    fn serialize_params(self) -> Self::Iterator {
        let mut params = vec![
            ("pj", self.endpoint.as_str().to_string()),
            ("pjos", if self.disable_output_substitution { "1" } else { "0" }.to_string()),
        ];
        #[cfg(feature = "v2")]
        if let Some(config) = &self.ohttp_config {
            if let Ok(config) = config.encode() {
                let encoded_config =
                    bitcoin::base64::encode_config(config, bitcoin::base64::URL_SAFE);
                params.push(("ohttp", encoded_config));
            } else {
                log::warn!("Failed to encode ohttp config, ignoring");
            }
            log::warn!("Ohttp config is not set, ignoring");
        }
        params.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use crate::Uri;

    #[test]
    fn test_short() {
        assert!(Uri::try_from("").is_err());
        assert!(Uri::try_from("bitcoin").is_err());
        assert!(Uri::try_from("bitcoin:").is_err());
    }

    #[ignore]
    #[test]
    fn test_todo_url_encoded() {
        let uri = "bitcoin:12c6DSiU4Rq3P4ZxziKxzrL5LmMBrzjrJX?amount=1&pj=https://example.com?ciao";
        assert!(Uri::try_from(uri).is_err(), "pj url should be url encoded");
    }

    #[test]
    fn test_valid_url() {
        let uri = "bitcoin:12c6DSiU4Rq3P4ZxziKxzrL5LmMBrzjrJX?amount=1&pj=this_is_NOT_a_validURL";
        assert!(Uri::try_from(uri).is_err(), "pj is not a valid url");
    }

    #[test]
    fn test_missing_amount() {
        let uri = "bitcoin:12c6DSiU4Rq3P4ZxziKxzrL5LmMBrzjrJX?pj=https://testnet.demo.btcpayserver.org/BTC/pj";
        assert!(Uri::try_from(uri).is_ok(), "missing amount should be ok");
    }

    #[test]
    fn test_unencrypted() {
        let uri = "bitcoin:12c6DSiU4Rq3P4ZxziKxzrL5LmMBrzjrJX?amount=1&pj=http://example.com";
        assert!(Uri::try_from(uri).is_err(), "unencrypted connection");

        let uri = "bitcoin:12c6DSiU4Rq3P4ZxziKxzrL5LmMBrzjrJX?amount=1&pj=ftp://foo.onion";
        assert!(Uri::try_from(uri).is_err(), "unencrypted connection");
    }

    #[test]
    fn test_valid_uris() {
        let https = "https://example.com";
        let onion = "http://vjdpwgybvubne5hda6v4c5iaeeevhge6jvo3w2cl6eocbwwvwxp7b7qd.onion";

        let base58 = "bitcoin:12c6DSiU4Rq3P4ZxziKxzrL5LmMBrzjrJX";
        let bech32_upper = "BITCOIN:TB1Q6D3A2W975YNY0ASUVD9A67NER4NKS58FF0Q8G4";
        let bech32_lower = "bitcoin:tb1q6d3a2w975yny0asuvd9a67ner4nks58ff0q8g4";

        for address in [base58, bech32_upper, bech32_lower].iter() {
            for pj in [https, onion].iter() {
                // TODO add with and without amount
                // TODO shuffle params
                let uri = format!("{}?amount=1&pj={}", address, pj);
                assert!(Uri::try_from(&*uri).is_ok());
            }
        }
    }

    #[test]
    fn test_unsupported() {
        assert!(!Uri::try_from("bitcoin:12c6DSiU4Rq3P4ZxziKxzrL5LmMBrzjrJX")
            .unwrap()
            .extras
            .pj_is_supported());
    }
}
