use std::time::Duration;

use fractic_server_error::ServerError;
use jose_jws::Jws;
use jwtk::{jwk::RemoteJwksVerifier, OneOrMany};
use serde::de::DeserializeOwned;

use crate::{
    constants::{APPLE_JWK_URL, GOOGLE_JWK_URL},
    errors::{InvalidAppleSignature, InvalidJws},
};

/// Decodes the payload from a JWS object, without performing any signature
/// verification.
pub(crate) fn decode_jws_payload<T: DeserializeOwned>(data: &str) -> Result<T, ServerError> {
    let jws = match serde_json::from_str(data) {
        Ok(Jws::General(jws)) => jws,
        Err(parsing_error) => {
            return Err(InvalidJws::with_debug(
                "failed to parse JWS struct",
                &parsing_error,
            ))
        }
        _ => return Err(InvalidJws::new("invalid JWS type")),
    };
    let payload = jws
        .payload
        .ok_or(InvalidJws::new("JWS payload is missing"))?;
    serde_json::from_slice(&payload)
        .map_err(|e| InvalidJws::with_debug("failed to parse JWS payload", &e))
}

/// Validates that the jws is signed by Apple.
pub(crate) async fn validate_apple_signature(
    jws: &str,
    expected_aud: &str,
) -> Result<(), ServerError> {
    validate_token(jws, APPLE_JWK_URL.to_string(), expected_aud).await
}

/// Validates that the jwt is signed by Google.
pub(crate) async fn validate_google_signature(
    jwt: &str,
    expected_aud: &str,
) -> Result<(), ServerError> {
    validate_token(jwt, GOOGLE_JWK_URL.to_string(), expected_aud).await
}

async fn validate_token(
    token: &str,
    jwk_url: String,
    expected_aud: &str,
) -> Result<(), ServerError> {
    // NOTE: Since we create a new RemoteJwksVerifier every time, we don't
    // really benefit from the cache here. If this code gets lots of traffic in
    // the future, it should probably be refactored to share the verifier
    // between requests.
    let verifier = RemoteJwksVerifier::new(jwk_url, None, Duration::from_secs(300));
    let result = verifier
        .verify::<serde_json::Map<String, serde_json::Value>>(token)
        .await
        .map_err(|e| InvalidAppleSignature::with_debug("token", &e))?;
    let valid_aud = match result.claims().aud {
        OneOrMany::One(ref aud) => aud == expected_aud,
        OneOrMany::Vec(ref auds) => auds.iter().any(|aud| aud == expected_aud),
    };
    if !valid_aud {
        return Err(InvalidAppleSignature::with_debug(
            "audience",
            &result.claims(),
        ));
    }
    Ok(())
}
