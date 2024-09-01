#![allow(non_snake_case)]

mod proxy;

use multiversx_sc_snippets::imports::*;
use multiversx_sc_snippets::sdk;
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::Path,
};


const GATEWAY: &str = sdk::blockchain::DEVNET_GATEWAY;
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
        "mergeFarmTokens" => interact.merge_farm_tokens_endpoint().await,
        "checkLocalRolesSet" => interact.check_local_roles_set().await,
        "calculateRewardsForGivenPosition" => interact.calculate_rewards_for_given_position().await,
        "topUpRewards" => interact.top_up_rewards().await,
        "startProduceRewards" => interact.start_produce_rewards_endpoint().await,
        "getAccumulatedRewards" => interact.accumulated_rewards().await,
        "getRewardCapacity" => interact.reward_capacity().await,
        "getGuildMasterRewardPerShare" => interact.guild_master_rps().await,
        "getUserRewardPerShare" => interact.user_rps().await,
        "getRewardReserve" => interact.reward_reserve().await,
        "getFarmingTokenId" => interact.farming_token_id().await,
        "getRewardTokenId" => interact.reward_token_id().await,
        "getPerBlockRewardAmount" => interact.per_block_reward_amount().await,
        "getLastRewardBlockNonce" => interact.last_reward_block_nonce().await,
        "getDivisionSafetyConstant" => interact.division_safety_constant().await,
        "registerFarmToken" => interact.register_farm_token().await,
        "setTransferRoleFarmToken" => interact.set_transfer_role_farm_token().await,
        "getFarmTokenId" => interact.farm_token().await,
        "getFarmTokenSupply" => interact.farm_token_supply().await,
        "addToPauseWhitelist" => interact.add_to_pause_whitelist().await,
        "removeFromPauseWhitelist" => interact.remove_from_pause_whitelist().await,
        "pause" => interact.pause().await,
        "resume" => interact.resume().await,
        "getState" => interact.state().await,
        "addAdmin" => interact.add_admin_endpoint().await,
        "removeAdmin" => interact.remove_admin_endpoint().await,
        "updateOwnerOrAdmin" => interact.update_owner_or_admin_endpoint().await,
        "getPermissions" => interact.permissions().await,
        "stakeFarm" => interact.stake_farm_endpoint().await,
        "claimRewards" => interact.claim_rewards().await,
        "compoundRewards" => interact.compound_rewards().await,
        "unstakeFarm" => interact.unstake_farm().await,
        "unbondFarm" => interact.unbond_farm().await,
        "cancelUnbond" => interact.cancel_unbond().await,
        "registerUnbondToken" => interact.register_unbond_token().await,
        "setTransferRoleUnbondToken" => interact.set_transfer_role_unbond_token().await,
        "getUnbondTokenId" => interact.unbond_token().await,
        "getUserStakedTokens" => interact.get_user_staked_tokens().await,
        "closeGuild" => interact.close_guild().await,
        "migrateToOtherGuild" => interact.migrate_to_other_guild().await,
        "isGuildClosing" => interact.guild_closing().await,
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
            "mxsc:../output/guild-sc.mxsc.json",
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
        let farming_token_id = TokenIdentifier::from_esdt_bytes(&b""[..]);
        let division_safety_constant = BigUint::<StaticApi>::from(0u128);
        let config_sc_address = bech32::decode("");
        let guild_master = bech32::decode("");
        let admins = MultiValueVec::from(vec![bech32::decode("")]);

        let new_address = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .typed(proxy::FarmStakingProxy)
            .init(farming_token_id, division_safety_constant, config_sc_address, guild_master, admins)
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

    async fn merge_farm_tokens_endpoint(&mut self) {
        let token_id = String::new();
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(0u128);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::FarmStakingProxy)
            .merge_farm_tokens_endpoint()
            .payment((TokenIdentifier::from(token_id.as_str()), token_nonce, token_amount))
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn check_local_roles_set(&mut self) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::FarmStakingProxy)
            .check_local_roles_set()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn calculate_rewards_for_given_position(&mut self) {
        let user = bech32::decode("");
        let farm_token_amount = BigUint::<StaticApi>::from(0u128);
        let attributes = PlaceholderInput;

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::FarmStakingProxy)
            .calculate_rewards_for_given_position(user, farm_token_amount, attributes)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn top_up_rewards(&mut self) {
        let token_id = String::new();
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(0u128);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::FarmStakingProxy)
            .top_up_rewards()
            .payment((TokenIdentifier::from(token_id.as_str()), token_nonce, token_amount))
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn start_produce_rewards_endpoint(&mut self) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::FarmStakingProxy)
            .start_produce_rewards_endpoint()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn accumulated_rewards(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::FarmStakingProxy)
            .accumulated_rewards()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn reward_capacity(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::FarmStakingProxy)
            .reward_capacity()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn guild_master_rps(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::FarmStakingProxy)
            .guild_master_rps()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn user_rps(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::FarmStakingProxy)
            .user_rps()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn reward_reserve(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::FarmStakingProxy)
            .reward_reserve()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn farming_token_id(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::FarmStakingProxy)
            .farming_token_id()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn reward_token_id(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::FarmStakingProxy)
            .reward_token_id()
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
            .typed(proxy::FarmStakingProxy)
            .per_block_reward_amount()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn last_reward_block_nonce(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::FarmStakingProxy)
            .last_reward_block_nonce()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn division_safety_constant(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::FarmStakingProxy)
            .division_safety_constant()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn register_farm_token(&mut self) {
        let egld_amount = BigUint::<StaticApi>::from(0u128);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::FarmStakingProxy)
            .register_farm_token()
            .egld(egld_amount)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn set_transfer_role_farm_token(&mut self) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::FarmStakingProxy)
            .set_transfer_role_farm_token()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn farm_token(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::FarmStakingProxy)
            .farm_token()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn farm_token_supply(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::FarmStakingProxy)
            .farm_token_supply()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn add_to_pause_whitelist(&mut self) {
        let address_list = MultiValueVec::from(vec![bech32::decode("")]);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::FarmStakingProxy)
            .add_to_pause_whitelist(address_list)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn remove_from_pause_whitelist(&mut self) {
        let address_list = MultiValueVec::from(vec![bech32::decode("")]);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::FarmStakingProxy)
            .remove_from_pause_whitelist(address_list)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn pause(&mut self) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::FarmStakingProxy)
            .pause()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn resume(&mut self) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::FarmStakingProxy)
            .resume()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn state(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::FarmStakingProxy)
            .state()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn add_admin_endpoint(&mut self) {
        let address = bech32::decode("");

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::FarmStakingProxy)
            .add_admin_endpoint(address)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn remove_admin_endpoint(&mut self) {
        let address = bech32::decode("");

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::FarmStakingProxy)
            .remove_admin_endpoint(address)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn update_owner_or_admin_endpoint(&mut self) {
        let previous_owner = bech32::decode("");

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::FarmStakingProxy)
            .update_owner_or_admin_endpoint(previous_owner)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn permissions(&mut self) {
        let address = bech32::decode("");

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::FarmStakingProxy)
            .permissions(address)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn stake_farm_endpoint(&mut self) {
        let token_id = String::new();
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(0u128);

        let opt_original_caller = OptionalValue::Some(bech32::decode(""));

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::FarmStakingProxy)
            .stake_farm_endpoint(opt_original_caller)
            .payment((TokenIdentifier::from(token_id.as_str()), token_nonce, token_amount))
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn claim_rewards(&mut self) {
        let token_id = String::new();
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(0u128);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::FarmStakingProxy)
            .claim_rewards()
            .payment((TokenIdentifier::from(token_id.as_str()), token_nonce, token_amount))
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn compound_rewards(&mut self) {
        let token_id = String::new();
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(0u128);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::FarmStakingProxy)
            .compound_rewards()
            .payment((TokenIdentifier::from(token_id.as_str()), token_nonce, token_amount))
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn unstake_farm(&mut self) {
        let token_id = String::new();
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(0u128);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::FarmStakingProxy)
            .unstake_farm()
            .payment((TokenIdentifier::from(token_id.as_str()), token_nonce, token_amount))
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn unbond_farm(&mut self) {
        let token_id = String::new();
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(0u128);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::FarmStakingProxy)
            .unbond_farm()
            .payment((TokenIdentifier::from(token_id.as_str()), token_nonce, token_amount))
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn cancel_unbond(&mut self) {
        let token_id = String::new();
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(0u128);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::FarmStakingProxy)
            .cancel_unbond()
            .payment((TokenIdentifier::from(token_id.as_str()), token_nonce, token_amount))
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn register_unbond_token(&mut self) {
        let egld_amount = BigUint::<StaticApi>::from(0u128);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::FarmStakingProxy)
            .register_unbond_token()
            .egld(egld_amount)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn set_transfer_role_unbond_token(&mut self) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::FarmStakingProxy)
            .set_transfer_role_unbond_token()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn unbond_token(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::FarmStakingProxy)
            .unbond_token()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn get_user_staked_tokens(&mut self) {
        let user = bech32::decode("");

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::FarmStakingProxy)
            .get_user_staked_tokens(user)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn close_guild(&mut self) {
        let token_id = String::new();
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(0u128);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::FarmStakingProxy)
            .close_guild()
            .payment((TokenIdentifier::from(token_id.as_str()), token_nonce, token_amount))
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

        let guild_address = bech32::decode("");

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::FarmStakingProxy)
            .migrate_to_other_guild(guild_address)
            .payment((TokenIdentifier::from(token_id.as_str()), token_nonce, token_amount))
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn guild_closing(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::FarmStakingProxy)
            .guild_closing()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

}
