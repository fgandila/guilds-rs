// Code generated by the multiversx-sc build system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Endpoints:                           10
// Async Callback (empty):               1
// Total number of exported functions:  12

#![no_std]
#![allow(internal_features)]
#![feature(lang_items)]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    guild_sc_config
    (
        init => init
        upgrade => upgrade
        setMinUnbondEpochs => set_min_unbond_epochs_endpoint
        addGuildMasterTiers => add_guild_master_tiers
        setGuildMasterTierApr => set_guild_master_tier_apr
        addUserTiers => add_user_tiers
        setUserTierApr => set_user_tier_apr
        getGuildMasterTiers => guild_master_tiers
        getUserTiers => user_tiers
        getMaxStakedTokens => max_staked_tokens
        getMinUnbondEpochs => min_unbond_epochs
    )
}

multiversx_sc_wasm_adapter::async_callback_empty! {}
