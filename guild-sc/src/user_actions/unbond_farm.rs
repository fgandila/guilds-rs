multiversx_sc::imports!();

use crate::tokens::token_attributes::FixedSupplyToken;
use crate::{
    contexts::storage_cache::StorageCache, farm_base_impl::base_traits_impl::FarmStakingWrapper,
    tokens::token_attributes::LocalFarmToken,
};

use crate::{
    tiered_rewards::total_tokens::TotalTokens, tokens::token_attributes::UnbondSftAttributes,
};

#[multiversx_sc::module]
pub trait UnbondFarmModule:
    crate::custom_rewards::CustomRewardsModule
    + crate::tokens::unbond_token::UnbondTokenModule
    + crate::rewards::RewardsModule
    + crate::config::ConfigModule
    + crate::events::EventsModule
    + token_send::TokenSendModule
    + crate::tokens::farm_token::FarmTokenModule
    + crate::tokens::request_id::RequestIdModule
    + pausable::PausableModule
    + permissions_module::PermissionsModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
    + crate::farm_base_impl::base_farm_validation::BaseFarmValidationModule
    + crate::farm_base_impl::enter_farm::BaseEnterFarmModule
    + utils::UtilsModule
    + crate::tiered_rewards::read_config::ReadConfigModule
    + crate::tiered_rewards::total_tokens::TokenPerTierModule
    + crate::tiered_rewards::call_config::CallConfigModule
    + super::custom_events::CustomEventsModule
    + super::close_guild::CloseGuildModule
{
    #[payable("*")]
    #[endpoint(unbondFarm)]
    fn unbond_farm(&self) -> EsdtTokenPayment {
        let storage_cache = StorageCache::new(self);
        self.validate_contract_state(storage_cache.contract_state, &storage_cache.farm_token_id);

        let unbond_token_mapper = self.unbond_token();
        let payments = self.get_non_empty_payments();

        let mut total_farming_tokens = BigUint::zero();
        for payment in &payments {
            unbond_token_mapper.require_same_token(&payment.token_identifier);

            let attributes: UnbondSftAttributes<Self::Api> =
                unbond_token_mapper.get_token_attributes(payment.token_nonce);
            require!(
                attributes.supply == payment.amount,
                "Must unbond with all tokens"
            );

            let current_epoch = self.blockchain().get_block_epoch();
            require!(
                current_epoch >= attributes.unlock_epoch,
                "Unbond period not over"
            );

            unbond_token_mapper.nft_burn(payment.token_nonce, &payment.amount);

            total_farming_tokens += payment.amount;
        }

        let caller = self.blockchain().get_caller();
        let farming_tokens = EsdtTokenPayment::new(
            storage_cache.farming_token_id.clone(),
            0,
            total_farming_tokens,
        );
        self.send_payment_non_zero(&caller, &farming_tokens);

        farming_tokens
    }

    #[payable("*")]
    #[endpoint(cancelUnbond)]
    fn cancel_unbond(&self) -> EsdtTokenPayment {
        self.require_not_closing();
        self.require_not_globally_paused();

        let unbond_token_mapper = self.unbond_token();
        let payment = self.call_value().single_esdt();
        unbond_token_mapper.require_same_token(&payment.token_identifier);

        let unbond_attributes: UnbondSftAttributes<Self::Api> =
            unbond_token_mapper.get_token_attributes(payment.token_nonce);
        require!(
            payment.amount == unbond_attributes.supply,
            "Must cancel unbond with all tokens"
        );
        require!(
            unbond_attributes.opt_original_attributes.is_some(),
            "May not cancel unbond for this token"
        );

        unbond_token_mapper.nft_burn(payment.token_nonce, &payment.amount);

        let original_attributes = unsafe {
            unbond_attributes
                .opt_original_attributes
                .clone()
                .unwrap_unchecked()
        };

        let caller = self.blockchain().get_caller();
        let total_farming_tokens = original_attributes.get_initial_farming_tokens();
        let farming_token_id = self.farming_token_id().get();
        let farming_token_payment =
            EsdtTokenPayment::new(farming_token_id, 0, total_farming_tokens);
        let enter_result = self.enter_farm_base_no_token_create::<FarmStakingWrapper<Self>>(
            caller.clone(),
            ManagedVec::from_single_item(farming_token_payment),
        );

        let mut new_attributes = enter_result.new_farm_token.attributes;
        new_attributes.current_farm_amount += &original_attributes.compounded_reward;
        new_attributes.compounded_reward = original_attributes.compounded_reward;

        let initial_farming_tokens = new_attributes.get_initial_farming_tokens();
        self.add_total_base_staked_tokens(&initial_farming_tokens);
        self.add_tokens(
            &caller,
            &TotalTokens::new(
                initial_farming_tokens,
                new_attributes.compounded_reward.clone(),
            ),
        );
        self.call_increase_total_staked_tokens(FixedSupplyToken::<Self>::get_total_supply(
            &new_attributes,
        ));

        self.total_compounded_tokens()
            .update(|total| *total += &new_attributes.compounded_reward);

        let total_farm_tokens = FixedSupplyToken::<Self>::get_total_supply(&new_attributes);
        let new_farm_token =
            self.farm_token()
                .nft_create_and_send(&caller, total_farm_tokens, &new_attributes);

        let base_farm_amount = new_attributes.get_initial_farming_tokens();
        let compounded_rewards = new_attributes.get_compounded_rewards();
        self.tokens_for_nonce(new_farm_token.token_nonce)
            .set(TotalTokens::new(base_farm_amount, compounded_rewards));

        self.emit_cancel_unbond_event(
            &caller,
            unbond_attributes,
            new_farm_token.clone(),
            new_attributes,
        );

        new_farm_token
    }
}
