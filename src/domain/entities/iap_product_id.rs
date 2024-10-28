#[derive(Debug, Clone)]
pub struct IapNonConsumableId(pub String);

#[derive(Debug, Clone)]
pub struct IapConsumableId(pub String);

#[derive(Debug, Clone)]
pub struct IapSubscriptionId(pub String);

// Internal type sugar:
// ----------------------------

pub(crate) mod private {
    use super::{IapConsumableId, IapNonConsumableId, IapSubscriptionId};

    pub trait IapProductId: Send + Sync {
        fn product_type() -> _ProductIdType;
        fn sku(&self) -> &str;
    }

    #[derive(Debug)]
    pub enum _ProductIdType {
        Subscription,
        Consumable,
        NonConsumable,
    }

    impl IapProductId for IapSubscriptionId {
        fn product_type() -> _ProductIdType {
            _ProductIdType::Subscription
        }
        fn sku(&self) -> &str {
            &self.0
        }
    }

    impl IapProductId for IapConsumableId {
        fn product_type() -> _ProductIdType {
            _ProductIdType::Consumable
        }
        fn sku(&self) -> &str {
            &self.0
        }
    }

    impl IapProductId for IapNonConsumableId {
        fn product_type() -> _ProductIdType {
            _ProductIdType::NonConsumable
        }
        fn sku(&self) -> &str {
            &self.0
        }
    }
}
