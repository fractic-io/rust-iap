use std::time::Duration;

use base64::{prelude::BASE64_STANDARD, Engine as _};
use fractic_server_error::{CriticalError, ServerError};
use jsonwebtoken::decode_header;
use jwtk::{jwk::RemoteJwksVerifier, OneOrMany};
use once_cell::sync::Lazy;
use openssl::{
    error::ErrorStack,
    stack::Stack,
    x509::{
        store::{X509Store, X509StoreBuilder},
        X509StoreContext, X509,
    },
};
use serde::de::DeserializeOwned;

use crate::{
    constants::GOOGLE_JWK_URL,
    errors::{InvalidAppleSignature, InvalidGoogleSignature, InvalidJws},
};

static APPLE_TRUST_STORE: Lazy<Result<X509Store, ErrorStack>> = Lazy::new(|| {
    let mut store_builder = X509StoreBuilder::new()?;
    X509::from_der(include_bytes!("../../../res/trust/AppleRootCA-G2.cer"))
        .and_then(|cert| store_builder.add_cert(cert))?;
    X509::from_der(include_bytes!("../../../res/trust/AppleRootCA-G3.cer"))
        .and_then(|cert| store_builder.add_cert(cert))?;
    X509::from_der(include_bytes!("../../../res/trust/AppleWWDRCAG2.cer"))
        .and_then(|cert| store_builder.add_cert(cert))?;
    X509::from_der(include_bytes!("../../../res/trust/AppleWWDRCAG3.cer"))
        .and_then(|cert| store_builder.add_cert(cert))?;
    X509::from_der(include_bytes!("../../../res/trust/AppleWWDRCAG4.cer"))
        .and_then(|cert| store_builder.add_cert(cert))?;
    X509::from_der(include_bytes!("../../../res/trust/AppleWWDRCAG5.cer"))
        .and_then(|cert| store_builder.add_cert(cert))?;
    X509::from_der(include_bytes!("../../../res/trust/AppleWWDRCAG6.cer"))
        .and_then(|cert| store_builder.add_cert(cert))?;
    X509::from_der(include_bytes!("../../../res/trust/AppleWWDRCAG8.cer"))
        .and_then(|cert| store_builder.add_cert(cert))?;
    Ok(store_builder.build())
});

static GOOGLE_JWK_VERIFIER: Lazy<RemoteJwksVerifier> = Lazy::new(|| {
    RemoteJwksVerifier::new(GOOGLE_JWK_URL.to_owned(), None, Duration::from_secs(300))
});

/// Validates that the jws is signed by Apple, and returns the payload parsed as
/// type T from JSON.
pub(crate) async fn validate_and_parse_apple_jws<T: DeserializeOwned>(
    jws: &str,
    expected_aud: &str,
) -> Result<T, ServerError> {
    // Parse x5c cert chain from JWS header.
    let header =
        decode_header(jws).map_err(|e| InvalidJws::with_debug("failed to parse JWS header", &e))?;
    let x5c_chain = header
        .x5c
        .ok_or(InvalidJws::new("missing x5c field in JWS header"))?;
    let certs = x5c_chain
        .into_iter()
        .map(|x5c| {
            X509::from_der(&BASE64_STANDARD.decode(x5c.as_bytes()).map_err(|e| {
                InvalidAppleSignature::with_debug("failed to base64 decode x5c certs", &e)
            })?)
            .map_err(|e| InvalidAppleSignature::with_debug("failed to decode x5c certs", &e))
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Validate certificate chain.
    let mut chain =
        Stack::new().map_err(|e| CriticalError::with_debug("failed to create X509 stack", &e))?;
    let mut certs_iter = certs.into_iter();
    let leaf_cert = certs_iter
        .next()
        .ok_or(InvalidAppleSignature::new("empty x5c chain"))?;
    for cert in certs_iter {
        chain
            .push(cert.clone())
            .map_err(|e| CriticalError::with_debug("failed to push cert to X509 stack", &e))?;
    }
    let mut cxt = X509StoreContext::new()
        .map_err(|e| CriticalError::with_debug("failed to create X509 store context", &e))?;
    let trust_store = APPLE_TRUST_STORE
        .as_ref()
        .map_err(|e| CriticalError::with_debug("failed to build Apple trust store", e))?;
    let valid = cxt
        .init(&trust_store, &leaf_cert, &chain, |cxt| cxt.verify_cert())
        .map_err(|e| InvalidAppleSignature::with_debug("failed to validate x5c chain", &e))?;
    if !valid {
        return Err(InvalidAppleSignature::new("invalid x5c chain"));
    }

    // Calculate public key used to sign JWS.
    let public_key = leaf_cert.public_key().map_err(|e| {
        InvalidAppleSignature::with_debug("couldn't get public key from leaf cert", &e)
    })?;
    let public_key_pem = public_key
        .public_key_to_pem()
        .map_err(|e| InvalidAppleSignature::with_debug("couldn't convert public key to PEM", &e))?;

    // Verify JWS signature.
    let decoding_key = jsonwebtoken::DecodingKey::from_ec_pem(&public_key_pem)
        .map_err(|e| InvalidAppleSignature::with_debug("failed to create decoding key", &e))?;
    let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::ES256);
    validation.required_spec_claims = Default::default();
    validation.set_audience(&[expected_aud]);
    let payload = jsonwebtoken::decode::<serde_json::Value>(jws, &decoding_key, &validation)
        .map_err(|e| InvalidAppleSignature::with_debug("failed to verify JWS signature", &e))?;

    // Parse payload.
    //
    // Since this is a JWT library, it expects the data to be JWT 'claims'.
    // However in our case, that's actually our JWS data.
    serde_json::from_value(payload.claims)
        .map_err(|e| InvalidJws::with_debug("failed to parse JWS payload", &e))
}

/// Validates that the jwt is signed by Google.
pub(crate) async fn validate_google_header(
    authentication_header: &str,
    expected_aud: &str,
) -> Result<(), ServerError> {
    let token = authentication_header.trim_start_matches("Bearer ").trim();
    let result = GOOGLE_JWK_VERIFIER
        .verify::<serde_json::Map<String, serde_json::Value>>(token)
        .await
        .map_err(|e| InvalidGoogleSignature::with_debug("token", &e))?;
    let valid_aud = match result.claims().aud {
        OneOrMany::One(ref aud) => aud == expected_aud,
        OneOrMany::Vec(ref auds) => auds.iter().any(|aud| aud == expected_aud),
    };
    if !valid_aud {
        return Err(InvalidGoogleSignature::with_debug(
            "audience",
            &result.claims(),
        ));
    }
    Ok(())
}
