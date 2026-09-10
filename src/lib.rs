#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, BytesN, Env, String};

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn __constructor(env: Env, admin: Address) {
        env.storage().instance().set(&symbol_short!("admin"), &admin);
    }

    pub fn hello(_env: Env, to: String) -> String {
        to
    }

    pub fn goodbye(_env: Env, to: String) -> String {
        to
    }

    /// Returns the admin address. `stellar registry upgrade` looks for this
    /// exact function to require the admin's auth at the top level of the
    /// upgrade transaction before invoking `upgrade` below.
    pub fn admin(env: Env) -> Address {
        env.storage().instance().get(&symbol_short!("admin")).unwrap()
    }

    /// Called by `stellar registry upgrade` to swap this instance to a new
    /// published wasm version.
    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) {
        let admin: Address = env.storage().instance().get(&symbol_short!("admin")).unwrap();
        admin.require_auth();
        env.deployer().update_current_contract_wasm(new_wasm_hash);
    }
}

#[cfg(test)]
mod test {
    use super::{Contract, ContractClient};
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    #[test]
    fn hello_returns_its_argument() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let contract_id = env.register(Contract, (admin,));
        let client = ContractClient::new(&env, &contract_id);

        let to = String::from_str(&env, "verify-build-demo");
        assert_eq!(client.hello(&to), to);
    }
}
