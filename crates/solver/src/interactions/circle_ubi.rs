use anyhow::Result;
use ethcontract::Bytes;
use primitive_types::{H160, U256};
use crate::interactions::Interaction;
use contracts::circles_hub::CirclesHub;

#[derive(Debug)]
pub struct CircleUbiTransitiveInteraction {
    pub circles_hub: CirclesHub,
    pub token_owners: Vec<H160>,
    pub srcs: Vec<H160>,
    pub dests: Vec<H160>,
    pub amounts: Vec<U256>,
}

impl Interaction for CircleUbiTransitiveInteraction {
    fn encode(&self) -> (H160, U256, Bytes) {
        let method = self.circles_hub.transfer_through(
            self.token_owners.clone(),
            self.srcs.clone(),
            self.dests.clone(),
            self.amounts.clone(),
        );
        let calldata = method.tx.data.expect("no calldata").0;
        (self.circles_hub.address(), 0.into(), Bytes(calldata))
    }
} 