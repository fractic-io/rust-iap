use fractic_generic_server_error::{
    define_internal_error_type, GenericServerError, GenericServerErrorTrait,
};

// Google Play Developer API.
define_internal_error_type!(
    GooglePlayDeveloperApiKeyInvalid,
    "Invalid Google Play Developer API key."
);
define_internal_error_type!(
    GooglePlayDeveloperApiError,
    "Error calling out to Google Play Developer API."
);

// Google Cloud RTDN Notifications.
define_internal_error_type!(
    GoogleCloudRtdnNotificationParseError,
    "Error parsing Google Cloud RTDN notification."
);

// App Store Server API.
define_internal_error_type!(
    AppStoreServerApiKeyInvalid,
    "Invalid App Store Server API key."
);
define_internal_error_type!(
    AppStoreServerApiError,
    "Error calling out to App Store Server API."
);

// App Store Server Notifications.
define_internal_error_type!(
    AppStoreServerNotificationParseError,
    "Error parsing App Store Server notification."
);
