use machete::models::library::item::Currency;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CurrencyOrGold {
    Currency(Currency),
    Gold(u32),
}

impl CurrencyOrGold {
    pub fn as_currency(&self) -> Currency {
        match self {
            CurrencyOrGold::Currency(c) => c.clone(),
            CurrencyOrGold::Gold(g) => Currency {
                gold: *g,
                silver: 0,
                copper: 0,
            },
        }
    }
}

impl Default for CurrencyOrGold {
    fn default() -> Self {
        CurrencyOrGold::Gold(0)
    }
}