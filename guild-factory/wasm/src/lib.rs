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
    guild_factory
    (
        init => init
        upgrade => upgrade
        deployConfigSc => deploy_config_sc
        callConfigFunction => call_config_function
        deployGuild => deploy_guild
        setTransferRoleGuild => set_transfer_role_guild
        setTransferRoleForFactory => set_transfer_role_for_factory
        isAdmin => is_admin
        addAdmin => add_admin
        removeAdmin => remove_admin
        getAdmins => admins
    )
}

multiversx_sc_wasm_adapter::async_callback_empty! {}
