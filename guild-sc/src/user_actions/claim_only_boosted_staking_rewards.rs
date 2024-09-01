use crate::base_impl_wrapper::FarmStakingWrapper;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait ClaimOnlyBoostedStakingRewardsModule:
    config::ConfigModule
    + rewards::RewardsModule
    + farm_token::FarmTokenModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
    + week_timekeeping::WeekTimekeepingModule
    + pausable::PausableModule
    + permissions_module::PermissionsModule
    + weekly_rewards_splitting::WeeklyRewardsSplittingModule
    + weekly_rewards_splitting::events::WeeklyRewardsSplittingEventsModule
    + weekly_rewards_splitting::global_info::WeeklyRewardsGlobalInfo
    + weekly_rewards_splitting::locked_token_buckets::WeeklyRewardsLockedTokenBucketsModule
    + weekly_rewards_splitting::update_claim_progress_energy::UpdateClaimProgressEnergyModule
    + energy_query::EnergyQueryModule
    + token_send::TokenSendModule
    + events::EventsModule
    + utils::UtilsModule
    + farm_boosted_yields::FarmBoostedYieldsModule
    + farm_boosted_yields::boosted_yields_factors::BoostedYieldsFactorsModule
    + crate::custom_rewards::CustomRewardsModule
{
    #[endpoint(claimBoostedRewards)]
    fn claim_boosted_rewards(&self, opt_user: OptionalValue<ManagedAddress>) -> EsdtTokenPayment {
        let current_epoch = self.blockchain().get_block_epoch();
        let first_week_start_epoch = self.first_week_start_epoch().get();
        require!(
            first_week_start_epoch <= current_epoch,
            "Cannot claim rewards yet"
        );

        let caller = self.blockchain().get_caller();
        let user = match &opt_user {
            OptionalValue::Some(user) => user,
            OptionalValue::None => &caller,
        };
        let user_total_farm_position = self.get_user_total_farm_position(user);
        if user != &caller {
            require!(
                user_total_farm_position.allow_external_claim_boosted_rewards,
                "Cannot claim rewards for this address"
            );
        }

        let accumulated_boosted_rewards = self.accumulated_rewards_per_user(user).take();
        let boosted_rewards = self.claim_only_boosted_payment(user);
        let boosted_rewards_payment = EsdtTokenPayment::new(
            self.reward_token_id().get(),
            0,
            accumulated_boosted_rewards + boosted_rewards,
        );

        self.send_payment_non_zero(user, &boosted_rewards_payment);

        boosted_rewards_payment
    }

    fn migrate_old_farm_positions(&self, caller: &ManagedAddress) -> BigUint {
        let payments = self.call_value().all_esdt_transfers().clone_value();
        let farm_token_mapper = self.farm_token();
        let farm_token_id = farm_token_mapper.get_token_id();
        let mut migrated_amount = BigUint::zero();
        for farm_position in &payments {
            if farm_position.token_identifier == farm_token_id
                && self.is_old_farm_position(farm_position.token_nonce)
            {
                migrated_amount += farm_position.amount;
            }
        }

        if migrated_amount > 0 {
            let mut user_total_farm_position = self.get_user_total_farm_position(caller);
            user_total_farm_position.total_farm_position += &migrated_amount;
            self.user_total_farm_position(caller)
                .set(user_total_farm_position);
        }

        migrated_amount
    }

    fn decrease_old_farm_positions(&self, migrated_amount: BigUint, caller: &ManagedAddress) {
        if migrated_amount == BigUint::zero() {
            return;
        }
        self.user_total_farm_position(caller)
            .update(|user_total_farm_position| {
                user_total_farm_position.total_farm_position -= migrated_amount;
            });
    }

    // Cannot import the one from farm, as the Wrapper struct has different dependencies
    fn claim_only_boosted_payment(&self, caller: &ManagedAddress) -> BigUint {
        let reward = FarmStakingWrapper::<Self>::calculate_boosted_rewards(self, caller);
        if reward > 0 {
            self.reward_reserve().update(|reserve| *reserve -= &reward);
        }

        reward
    }
}
