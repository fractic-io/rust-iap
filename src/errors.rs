use fractic_generic_server_error::{
    define_internal_error_type, GenericServerError, GenericServerErrorTrait,
};

define_internal_error_type!(
    GooglePlayDeveloperApiKeyInvalid,
    "Invalid Google Play Developer API key."
);
define_internal_error_type!(
    GooglePlayDeveloperApiError,
    "Error calling out to Google Play Developer API."
);
define_internal_error_type!(
    AppStoreServerApiKeyInvalid,
    "Invalid App Store Server API key."
);
define_internal_error_type!(
    AppStoreServerApiError,
    "Error calling out to App Store Server API."
);
