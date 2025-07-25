use fractic_context::define_ctx_view;

define_ctx_view!(
    name: IapCtxView,
    env {},
    secrets {
        GOOGLE_API_KEY: String,
        APPLE_API_KEY: String,
        APPLE_KEY_ID: String,
        APPLE_ISSUER_ID: String,
    },
    deps_overlay {},
    req_impl {}
);
