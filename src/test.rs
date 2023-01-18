#![cfg(test)]

use super::{Contract, ContractClient};
use soroban_sdk::{symbol, vec, Env};

#[test]
fn test() {
    // In any test the first thing that is always required is an Env,
    // which is the Soroban environment that the contract will run inside of
    let env = Env::default();

    // the first arg can be either 'contract ID' or 'None'
    let contract_id = env.register_contract(None, Contract);
    let client = ContractClient::new(&env, &contract_id);

    let words = client.hello(&symbol!("Dev"));
    assert_eq!(
        words,
        vec![&env, symbol!("Hello"), symbol!("Dev"),]
    );
}