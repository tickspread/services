// File: crates/solver/src/solver/mod.rs

pub mod circles_detection;
pub mod web3_provider;

use anyhow::Result;
use std::sync::Arc;

use ethcontract::H160;
use model::order::Order;
use crate::config::circles_config::CirclesConfig;
use crate::solver::circles_detection::{identify_crc_orders, match_crc_pairs, CRCOrderInfo};
use crate::solver::web3_provider::Web3Provider; // Assume this exists and is imported
use ethcontract::web3::transports::Http;
use ethcontract::web3::Web3;

// The solver struct or main entry could already exist, we add solve_orders logic here:
pub struct Solver {
    web3: Arc<Web3Provider<Http>>,
    circles_config: CirclesConfig,
}

impl Solver {
    pub fn new(web3: Arc<Web3Provider<Http>>, circles_config: CirclesConfig) -> Self {
        Solver { web3, circles_config }
    }

    /// Integrate CRC order detection and matching into the solver pipeline.
    /// For now, we only log/store the pairs.
    pub async fn solve_orders(&self, orders: Vec<Order>) -> Result<()> {
        // Step 1: Identify CRC orders
        let crc_orders = identify_crc_orders(self.web3.as_ref(), &self.circles_config, orders).await?;

        // Step 2: Match CRC pairs
        let pairs = match_crc_pairs(&crc_orders);

        // Just log how many pairs we found (or store them in a variable)
        println!("Found {} CRC pairs", pairs.len());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::circles_config::CirclesConfig;
    use crate::solver::web3_provider::Web3Provider;
    use ethcontract::web3::transports::Http;
    use model::order::{Order, OrderData};
    use std::str::FromStr;

    // Mock web3 provider for testing
    fn mock_web3_provider() -> Arc<Web3Provider<Http>> {
        // In a real test you could use a local Ganache or a mock transport.
        // For simplicity, we just create a provider that points to localhost.
        let transport = Http::new("http://localhost:8545").unwrap();
        Arc::new(Web3Provider::new(Web3::new(transport)))
    }

    #[tokio::test]
    async fn test_solve_orders_no_crc() {
        let web3 = mock_web3_provider();
        let config = CirclesConfig::new(vec![]);
        let solver = Solver::new(web3, config);

        // Create a dummy order that won't be CRC
        let order = Order {
            data: OrderData {
                sell_token: H160::from_str("0x0000000000000000000000000000000000000001").unwrap(),
                buy_token: H160::from_str("0x0000000000000000000000000000000000000002").unwrap(),
                ..Default::default()
            },
            ..Default::default()
        };

        let orders = vec![order];
        let result = solver.solve_orders(orders).await;
        assert!(result.is_ok());
        // Just ensure it doesn't panic and prints "Found 0 CRC pairs".
        // Since no CRC token/hub known, no pairs found.
    }

    #[tokio::test]
    async fn test_solve_orders_crc_pairs() {
        let web3 = mock_web3_provider();
        // Suppose we have a known hub, and identify_crc_orders might detect CRC tokens.
        // In practice, you'd mock or set up the scenario. For this test, we rely on the logic
        // that if identify_crc_orders and match_crc_pairs run, we won't panic.
        let known_hub: H160 = H160::from_low_u64_be(0x1111);
        let config = CirclesConfig::new(vec![known_hub]);
        let solver = Solver::new(web3, config);

        // Create orders that might form a CRC pair.
        let order1 = Order {
            data: OrderData {
                sell_token: known_hub, // pretend this token maps to CRC
                buy_token: H160::from_low_u64_be(0x2222),
                ..Default::default()
            },
            ..Default::default()
        };

        let order2 = Order {
            data: OrderData {
                sell_token: H160::from_low_u64_be(0x2222),
                buy_token: known_hub, // opposite direction, forms a cycle
                ..Default::default()
            },
            ..Default::default()
        };

        let orders = vec![order1, order2];
        let result = solver.solve_orders(orders).await;
        assert!(result.is_ok());
        // If CRC pairs are identified, it prints how many pairs. Check no panic.
    }
}

// Command to test this file:
// cargo test -p solver --lib solver::mod
