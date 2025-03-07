// Crate-internal.
// ---

pub(crate) mod data {
    pub(crate) mod datasources {
        pub(crate) mod app_store_server_api_datasource;
        pub(crate) mod app_store_server_notification_datasource;
        pub(crate) mod google_cloud_rtdn_notification_datasource;
        pub(crate) mod google_play_developer_api_datasource;
        mod utils;
    }
    pub(crate) mod models {
        pub(crate) mod app_store_server_api {
            pub(crate) mod common;
            pub(crate) mod jws_renewal_info_decoded_payload_model;
            pub(crate) mod jws_transaction_decoded_payload_model;
            pub(crate) mod send_test_notification_response;
            pub(crate) mod transaction_info_response_model;
        }
        pub(crate) mod app_store_server_notifications {
            pub(crate) mod response_body_v2_decoded_payload_model;
            pub(crate) mod response_body_v2_model;
        }
        pub(crate) mod google_cloud_rtdn_notifications {
            pub(crate) mod developer_notification_model;
            pub(crate) mod pub_sub_model;
        }
        pub(crate) mod google_play_developer_api {
            pub(crate) mod in_app_product_model;
            pub(crate) mod product_purchase_model;
            pub(crate) mod subscription_purchase_v2_model;
        }
    }
    pub(crate) mod repositories {
        pub(crate) mod iap_repository_impl;
    }
}

pub(crate) mod domain {
    pub(crate) mod entities {
        pub(crate) mod iap_details;
        pub(crate) mod iap_product_id;
        pub(crate) mod iap_purchase_id;
        pub(crate) mod iap_update_notification;
    }
    pub(crate) mod repositories {
        pub(crate) mod iap_repository;
    }
}

// Public exports.
// ---

pub mod exports {
    // This mod represents how clients see the library, and can differ from the
    // internal structure.
    //
    // The contents of this mod are re-exported in the root of the crate.

    pub mod entities {
        pub use crate::domain::entities::iap_details::*;
        pub use crate::domain::entities::iap_product_id::*;
        pub use crate::domain::entities::iap_purchase_id::*;
        pub use crate::domain::entities::iap_update_notification::*;
    }
}
