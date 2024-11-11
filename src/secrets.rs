use fractic_env_config::{define_secret_key, define_secrets_config, SecretsConfigEnum};

define_secret_key!(GOOGLE_API_KEY);
define_secret_key!(APPLE_API_KEY);
define_secret_key!(APPLE_KEY_ID);
define_secret_key!(APPLE_ISSUER_ID);

define_secrets_config!(
    IapSecretsConfig,
    GoogleApiKey => GOOGLE_API_KEY,
    AppleApiKey => APPLE_API_KEY,
    AppleKeyId => APPLE_KEY_ID,
    AppleIssuerId => APPLE_ISSUER_ID,
);
