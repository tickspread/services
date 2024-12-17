pub mod circles_detection;
pub mod web3_provider;

use anyhow::Result;
use std::sync::Arc;

use ethcontract::web3::transports::Http;
use ethcontract::web3::Web3;
use ethcontract::H160;

use model::order::Order;
use crate::config::circles_config::CirclesConfig;
use crate::solver::circles_detection::{identify_crc_orders, match_crc_pairs};
use crate::solver::web3_provider::Web3Provider;

pub struct Solver {
    pub web3: Arc<Web3Provider<Http>>,
    pub circles_config: CirclesConfig,
}

impl Solver {
    pub async fn solve_orders(&self, orders: Vec<Order>) -> Result<()> {
        let crc_orders = identify_crc_orders(self.web3.as_ref(), &self.circles_config, orders).await?;
        let pairs = match_crc_pairs(&crc_orders);

        if pairs.is_empty() {
            println!("No CRC pairs found.");
        } else {
            println!("Found CRC pairs: {:?}", pairs);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use model::order::OrderData;

    #[tokio::test]
    async fn test_solve_orders_no_pairs() {
        let transport = Http::new("http://localhost:8545").unwrap();
        let web3 = Web3::new(transport);
        let provider = Arc::new(Web3Provider::new(web3));

        let circles_config = CirclesConfig::new(vec![]);

        let solver = Solver {
            web3: provider,
            circles_config,
        };

        let orders = vec![];

        let result = solver.solve_orders(orders).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_solve_orders_with_orders_but_no_crc() {
        let transport = Http::new("http://localhost:8545").unwrap();
        let web3 = Web3::new(transport);
        let provider = Arc::new(Web3Provider::new(web3));

        let known_hub: H160 = "0x1111111111111111111111111111111111111111".parse().unwrap();
        let circles_config = CirclesConfig::new(vec![known_hub]);

        let solver = Solver {
            web3: provider,
            circles_config,
        };

        let order = Order {
            data: OrderData {
                sell_token: H160::from_low_u64_be(0x01),
                buy_token: H160::from_low_u64_be(0x02),
                ..Default::default()
            },
            ..Default::default()
        };

        let orders = vec![order];

        let result = solver.solve_orders(orders).await;
        assert!(result.is_ok());
    }
}

// To run tests:
// cargo test -p solver
