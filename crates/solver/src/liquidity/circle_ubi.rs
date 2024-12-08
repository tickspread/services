use std::sync::Arc;
use primitive_types::{H160, U256};
use crate::{
    liquidity::{Settleable, SettlementHandling, AmmOrderExecution},
    settlement::SettlementEncoder,
    interactions::circle_ubi::CircleUbiTransitiveInteraction,
    interactions::Interaction,
};
use anyhow::Result;
use contracts::circles_hub::CirclesHub;
use crate::interactions::allowances::{AllowanceManaging, ApprovalRequest};

pub struct CircleUbiOrder {
    pub input_token_owner: H160,
    pub input_src: H160,
    pub output_dest: H160,
    pub amount: U256,
    pub settlement_handling: Arc<dyn SettlementHandling<Self>>,
}

impl Settleable for CircleUbiOrder {
    type Execution = AmmOrderExecution;

    fn settlement_handling(&self) -> &dyn SettlementHandling<Self> {
        &*self.settlement_handling
    }
}

// Settlement Handler
pub struct CircleUbiSettlementHandler {
    pub allowance_manager: Arc<dyn AllowanceManaging>,
    pub circles_hub: CirclesHub,
}

impl SettlementHandling<CircleUbiOrder> for CircleUbiSettlementHandler {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn encode(&self, execution: AmmOrderExecution, encoder: &mut SettlementEncoder) -> Result<()> {
        // Handle approval if needed for non-Circles tokens
        let approval_req = ApprovalRequest {
            token: execution.input_max.token,
            spender: self.circles_hub.address(),
            amount: execution.input_max.amount,
        };
        if let Some(approval) = self.allowance_manager.get_approval(&approval_req)? {
            encoder.append_to_execution_plan(Arc::new(approval));
        }

        // Add the Circle UBI interaction
        let interaction = CircleUbiTransitiveInteraction {
            circles_hub: self.circles_hub.clone(),
            token_owners: vec![execution.input_max.token],
            srcs: vec![execution.input_max.token],
            dests: vec![execution.output.token],
            amounts: vec![execution.output.amount],
        };
        encoder.append_to_execution_plan(Arc::new(interaction));

        Ok(())
    }
} 