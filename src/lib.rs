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
            pub(crate) mod product_purchase_model;
            pub(crate) mod subscription_purchase_v2_model;
        }
    }
    pub(crate) mod repositories {
        pub(crate) mod iap_repository_impl;
    }
}

pub mod domain {
    pub mod entities {
        pub mod iap_purchase;
        pub mod iap_subscription;
        pub mod server_notification;
    }
    pub mod repositories {
        pub mod iap_repository;
    }
}

pub mod errors;
pub mod secrets;
pub mod util;
