use fractic_server_error::{define_internal_error, define_sensitive_error};

// General.
define_sensitive_error!(
    NotActive,
    "In-app-purchase exists, but is not currently valid / active."
);

// Google Play Developer API.
define_internal_error!(
    GooglePlayDeveloperApiKeyInvalid,
    "Invalid Google Play Developer API key: {details}.",
    { details: &str }
);
define_internal_error!(
    GooglePlayDeveloperApiError,
    "Error calling Google Play Developer API '{function_name}': {details}.",
    { function_name: &str, details: &str }
);
define_internal_error!(
    GooglePlayDeveloperApiInvalidResponse,
    "Invalid response from Google Play Developer API: {details}.",
    { details: &str }
);

// Google Cloud RTDN Notifications.
define_internal_error!(
    GoogleCloudRtdnNotificationParseError,
    "Error parsing Google Cloud RTDN notification: {details}.",
    { details: &str }
);

// App Store Server API.
define_internal_error!(
    AppStoreServerApiKeyInvalid,
    "Invalid App Store Server API key: {details}.",
    { details: &str }
);
define_internal_error!(
    AppStoreServerApiError,
    "Error calling App Store Server API '{function_name}': {details}.",
    { function_name: &str, details: &str }
);
define_internal_error!(
    AppStoreServerApiInvalidResponse,
    "Invalid response from App Store Server API: {details}.",
    { details: &str }
);

// App Store Server Notifications.
define_internal_error!(
    AppStoreServerNotificationParseError,
    "Error parsing App Store Server notification."
);

// JWS / JWT decoding and signature verification.
define_sensitive_error!(
    InvalidGoogleSignature,
    "Unable to verify the request was signed by Google (invalid component: {invalid_component}).",
    { invalid_component: &str }
);
define_sensitive_error!(
    InvalidAppleSignature,
    "Unable to verify the request was signed by Apple (invalid component: {invalid_component}).",
    { invalid_component: &str }
);
define_sensitive_error!(
    InvalidJws,
    "Unable to decode JWS payload: {details}.",
    { details: &str }
);
