#![no_std]
#![allow(clippy::from_over_into)]
#![feature(trait_alias)]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use base_impl_wrapper::FarmStakingWrapper;
use common_structs::Epoch;
use contexts::storage_cache::StorageCache;
use farm_base_impl::base_traits_impl::FarmContract;
use fixed_supply_token::FixedSupplyToken;
use token_attributes::StakingFarmTokenAttributes;

pub mod base_impl_wrapper;
pub mod custom_rewards;
pub mod farm_token_roles;
pub mod tiered_rewards;
pub mod token_attributes;
pub mod unbond_token;
pub mod user_actions;

#[multiversx_sc::contract]
pub trait FarmStaking:
    custom_rewards::CustomRewardsModule
    + rewards::RewardsModule
    + config::ConfigModule
    + events::EventsModule
    + token_send::TokenSendModule
    + farm_token::FarmTokenModule
    + sc_whitelist_module::SCWhitelistModule
    + pausable::PausableModule
    + permissions_module::PermissionsModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
    + farm_base_impl::base_farm_init::BaseFarmInitModule
    + farm_base_impl::base_farm_validation::BaseFarmValidationModule
    + farm_base_impl::enter_farm::BaseEnterFarmModule
    + farm_base_impl::claim_rewards::BaseClaimRewardsModule
    + farm_base_impl::compound_rewards::BaseCompoundRewardsModule
    + farm_base_impl::exit_farm::BaseExitFarmModule
    + utils::UtilsModule
    + farm_token_roles::FarmTokenRolesModule
    + user_actions::stake_farm::StakeFarmModule
    + user_actions::claim_stake_farm_rewards::ClaimStakeFarmRewardsModule
    + user_actions::compound_stake_farm_rewards::CompoundStakeFarmRewardsModule
    + user_actions::unstake_farm::UnstakeFarmModule
    + user_actions::unbond_farm::UnbondFarmModule
    + user_actions::claim_only_boosted_staking_rewards::ClaimOnlyBoostedStakingRewardsModule
    + unbond_token::UnbondTokenModule
    + farm_boosted_yields::FarmBoostedYieldsModule
    + farm_boosted_yields::boosted_yields_factors::BoostedYieldsFactorsModule
    + week_timekeeping::WeekTimekeepingModule
    + weekly_rewards_splitting::WeeklyRewardsSplittingModule
    + weekly_rewards_splitting::events::WeeklyRewardsSplittingEventsModule
    + weekly_rewards_splitting::global_info::WeeklyRewardsGlobalInfo
    + weekly_rewards_splitting::locked_token_buckets::WeeklyRewardsLockedTokenBucketsModule
    + weekly_rewards_splitting::update_claim_progress_energy::UpdateClaimProgressEnergyModule
    + energy_query::EnergyQueryModule
    + tiered_rewards::read_config::ReadConfigModule
    + tiered_rewards::tokens_per_tier::TokenPerTierModule
{
    #[init]
    fn init(
        &self,
        farming_token_id: TokenIdentifier,
        division_safety_constant: BigUint,
        owner: ManagedAddress,
        config_sc_address: ManagedAddress,
        guild_master: ManagedAddress,
        first_week_start_epoch: Epoch,
        admins: MultiValueEncoded<ManagedAddress>,
    ) {
        // farming and reward token are the same
        self.base_farm_init(
            farming_token_id.clone(),
            farming_token_id,
            division_safety_constant,
            owner,
            admins,
        );

        let current_epoch = self.blockchain().get_block_epoch();
        require!(
            first_week_start_epoch >= current_epoch,
            "Invalid start epoch"
        );
        self.require_sc_address(&config_sc_address);

        self.first_week_start_epoch().set(first_week_start_epoch);
        self.config_sc_address().set(config_sc_address);
        self.guild_master().set(guild_master);
    }

    #[endpoint]
    fn upgrade(&self) {
        // Farm position migration code
        let farm_token_mapper = self.farm_token();
        self.try_set_farm_position_migration_nonce(farm_token_mapper);
    }

    #[payable("*")]
    #[endpoint(mergeFarmTokens)]
    fn merge_farm_tokens_endpoint(&self) -> EsdtTokenPayment {
        let caller = self.blockchain().get_caller();
        let boosted_rewards = self.claim_only_boosted_payment(&caller);
        self.add_boosted_rewards(&caller, &boosted_rewards);

        let payments = self.get_non_empty_payments();
        let token_mapper = self.farm_token();
        let output_attributes: StakingFarmTokenAttributes<Self::Api> =
            self.merge_from_payments_and_burn(payments, &token_mapper);
        let new_token_amount = output_attributes.get_total_supply();

        let merged_farm_token = token_mapper.nft_create(new_token_amount, &output_attributes);
        self.send_payment_non_zero(&caller, &merged_farm_token);

        merged_farm_token
    }

    #[view(calculateRewardsForGivenPosition)]
    fn calculate_rewards_for_given_position(
        &self,
        farm_token_amount: BigUint,
        attributes: StakingFarmTokenAttributes<Self::Api>,
    ) -> BigUint {
        self.require_queried();

        let mut storage_cache = StorageCache::new(self);
        FarmStakingWrapper::<Self>::generate_aggregated_rewards(self, &mut storage_cache);

        let rewards = FarmStakingWrapper::<Self>::calculate_rewards(
            self,
            &ManagedAddress::zero(),
            &farm_token_amount,
            &attributes,
            &storage_cache,
        );

        rewards.base
    }

    fn require_queried(&self) {
        let caller = self.blockchain().get_caller();
        let sc_address = self.blockchain().get_sc_address();
        require!(
            caller == sc_address,
            "May only call this function through VM query"
        );
    }
}
