// @rust tip: #![no_std] to ensure that the Rust standard library is not included in
// the build since it's too big for blockchains
#![no_std]
use soroban_auth::{Identifier, Signature};
// importing the types and macros from soroban_sdk
use soroban_sdk::{contractimpl, contracttype, Env, Vec, BytesN};

mod token {
    soroban_sdk::contractimport!(file = "./token/soroban_token_spec.wasm");
}


pub struct ProfitDistributionContract;

/*
// Requirements:
// 1. MeetupDate: The date of the meetup
// 2. Attendees: Attendees who showed up and eligible to claim
// 3. Started: The date the admin started collecting
// 4. Admin: The person who can trigger the disbursement of the deposit
// 5. Token: 
// 6. User:
// 7. DepositFee: The cost of deposit
*/
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    MeetupDate,
    Balance,
    Attendees,
    Started,
    Admin,
    Token,
    User(Identifier),
    DepositFee,
}

#[derive(Clone)]
#[contracttype]
pub enum TimeBoundKind {
    Before,
    After,
}

#[derive(Clone)]
#[contracttype]
pub struct TimeBound {
    pub kind: TimeBoundKind,
    pub timestamp: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct DepositBalance {
    pub token: BytesN<32>,
    pub amount: i128,
    pub depositers: Vec<Identifier>,
    pub time_bound: TimeBound,
}

/*
// State
// - Running = 0
// - Success = 1
// - Expired = 2
*/

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum State {
    Running = 0,
    Success = 1,
    Expired= 2,
}

/*
// Contract Usage Pattern (pseudocode):
// 1. Call initialize(recipient, meetup_date_unix_epoch, amount, token) 
// 2. Depositor makes a deposit to this contract's address to REGISTER and the contract stores depositor's public key
// 3. Once the meetup date is reached, the contract (by admin) collects the attendees' public key, divides its total amount of balance by the # of attendees and send that amount to the attendees who match its depositors' public key
*/

// @rust tip: #[contractimpl] where contract lives
#[contractimpl]
impl ProfitDistributionContract {
    // @rust tip: any function that'll be called externally use 'pub'
    pub fn initialize(
        env: Env,
        admin: Identifier,
        meetup_date: u64,
        deposit_fee: i128,
        token: BytesN<32>
    ){
        assert!(is_initialized(&env), "Contract already initialized");

        env.storage().set(DataKey::Admin, admin);
        env.storage().set(DataKey::Started, get_ledger_timestamp(&env));
        env.storage().set(DataKey::MeetupDate, meetup_date);
        env.storage().set(DataKey::DepositFee, deposit_fee);
        env.storage().set(DataKey::Token, token);
    }

    pub fn deposit(
        env: Env,
        token: BytesN<32>,
        amount: i128,
        depositers: Vec<Identifier>,
        time_bound: TimeBound
    ){
        if amount < 0 {
            panic!("negative amount is not allowed")
        }

        /*  
        // Transfer token to this contract address
        // @soroban tip: The env.invoker() always returns the invoker of the currently executing contract. Returning either: 
        // - Account with an AccountId if the contract was invoked directly by an account
        // - Contract with a BytesN<32> contract ID if the contract was invoked by another contract
        // https://soroban.stellar.org/docs/examples/auth#invoker
        */
        deposit_to_contract(&env, &env.invoker().into(), &amount);
        // Store all the necessary info to allow one of the claimants to claim it.
        env.storage().set(
            DataKey::Balance,
            DepositBalance {
                token,
                amount,
                time_bound,
                depositers,
            },
        );
    }

    pub fn distribute(env:Env){
        let balance:DepositBalance = env.storage().get_unchecked(DataKey::Balance).unwrap();

        let attendee_id = env.invoker().into();
        let depositers= &balance.depositers;

        if !depositers.contains(&attendee_id) {
            panic!("this attendee didn't make a deposit to register for the meetup. They're not eligible to receive any deposit back");
        }

        // Transfer the stored amount of token to claimant after passing
        // all the checks.
        distribute_from_contract_to_account(
            &env,
            &attendee_id,
            &balance.amount,
        );
        // Remove the balance entry to prevent any further claims.
        env.storage().remove(DataKey::Balance);
    }
}

fn is_initialized(env: &Env) -> bool {
    env.storage().has(DataKey::Admin)
}

fn get_ledger_timestamp(env: &Env) -> u64 {
    env.ledger().timestamp()
}

fn get_contract_id(env: &Env) -> Identifier {
    Identifier::Contract(env.get_current_contract())
}

fn get_token(env: &Env) -> BytesN<32> {
    env.storage()
        .get(DataKey::Token)
        .expect("not initialized")
        .unwrap()
}

fn get_balance(env: &Env, contract_id: &BytesN<32>) -> i128 {
    let client = token::Client::new(env, contract_id);
    client.balance(&get_contract_id(env))
}

fn deposit_to_contract(
    env: &Env,
    user: &Identifier,
    amount: &i128,
) {
    let client = token::Client::new(env, &get_token(env));
    let nonce: i128 = 0;
    
    /* 
    // @soroban tips: client.xfer_from()
    // xfer
    // - an unprivileged mutator, which changes the state of the contract but do not require special privileges
    // - a "sender" can use xfer to send money to a "admin" or contract id. For xfer, the sender must provide authorization
    // invoker auth (&Signature::Invoker) is enough to use the built-in token with classic accounts
    // more info on: https://soroban.stellar.org/docs/built-in-contracts/stellar-asset-contract#sac-operations &
    // https://soroban.stellar.org/docs/common-interfaces/token
    */ 
    client.xfer_from(&Signature::Invoker,&nonce, user, &get_contract_id(env), amount);
}

fn distribute_from_contract_to_account(
    env: &Env,
    user: &Identifier,
    amount: &i128,
) {

    let client = token::Client::new(env, &get_token(env));
    let nonce: i128 = 0;

    client.xfer(&Signature::Invoker, &nonce, user, amount);
}

// @rust tip: importing test.rs
mod test;