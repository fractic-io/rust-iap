use fractic_generic_server_error::{
    define_internal_error_type, GenericServerError, GenericServerErrorTrait,
};

define_internal_error_type!(ApiKeyInvalid, "Invalid API key.");
define_internal_error_type!(
    GooglePlayDeveloperApiError,
    "Error calling out to Google Play Developer API."
);
define_internal_error_type!(
    AppStoreServerApiError,
    "Error calling out to App Store Server API."
);
