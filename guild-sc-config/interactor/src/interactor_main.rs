#![allow(non_snake_case)]

mod proxy;

use multiversx_sc_snippets::imports::*;
use multiversx_sc_snippets::sdk;
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::Path,
};


const GATEWAY: &str = sdk::gateway::DEVNET_GATEWAY;
const STATE_FILE: &str = "state.toml";


#[tokio::main]
async fn main() {
    env_logger::init();

    let mut args = std::env::args();
    let _ = args.next();
    let cmd = args.next().expect("at least one argument required");
    let mut interact = ContractInteract::new().await;
    match cmd.as_str() {
        "deploy" => interact.deploy().await,
        "upgrade" => interact.upgrade().await,
        "setMaxStakedTokens" => interact.set_max_staked_tokens().await,
        "addGuildMasterTiers" => interact.add_guild_master_tiers().await,
        "updateGuildMasterTiers" => interact.update_guild_master_tiers().await,
        "setGuildMasterTierApr" => interact.set_guild_master_tier_apr().await,
        "addUserTiers" => interact.add_user_tiers().await,
        "updateUserTiers" => interact.update_user_tiers().await,
        "setUserTierApr" => interact.set_user_tier_apr().await,
        "getGuildMasterTiers" => interact.guild_master_tiers().await,
        "getUserTiers" => interact.user_tiers().await,
        "setMinStakeUser" => interact.set_min_stake_user().await,
        "setMinStakeGuildMaster" => interact.set_min_stake_guild_master().await,
        "setTotalStakingTokenMinted" => interact.set_total_staking_token_minted().await,
        "increaseStakedTokens" => interact.increase_staked_tokens().await,
        "decreaseStakedTokens" => interact.decrease_staked_tokens().await,
        "setSecondsPerBlock" => interact.set_seconds_per_block().await,
        "setPerBlockRewardAmount" => interact.set_per_block_reward_amount().await,
        "pauseAllGuilds" => interact.pause_all_guilds().await,
        "unpauseAllGuilds" => interact.unpause_all_guilds().await,
        "getMaxStakedTokens" => interact.max_staked_tokens().await,
        "getMinUnbondEpochsUser" => interact.min_unbond_epochs_user().await,
        "getMinUnbondEpochsGuildMaster" => interact.min_unbond_epochs_guild_master().await,
        "getMinStakeUser" => interact.min_stake_user().await,
        "getMinStakeGuildMaster" => interact.min_stake_guild_master().await,
        "getTotalStakingTokenMinted" => interact.total_staking_token_minted().await,
        "getTotalStakingTokenStaked" => interact.total_staking_token_staked().await,
        "getBaseFarmTokenId" => interact.base_farm_token_id().await,
        "getBaseUnbondTokenId" => interact.base_unbond_token_id().await,
        "getBaseTokenDisplayName" => interact.base_token_display_name().await,
        "getTokenDecimals" => interact.tokens_decimals().await,
        "getSecondsPerBlock" => interact.seconds_per_block().await,
        "getPerBlockRewardAmount" => interact.per_block_reward_amount().await,
        "areAllGuildsPaused" => interact.global_pause_status().await,
        _ => panic!("unknown command: {}", &cmd),
    }
}


#[derive(Debug, Default, Serialize, Deserialize)]
struct State {
    contract_address: Option<Bech32Address>
}

impl State {
        // Deserializes state from file
        pub fn load_state() -> Self {
            if Path::new(STATE_FILE).exists() {
                let mut file = std::fs::File::open(STATE_FILE).unwrap();
                let mut content = String::new();
                file.read_to_string(&mut content).unwrap();
                toml::from_str(&content).unwrap()
            } else {
                Self::default()
            }
        }
    
        /// Sets the contract address
        pub fn set_address(&mut self, address: Bech32Address) {
            self.contract_address = Some(address);
        }
    
        /// Returns the contract address
        pub fn current_address(&self) -> &Bech32Address {
            self.contract_address
                .as_ref()
                .expect("no known contract, deploy first")
        }
    }
    
    impl Drop for State {
        // Serializes state to file
        fn drop(&mut self) {
            let mut file = std::fs::File::create(STATE_FILE).unwrap();
            file.write_all(toml::to_string(self).unwrap().as_bytes())
                .unwrap();
        }
    }

struct ContractInteract {
    interactor: Interactor,
    wallet_address: Address,
    contract_code: BytesValue,
    state: State
}

impl ContractInteract {
    async fn new() -> Self {
        let mut interactor = Interactor::new(GATEWAY).await;
        let wallet_address = interactor.register_wallet(test_wallets::alice());
        
        let contract_code = BytesValue::interpret_from(
            "mxsc:../output/guild-sc-config.mxsc.json",
            &InterpreterContext::default(),
        );

        ContractInteract {
            interactor,
            wallet_address,
            contract_code,
            state: State::load_state()
        }
    }

    async fn deploy(&mut self) {
        let args = InitArgs::<StaticApi>::default();

        let new_address = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .gas(30_000_000u64)
            .typed(proxy::GuildScConfigProxy)
            .init(args)
            .code(&self.contract_code)
            .returns(ReturnsNewAddress)
            .prepare_async()
            .run()
            .await;
        let new_address_bech32 = bech32::encode(&new_address);
        self.state
            .set_address(Bech32Address::from_bech32_string(new_address_bech32.clone()));

        println!("new address: {new_address_bech32}");
    }

    async fn upgrade(&mut self) {
        let response = self
            .interactor
            .tx()
            .to(self.state.current_address())
            .from(&self.wallet_address)
            .gas(30_000_000u64)
            .typed(proxy::GuildScConfigProxy)
            .upgrade()
            .code(&self.contract_code)
            .code_metadata(CodeMetadata::UPGRADEABLE)
            .returns(ReturnsNewAddress)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn set_max_staked_tokens(&mut self) {
        let max_staked_tokens = BigUint::<StaticApi>::from(0u128);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::GuildScConfigProxy)
            .set_max_staked_tokens(max_staked_tokens)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn add_guild_master_tiers(&mut self) {
        let tiers = MultiValueVec::from(vec![MultiValue2::<BigUint<StaticApi>, u64>::from((BigUint::<StaticApi>::from(0u128), 0u64))]);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::GuildScConfigProxy)
            .add_guild_master_tiers(tiers)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn update_guild_master_tiers(&mut self) {
        let tiers = MultiValueVec::from(vec![MultiValue2::<BigUint<StaticApi>, u64>::from((BigUint::<StaticApi>::from(0u128), 0u64))]);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::GuildScConfigProxy)
            .update_guild_master_tiers(tiers)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn set_guild_master_tier_apr(&mut self) {
        let max_stake = BigUint::<StaticApi>::from(0u128);
        let new_apr = 0u64;

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::GuildScConfigProxy)
            .set_guild_master_tier_apr(max_stake, new_apr)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn add_user_tiers(&mut self) {
        let tiers = MultiValueVec::from(vec![MultiValue2::<u64, u64>::from((0u64, 0u64))]);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::GuildScConfigProxy)
            .add_user_tiers(tiers)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn update_user_tiers(&mut self) {
        let tiers = MultiValueVec::from(vec![MultiValue2::<u64, u64>::from((0u64, 0u64))]);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::GuildScConfigProxy)
            .update_user_tiers(tiers)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn set_user_tier_apr(&mut self) {
        let max_percentage_staked = 0u64;
        let new_apr = 0u64;

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::GuildScConfigProxy)
            .set_user_tier_apr(max_percentage_staked, new_apr)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn guild_master_tiers(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::GuildScConfigProxy)
            .guild_master_tiers()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn user_tiers(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::GuildScConfigProxy)
            .user_tiers()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn set_min_stake_user(&mut self) {
        let min_stake = BigUint::<StaticApi>::from(0u128);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::GuildScConfigProxy)
            .set_min_stake_user(min_stake)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn set_min_stake_guild_master(&mut self) {
        let min_stake = BigUint::<StaticApi>::from(0u128);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::GuildScConfigProxy)
            .set_min_stake_guild_master(min_stake)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn set_total_staking_token_minted(&mut self) {
        let total_minted = BigUint::<StaticApi>::from(0u128);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::GuildScConfigProxy)
            .set_total_staking_token_minted(total_minted)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn increase_staked_tokens(&mut self) {
        let amount = BigUint::<StaticApi>::from(0u128);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::GuildScConfigProxy)
            .increase_staked_tokens(amount)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn decrease_staked_tokens(&mut self) {
        let amount = BigUint::<StaticApi>::from(0u128);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::GuildScConfigProxy)
            .decrease_staked_tokens(amount)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn set_seconds_per_block(&mut self) {
        let new_seconds_per_block = 0u64;

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::GuildScConfigProxy)
            .set_seconds_per_block(new_seconds_per_block)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn set_per_block_reward_amount(&mut self) {
        let new_per_block_reward_amount = BigUint::<StaticApi>::from(0u128);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::GuildScConfigProxy)
            .set_per_block_reward_amount(new_per_block_reward_amount)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn pause_all_guilds(&mut self) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::GuildScConfigProxy)
            .pause_all_guilds()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn unpause_all_guilds(&mut self) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::GuildScConfigProxy)
            .unpause_all_guilds()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn max_staked_tokens(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::GuildScConfigProxy)
            .max_staked_tokens()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn min_unbond_epochs_user(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::GuildScConfigProxy)
            .min_unbond_epochs_user()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn min_unbond_epochs_guild_master(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::GuildScConfigProxy)
            .min_unbond_epochs_guild_master()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn min_stake_user(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::GuildScConfigProxy)
            .min_stake_user()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn min_stake_guild_master(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::GuildScConfigProxy)
            .min_stake_guild_master()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn total_staking_token_minted(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::GuildScConfigProxy)
            .total_staking_token_minted()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn total_staking_token_staked(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::GuildScConfigProxy)
            .total_staking_token_staked()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn base_farm_token_id(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::GuildScConfigProxy)
            .base_farm_token_id()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn base_unbond_token_id(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::GuildScConfigProxy)
            .base_unbond_token_id()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn base_token_display_name(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::GuildScConfigProxy)
            .base_token_display_name()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn tokens_decimals(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::GuildScConfigProxy)
            .tokens_decimals()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn seconds_per_block(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::GuildScConfigProxy)
            .seconds_per_block()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn per_block_reward_amount(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::GuildScConfigProxy)
            .per_block_reward_amount()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn global_pause_status(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::GuildScConfigProxy)
            .global_pause_status()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

}
