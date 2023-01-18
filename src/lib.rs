// @rust tip: #![no_std] to ensure that the Rust standard library is not included in
// the build since it's too big for blockchains
#![no_std]
use soroban_auth::{Identifier, Signature};
// importing the types and macros from soroban_sdk
use soroban_sdk::{contractimpl, contracttype, symbol, vec, map, Env, Symbol, Vec, Map, BytesN, Address};

mod token {
    soroban_sdk::contractimport!(file = "./token/soroban_token_spec.wasm");
}

// @rust tip: importing test.rs
mod test;

pub struct ProfitDistributionContract;

/*
// Requirements:
// 1. MeetupDate: The date of the meetup
// 2. Balance: The total amount of deposits
// 2. Depositers: People who deposited
// 3. Attendees: Attendees who showed up and eligible to claim
// 4. Started: The date the admin started collecting
// 5. Admin: The person who can trigger the disbursement of the deposit
// 6. Token: 
// 7. User:
// 8. Recipient: Meetup Pub Key
// 9. Amount: The cost of deposit
*/
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    MeetupDate,
    Balance,
    Depositers,
    Attendees,
    Started,
    Admin,
    Token,
    // User(Identifier),
    Recipient,
    Amount,
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
// 1. Call initialize(recipient, meetup_date_unix_epoch, target_amount, token) 
// 2. Depositor makes a deposit to this contract's address to REGISTER and the contract stores depositor's public key
// 3. Once the meetup date is reached, the contract collects the attendees' public key, divides its total amount of balance by the # of attendees and send that amount to the attendees who match its depositors' public key
*/


// MeetupDate,
// Balance,
// Depositers,
// Attendees,
// Started,
// Admin,
// Token,
// User(Identifier),
// Recipient,
// Amount,

// pub struct DepositBalance {
//     pub token: BytesN<32>,
//     pub balance: i128,
//     pub depositers: Vec<Identifier>,
//     pub time_bound: TimeBound,
// }

// @rust tip: #[contractimpl] where contract lives
#[contractimpl]
impl ProfitDistributionContract {
    // @rust tip: any function that'll be called externally has 'pub'
    pub fn initialize(
        env: Env,
        recipient: Identifier,
        balance: i128,
        meetup_date: u64,
        amount: i128,
        token: BytesN<32>
    ){
        assert!(is_initialized(&env), "Contract already initialized");

        env.storage().set(DataKey::Recipient, recipient);
        env.storage().set(DataKey::Started, get_ledger_timestamp(&env));
        env.storage().set(DataKey::MeetupDate, meetup_date);
        env.storage().set(DataKey::Balance, balance);
        env.storage().set(DataKey::Amount, amount);
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

        // Transfer token to this contract address.
        transfer_from_account_to_contract(&env, &token, &env.invoker().into(), &amount);
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

    pub fn distribute(

    ){}
}

fn is_initialized(env: &Env) -> bool {
    env.storage().has(DataKey::Recipient)
}

fn get_ledger_timestamp(e: &Env) -> u64 {
    e.ledger().timestamp()
}

fn get_contract_id(e: &Env) -> Identifier {
    Identifier::Contract(e.get_current_contract())
}

fn transfer_from_account_to_contract(
    e: &Env,
    token_id: &BytesN<32>,
    from: &Identifier,
    amount: &i128,
) {
    let client = token::Client::new(e, token_id);
    client.xfer_from(&Signature::Invoker, &0, from, &get_contract_id(e), amount);
}

fn transfer_from_contract_to_account(
    e: &Env,
    token_id: &BytesN<32>,
    to: &Identifier,
    amount: &i128,
) {
    let client = token::Client::new(e, token_id);
    client.xfer(&Signature::Invoker, &0, to, amount);
}