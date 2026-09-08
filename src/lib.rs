#![no_std]
use soroban_sdk::{contract, contractimpl, Env, String};

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn hello(_env: Env, to: String) -> String {
        to
    }
}

#[cfg(test)]
mod test {
    use super::{Contract, ContractClient};
    use soroban_sdk::{Env, String};

    #[test]
    fn hello_returns_its_argument() {
        let env = Env::default();
        let contract_id = env.register(Contract, ());
        let client = ContractClient::new(&env, &contract_id);

        let to = String::from_str(&env, "verify-build-demo");
        assert_eq!(client.hello(&to), to);
    }
}
