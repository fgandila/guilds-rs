multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use common_structs::Nonce;
use pausable::State;

pub const DEFAULT_NFT_DEPOSIT_MAX_LEN: usize = 10;
pub const DEFAULT_FARM_POSITION_MIGRATION_NONCE: u64 = 1;

#[derive(
    ManagedVecItem,
    TopEncode,
    TopDecode,
    NestedEncode,
    NestedDecode,
    TypeAbi,
    Clone,
    PartialEq,
    Debug,
)]
pub struct UserTotalFarmPosition<M: ManagedTypeApi> {
    pub total_farm_position: BigUint<M>,
    pub allow_external_claim_boosted_rewards: bool,
}

impl<M: ManagedTypeApi> Default for UserTotalFarmPosition<M> {
    fn default() -> Self {
        Self {
            total_farm_position: BigUint::zero(),
            allow_external_claim_boosted_rewards: false,
        }
    }
}

#[multiversx_sc::module]
pub trait ConfigModule: pausable::PausableModule + permissions_module::PermissionsModule {
    #[endpoint(allowExternalClaimBoostedRewards)]
    fn allow_external_claim_boosted_rewards(&self, allow_external_claim: bool) {
        let caller = self.blockchain().get_caller();
        let mut user_total_farm_position = self.get_user_total_farm_position(&caller);
        user_total_farm_position.allow_external_claim_boosted_rewards = allow_external_claim;
        self.user_total_farm_position(&caller)
            .set(user_total_farm_position);
    }

    #[view(getAllowExternalClaimRewards)]
    fn get_allow_external_claim_rewards(&self, user: ManagedAddress) -> bool {
        let user_total_farm_position = self.get_user_total_farm_position(&user);
        user_total_farm_position.allow_external_claim_boosted_rewards
    }

    #[inline]
    fn is_active(&self) -> bool {
        let state = self.state().get();
        state == State::Active
    }

    fn get_user_total_farm_position(
        &self,
        user: &ManagedAddress,
    ) -> UserTotalFarmPosition<Self::Api> {
        let user_total_farm_position_mapper = self.user_total_farm_position(user);
        if user_total_farm_position_mapper.is_empty() {
            UserTotalFarmPosition::default()
        } else {
            user_total_farm_position_mapper.get()
        }
    }

    fn is_old_farm_position(&self, token_nonce: Nonce) -> bool {
        let farm_position_migration_nonce = self.farm_position_migration_nonce().get();
        token_nonce > 0 && token_nonce < farm_position_migration_nonce
    }

    fn try_set_farm_position_migration_nonce(
        &self,
        farm_token_mapper: NonFungibleTokenMapper<Self::Api>,
    ) {
        if !self.farm_position_migration_nonce().is_empty() {
            return;
        }

        let migration_farm_token_nonce = if farm_token_mapper.get_token_state().is_set() {
            let token_identifier = farm_token_mapper.get_token_id_ref();
            let current_nonce = self
                .blockchain()
                .get_current_esdt_nft_nonce(&self.blockchain().get_sc_address(), token_identifier);
            current_nonce + DEFAULT_FARM_POSITION_MIGRATION_NONCE
        } else {
            DEFAULT_FARM_POSITION_MIGRATION_NONCE
        };

        self.farm_position_migration_nonce()
            .set(migration_farm_token_nonce);
    }

    #[view(getFarmingTokenId)]
    #[storage_mapper("farming_token_id")]
    fn farming_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getRewardTokenId)]
    #[storage_mapper("reward_token_id")]
    fn reward_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getPerBlockRewardAmount)]
    #[storage_mapper("per_block_reward_amount")]
    fn per_block_reward_amount(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("produce_rewards_enabled")]
    fn produce_rewards_enabled(&self) -> SingleValueMapper<bool>;

    #[view(getLastRewardBlockNonce)]
    #[storage_mapper("last_reward_block_nonce")]
    fn last_reward_block_nonce(&self) -> SingleValueMapper<Nonce>;

    #[view(getDivisionSafetyConstant)]
    #[storage_mapper("division_safety_constant")]
    fn division_safety_constant(&self) -> SingleValueMapper<BigUint>;

    #[view(getUserTotalFarmPosition)]
    #[storage_mapper("userTotalFarmPosition")]
    fn user_total_farm_position(
        &self,
        user: &ManagedAddress,
    ) -> SingleValueMapper<UserTotalFarmPosition<Self::Api>>;

    #[view(getFarmPositionMigrationNonce)]
    #[storage_mapper("farm_position_migration_nonce")]
    fn farm_position_migration_nonce(&self) -> SingleValueMapper<Nonce>;
}
