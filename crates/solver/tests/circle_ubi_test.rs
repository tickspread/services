use anyhow::Result;
use contracts::circles_hub::CirclesHub;
use primitive_types::{H160, U256};
use solver::{
    interactions::circle_ubi::CircleUbiTransitiveInteraction,
    liquidity::{
        circle_ubi::{CircleUbiOrder, CircleUbiSettlementHandler},
        AmmOrderExecution,
    },
    settlement::{SettlementEncoder, InternalizationStrategy},
    interactions::allowances::{AllowanceManaging, ApprovalRequest},
};
use std::sync::Arc;

struct MockAllowanceManager;

#[async_trait::async_trait]
impl AllowanceManaging for MockAllowanceManager {
    async fn get_approval(&self, _: &ApprovalRequest) -> Result<Option<Box<dyn Interaction>>> {
        Ok(None)
    }
}

#[derive(Debug)]
struct TokenAmount {
    token: H160,
    amount: U256,
}

#[tokio::test]
async fn test_circle_ubi_interaction_encoding() -> Result<()> {
    let circles_hub = CirclesHub::at(
        web3::Web3::new(web3::transports::Http::new("http://localhost:8545")?),
        H160::from_low_u64_be(0x1234),
    );

    let handler = CircleUbiSettlementHandler {
        allowance_manager: Arc::new(MockAllowanceManager),
        circles_hub: circles_hub.clone(),
    };

    let order = CircleUbiOrder {
        input_token_owner: H160::from_low_u64_be(0xABC),
        input_src: H160::from_low_u64_be(0xABC),
        output_dest: H160::from_low_u64_be(0xDEF),
        amount: U256::from(1000),
        settlement_handling: Arc::new(handler),
    };

    let execution = AmmOrderExecution {
        input_max: TokenAmount {
            token: H160::from_low_u64_be(0xABC),
            amount: U256::from(1000),
        },
        output: TokenAmount {
            token: H160::from_low_u64_be(0xDEF),
            amount: U256::from(1000),
        },
        internalizable: false,
    };

    let mut encoder = SettlementEncoder::new(Default::default());
    order.settlement_handling().encode(execution, &mut encoder)?;
    let settlement = encoder.finish(InternalizationStrategy::EncodeAllInteractions);

    // Verify that settlement contains the interaction as expected
    assert_eq!(settlement.interactions.len(), 1);
    assert_eq!(settlement.interactions[0].len(), 1);

    Ok(())
} 