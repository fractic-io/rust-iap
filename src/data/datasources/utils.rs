use fractic_generic_server_error::GenericServerError;
use jose_jws::Jws;
use serde::de::DeserializeOwned;

use crate::errors::JwsError;

pub(crate) fn decode_jws_payload<T: DeserializeOwned>(
    cxt: &'static str,
    data: &str,
) -> Result<T, GenericServerError> {
    let jws = match serde_json::from_str(data) {
        Ok(Jws::General(jws)) => jws,
        Err(parsing_error) => {
            return Err(JwsError::with_debug(
                cxt,
                "Failed to parse JWS struct.",
                format!("{:?}", parsing_error),
            ))
        }
        _ => return Err(JwsError::new(cxt, "Invalid JWS type.")),
    };
    let payload = jws.payload.unwrap();
    serde_json::from_slice(&payload)
        .map_err(|e| JwsError::with_debug(cxt, "Failed to parse JWS payload.", format!("{:?}", e)))
}
