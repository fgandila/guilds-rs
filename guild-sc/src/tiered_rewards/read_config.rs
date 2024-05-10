use common_structs::{Epoch, Percent};
use guild_sc_config::tiers::{
    GuildMasterRewardTier, RewardTier, UserRewardTier, TIER_NOT_FOUND_ERR_MSG,
};
use multiversx_sc::storage::StorageKey;

multiversx_sc::imports!();

static GUILD_MASTER_TIERS_STORAGE_KEY: &[u8] = b"guildMasterTiers";
static USER_TIERS_STORAGE_KEY: &[u8] = b"userTiers";
static MAX_TOKENS_STORAGE_KEY: &[u8] = b"maxStakedTokens";
static MIN_UNBOND_EPOCHS_USER_KEY: &[u8] = b"minUnbondEpochsUser";
static MIN_UNBOND_EPOCHS_GUILD_MASTER_KEY: &[u8] = b"minUnbondEpochsGuildMaster";
static MIN_STAKE_USER_KEY: &[u8] = b"minStakeUser";
static MIN_STAKE_GUILD_MASTER_KEY: &[u8] = b"minStakeGuildMaster";

#[multiversx_sc::module]
pub trait ReadConfigModule {
    fn find_any_user_tier_apr(
        &self,
        user: &ManagedAddress,
        base_farming_amount: &BigUint,
        percentage_staked: Percent,
    ) -> Percent {
        let guild_master = self.guild_master().get();
        if user != &guild_master {
            self.find_user_tier_apr(percentage_staked)
        } else {
            self.find_guild_master_tier_apr(base_farming_amount)
        }
    }

    // percentage_staked unused
    fn find_guild_master_tier_apr(&self, base_farming_amount: &BigUint) -> Percent {
        let mapper = self.get_guild_master_tiers_mapper();
        let tier = self.find_tier_common(base_farming_amount, Percent::default(), &mapper);

        tier.apr
    }

    // base_farming_amount unused
    fn find_user_tier_apr(&self, percentage_staked: Percent) -> Percent {
        let mapper = self.get_user_tiers_mapper();
        let tier = self.find_tier_common(&BigUint::default(), percentage_staked, &mapper);

        tier.apr
    }

    fn find_tier_common<T: TopEncode + TopDecode + RewardTier<Self::Api>>(
        &self,
        base_farming_amount: &BigUint,
        percentage_staked: Percent,
        mapper: &VecMapper<T, ManagedAddress>,
    ) -> T {
        for reward_tier in mapper.iter() {
            if reward_tier.is_in_range(base_farming_amount, percentage_staked) {
                return reward_tier;
            }
        }

        sc_panic!(TIER_NOT_FOUND_ERR_MSG);
    }

    fn get_guild_master_tiers_mapper(
        &self,
    ) -> VecMapper<GuildMasterRewardTier<Self::Api>, ManagedAddress> {
        let config_addr = self.config_sc_address().get();

        VecMapper::<_, _, ManagedAddress>::new_from_address(
            config_addr,
            StorageKey::new(GUILD_MASTER_TIERS_STORAGE_KEY),
        )
    }

    fn get_user_tiers_mapper(&self) -> VecMapper<UserRewardTier, ManagedAddress> {
        let config_addr = self.config_sc_address().get();

        VecMapper::<_, _, ManagedAddress>::new_from_address(
            config_addr,
            StorageKey::new(USER_TIERS_STORAGE_KEY),
        )
    }

    fn get_max_staked_tokens(&self) -> BigUint {
        let config_addr = self.config_sc_address().get();
        let mapper = SingleValueMapper::<_, _, ManagedAddress>::new_from_address(
            config_addr,
            StorageKey::new(MAX_TOKENS_STORAGE_KEY),
        );

        mapper.get()
    }

    fn get_min_unbond_epochs_user(&self) -> Epoch {
        let config_addr = self.config_sc_address().get();
        let mapper = SingleValueMapper::<_, _, ManagedAddress>::new_from_address(
            config_addr,
            StorageKey::new(MIN_UNBOND_EPOCHS_USER_KEY),
        );

        mapper.get()
    }

    fn get_min_unbond_epochs_guild_master(&self) -> Epoch {
        let config_addr = self.config_sc_address().get();
        let mapper = SingleValueMapper::<_, _, ManagedAddress>::new_from_address(
            config_addr,
            StorageKey::new(MIN_UNBOND_EPOCHS_GUILD_MASTER_KEY),
        );

        mapper.get()
    }

    fn get_min_stake_user(&self) -> BigUint {
        let config_addr = self.config_sc_address().get();
        let mapper = SingleValueMapper::<_, _, ManagedAddress>::new_from_address(
            config_addr,
            StorageKey::new(MIN_STAKE_USER_KEY),
        );

        mapper.get()
    }

    fn get_min_stake_guild_master(&self) -> BigUint {
        let config_addr = self.config_sc_address().get();
        let mapper = SingleValueMapper::<_, _, ManagedAddress>::new_from_address(
            config_addr,
            StorageKey::new(MIN_STAKE_GUILD_MASTER_KEY),
        );

        mapper.get()
    }

    fn get_min_stake_for_user(&self, user: &ManagedAddress) -> BigUint {
        let guild_master = self.guild_master().get();
        if user != &guild_master {
            self.get_min_stake_user()
        } else {
            self.get_min_stake_guild_master()
        }
    }

    #[storage_mapper("configScAddress")]
    fn config_sc_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("guildMaster")]
    fn guild_master(&self) -> SingleValueMapper<ManagedAddress>;
}
