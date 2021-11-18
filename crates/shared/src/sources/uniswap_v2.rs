//! Uniswap V2 baseline liquidity source implementation.

pub mod pair_provider;
pub mod pool_cache;
pub mod pool_fetching;

use self::pair_provider::PairProvider;
use crate::Web3;
use anyhow::Result;
use contracts::UniswapV2Factory;
use ethcontract::H160;
use hex_literal::hex;

const INIT_CODE_DIGEST: [u8; 32] =
    hex!("96e8ac4277198ff8b6f785478aa9a39f403cb768dd02cbee326c3e7da348845f");

/// Creates the pair provider for the specified Web3 instance.
pub async fn get_pair_provider(web3: &Web3) -> Result<PairProvider> {
    let factory = UniswapV2Factory::deployed(web3).await?;
    Ok(pair_provider_for_factory(factory.address()))
}

/// Returns a pair provider for the specified factory contract address.
pub fn pair_provider_for_factory(factory_address: H160) -> PairProvider {
    PairProvider {
        factory: factory_address,
        init_code_digest: INIT_CODE_DIGEST,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethcontract_mock::Mock;
    use model::TokenPair;

    #[tokio::test]
    async fn test_create2_mainnet() {
        // https://info.uniswap.org/pair/0x3e8468f66d30fc99f745481d4b383f89861702c6
        let mainnet_pair_provider = get_pair_provider(&Mock::new(1).web3()).await.unwrap();
        let mainnet_pair = TokenPair::new(
            addr!("6810e776880c02933d47db1b9fc05908e5386b96"),
            addr!("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"),
        )
        .unwrap();
        assert_eq!(
            mainnet_pair_provider.pair_address(&mainnet_pair),
            addr!("3e8468f66d30fc99f745481d4b383f89861702c6")
        );

        // Rinkeby
        let rinkeby_pair_provider = get_pair_provider(&Mock::new(4).web3()).await.unwrap();
        let rinkeby_pair = TokenPair::new(
            addr!("a7D1C04fAF998F9161fC9F800a99A809b84cfc9D"),
            addr!("c778417e063141139fce010982780140aa0cd5ab"),
        )
        .unwrap();
        assert_eq!(
            rinkeby_pair_provider.pair_address(&rinkeby_pair),
            addr!("9B79462e2A47487856D5521963449c573e273E79")
        );
    }
}