use std::ops::Div;
use std::str::FromStr;
use std::sync::LazyLock;

use alloy::network::Ethereum;
use alloy::primitives::utils::{format_ether, format_units};
use alloy::providers::RootProvider;
use rust_decimal::Decimal;

use crate::core::consts;
use crate::util::LibResult;

pub static TOKEN: LazyLock<Token> = LazyLock::new(|| Token::new());

pub struct Token {
    factory: consts::FACTORY::FACTORYInstance<(), RootProvider>,
    provider: RootProvider<Ethereum>,
}

impl Token {
    pub fn new() -> Self {
        let url = consts::PROVIDER
            .parse()
            .expect("Could not parse the Provider URL");
        let factory_address = consts::FACTORY_CONTRACT_ADDR
            .parse()
            .expect("Could not parse the Factory Address");
        let provider = RootProvider::new_http(url);
        let factory = consts::FACTORY::new(factory_address, provider.clone());
        Self { provider, factory }
    }

    pub async fn balance_of(&self, token: &str, user: &str) -> LibResult<Decimal> {
        let token_address = token.parse()?;
        let user_address = user.parse()?;
        let contract = consts::ERC20::new(token_address, self.provider.clone());
        let balance = contract.balanceOf(user_address).call().await?._0;
        println!("Balance: {}", balance);
        let amount = Decimal::from_str(&format_ether(balance))?;
        println!("Amount: {}", amount);
        Ok(amount)
    }

    pub async fn total_supply(&self, token: &str) -> LibResult<Decimal> {
        let token_address = token.parse()?;
        let contract = consts::ERC20::new(token_address, self.provider.clone());
        let balance = contract.totalSupply().call().await?._0;
        println!("Balance: {}", balance);
        let amount = Decimal::from_str(&format_ether(balance))?;
        println!("Amount: {}", amount);
        Ok(amount)
    }

    pub async fn oracle_price(&self, token: &str) -> LibResult<Decimal> {
        println!("Token: {}", token);
        Ok(Decimal::new(1, 0))
    }

    pub async fn curve_process(&self, token: &str) -> LibResult<Decimal> {
        let token_address = token.parse()?;
        let now_point = self
            .factory
            .tokenInfo(token_address)
            .call()
            .await?
            .data
            .totalShare;
        let end_point = self
            .factory
            .launchPointShare(token_address)
            .call()
            .await?
            ._0;
        let process = Decimal::from_str(&format_units(now_point.div(end_point), 2)?)?;
        println!("Process: {}", process);
        Ok(process)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_balance_of() {
        dotenvy::dotenv().expect("");
        let balance = TOKEN
            .balance_of(
                "0xB2284B8eee1E364F6bD4fA814e64303819a16aE8",
                "0xF41BBb59B4291Ae8711ef276DdC0a26E6AD0137C",
            )
            .await;
        assert!(balance.is_ok());
    }
}
