#![allow(non_snake_case)]

mod proxy;

use multiversx_sc_snippets::imports::*;
use multiversx_sc_snippets::sdk;
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::Path,
};
use tokio::time::sleep;

const GATEWAY: &str = sdk::gateway::DEVNET_GATEWAY;
const STATE_FILE: &str = "state.toml";
pub static REWARD_TOKEN_ID: &[u8] = b"UTK-abcdef"; // reward token ID
pub static FARMING_TOKEN_ID: &[u8] = b"UTK-abcdef"; // farming token ID
pub static FARM_TOKEN_ID: &[u8] = b"FARM1-abcdef";
pub static OTHER_FARM_TOKEN_ID: &[u8] = b"FARM2-abcdef";
pub static UNBOND_TOKEN_ID: &[u8] = b"UNBOND1-abcdef";
pub static OTHER_UNBOND_TOKEN_ID: &[u8] = b"UNBOND2-abcdef";
pub const DIVISION_SAFETY_CONSTANT: u64 = 1_000_000_000_000;
pub const MIN_UNBOND_EPOCHS: u64 = 5;
pub const MAX_APR: u64 = 2_500; // 25%
pub const PER_BLOCK_REWARD_AMOUNT: u64 = 5_000;
pub const TOTAL_REWARDS_AMOUNT: u64 = 1_000_000_000_000;
pub const TOTAL_STAKING_TOKENS_MINTED: u64 = 1_000_000_000_000_000_000;

pub const USER_TOTAL_RIDE_TOKENS: u64 = 5_000_000_000;
pub static WITHDRAW_AMOUNT_TOO_HIGH: &str =
    "Withdraw amount is higher than the remaining uncollected rewards!";

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
        "deployConfigSc" => interact.deploy_config_sc().await,
        "callConfigFunction" => interact.call_config_function().await,
        "getConfigAddress" => interact.config_sc_address().await,
        "deployGuild" => interact.deploy_guild().await,
        "resumeGuild" => interact.resume_guild_endpoint().await,
        "getAllGuilds" => interact.get_all_guilds().await,
        "getGuildId" => interact.get_guild_id().await,
        "getRemainingRewards" => interact.remaining_rewards().await,
        "requestRewards" => interact.request_rewards().await,
        "migrateToOtherGuild" => interact.migrate_to_other_guild().await,
        "depositRewardsGuild" => interact.deposit_rewards_guild().await,
        "closeGuildNoRewardsRemaining" => interact.close_guild_no_rewards_remaining().await,
        "depositRewardsAdmins" => interact.deposit_rewards_admins().await,
        "getClosedGuilds" => interact.closed_guilds().await,
        "isAdmin" => interact.is_admin().await,
        "addAdmin" => interact.add_admin().await,
        "removeAdmin" => interact.remove_admin().await,
        "getAdmins" => interact.admins().await,
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
            "mxsc:../output/guild-factory.mxsc.json",
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
        let guild_sc_source_address = bech32::decode("");
        let farming_token_id = TokenIdentifier::from_esdt_bytes(&b""[..]);
        let division_safety_constant = BigUint::<StaticApi>::from(0u128);
        let admins = MultiValueVec::from(vec![bech32::decode("")]);

        let new_address = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .gas(30_000_000u64)
            .typed(proxy::GuildFactoryProxy)
            .init(guild_sc_source_address, farming_token_id, division_safety_constant, admins)
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
            .typed(proxy::GuildFactoryProxy)
            .upgrade()
            .code(&self.contract_code)
            .code_metadata(CodeMetadata::UPGRADEABLE)
            .returns(ReturnsNewAddress)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn deploy_config_sc(&mut self) {
        let config_init_args = InitArgs::<StaticApi>::default();
        let config_sc_code = ManagedBuffer::new_from_bytes(&b""[..]);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::GuildFactoryProxy)
            .deploy_config_sc(config_init_args, config_sc_code)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn call_config_function(&mut self) {
        let function_name = ManagedBuffer::new_from_bytes(&b""[..]);
        let args = MultiValueVec::from(vec![ManagedBuffer::new_from_bytes(&b""[..])]);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::GuildFactoryProxy)
            .call_config_function(function_name, args)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn config_sc_address(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::GuildFactoryProxy)
            .config_sc_address()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn deploy_guild(&mut self) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::GuildFactoryProxy)
            .deploy_guild()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn resume_guild_endpoint(&mut self) {
        let guild = bech32::decode("");

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::GuildFactoryProxy)
            .resume_guild_endpoint(guild)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn get_all_guilds(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::GuildFactoryProxy)
            .get_all_guilds()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn get_guild_id(&mut self) {
        let guild_address = bech32::decode("");

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::GuildFactoryProxy)
            .get_guild_id(guild_address)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn remaining_rewards(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::GuildFactoryProxy)
            .remaining_rewards()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn request_rewards(&mut self) {
        let amount = BigUint::<StaticApi>::from(0u128);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::GuildFactoryProxy)
            .request_rewards(amount)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn migrate_to_other_guild(&mut self) {
        let token_id = String::new();
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(0u128);

        let guild = bech32::decode("");
        let original_caller = bech32::decode("");

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::GuildFactoryProxy)
            .migrate_to_other_guild(guild, original_caller)
            .payment((TokenIdentifier::from(token_id.as_str()), token_nonce, token_amount))
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn deposit_rewards_guild(&mut self) {
        let token_id = String::new();
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(0u128);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::GuildFactoryProxy)
            .deposit_rewards_guild()
            .payment((TokenIdentifier::from(token_id.as_str()), token_nonce, token_amount))
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn close_guild_no_rewards_remaining(&mut self) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::GuildFactoryProxy)
            .close_guild_no_rewards_remaining()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn deposit_rewards_admins(&mut self) {
        let token_id = String::new();
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(0u128);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::GuildFactoryProxy)
            .deposit_rewards_admins()
            .payment((TokenIdentifier::from(token_id.as_str()), token_nonce, token_amount))
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn closed_guilds(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::GuildFactoryProxy)
            .closed_guilds()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn is_admin(&mut self) {
        let address = bech32::decode("");

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::GuildFactoryProxy)
            .is_admin(address)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn add_admin(&mut self) {
        let address = bech32::decode("");

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::GuildFactoryProxy)
            .add_admin(address)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn remove_admin(&mut self) {
        let address = bech32::decode("");

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::GuildFactoryProxy)
            .remove_admin(address)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn admins(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::GuildFactoryProxy)
            .admins()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    #[tokio::test]
    #[ignore = "run on demand"]
    async fn deployConfigSCTest(){

        let guild_sc_source_address = bech32::decode("");
        let farming_token_id = TokenIdentifier::from_esdt_bytes(&b""[..]);
        let division_safety_constant = BigUint::<StaticApi>::from(0u128);
        let admins = MultiValueVec::from(vec![bech32::decode("")]);

        let mut interact = ContractInteract::new().await;
        interact
            .deploy(
                &Bech32Address::from_bech32_string(config.current_address()),
                farming_token_id,
                division_safety_constant,
                admins
            )
            .await;
    }

    #[tokio::test]
    async fn deployGuildTest(){
        let mut interact = ContractInteract::new().await;
        interact.deploy_guild().await

    }

    #[tokio::test]
    async fn closeGuildTest(){
        let mut interact = ContractInteract::new().await;
  
        let mut farm_setup = FarmStakingSetup::new(
            guild_sc::contract_obj,
            guild_sc_config::contract_obj,
            guild_factory::contract_obj,
        );

        let AMOUNT_FOR_FARM = 50_000_000;

        interact.stake_farm(AMOUNT_FOR_FARM).await;

        interact.close_guild_no_rewards_remaining().await;


    }

    #[tokio::test]
    async fn migrationTest(){

        let mut interact = ContractInteract::new().await;

        interact.migrate_to_other_guild().await;
        
    }
}
