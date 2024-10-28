#[derive(Debug)]
pub struct IapSubscriptionId {
    pub sku: String,
}

#[derive(Debug)]
pub struct IapConsumableId {
    pub sku: String,
}

#[derive(Debug)]
pub struct IapNonConsumableId {
    pub sku: String,
}

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
            &self.sku
        }
    }

    impl IapProductId for IapConsumableId {
        fn product_type() -> _ProductIdType {
            _ProductIdType::Consumable
        }
        fn sku(&self) -> &str {
            &self.sku
        }
    }

    impl IapProductId for IapNonConsumableId {
        fn product_type() -> _ProductIdType {
            _ProductIdType::NonConsumable
        }
        fn sku(&self) -> &str {
            &self.sku
        }
    }
}
