use cosmrs::Any;
use cosmwasm_std::Coin;

use prost::Message;
use test_tube::account::SigningAccount;

use test_tube::runner::result::{RunnerExecuteResult, RunnerResult};
use test_tube::runner::Runner;
use test_tube::BaseApp;

const FEE_DENOM: &str = "uaura";
const CHAIN_ID: &str = "aura-testnet";
const DEFAULT_GAS_ADJUSTMENT: f64 = 1.3;

#[derive(Debug, PartialEq)]
pub struct AuraTestApp {
    inner: BaseApp,
}

impl Default for AuraTestApp {
    fn default() -> Self {
        AuraTestApp::new()
    }
}

impl AuraTestApp {
    pub fn new() -> Self {
        Self {
            inner: BaseApp::new(FEE_DENOM, CHAIN_ID, DEFAULT_GAS_ADJUSTMENT),
        }
    }

    /// Initialize account with initial balance of any coins.
    /// This function mints new coins and send to newly created account
    pub fn init_account(&self, coins: &[Coin]) -> RunnerResult<SigningAccount> {
        self.inner.init_account(coins)
    }
    /// Convinience function to create multiple accounts with the same
    /// Initial coins balance
    pub fn init_accounts(&self, coins: &[Coin], count: u64) -> RunnerResult<Vec<SigningAccount>> {
        self.inner.init_accounts(coins, count)
    }

    /// Simulate transaction execution and return gas info
    pub fn simulate_tx<I>(
        &self,
        msgs: I,
        signer: &SigningAccount,
    ) -> RunnerResult<cosmrs::proto::cosmos::base::abci::v1beta1::GasInfo>
    where
        I: IntoIterator<Item = cosmrs::Any>,
    {
        self.inner.simulate_tx(msgs, signer)
    }

    /// Set parameter set for a given subspace.
    pub fn set_param_set(&self, subspace: &str, pset: Any) -> RunnerResult<()> {
        self.inner.set_param_set(subspace, pset)
    }

    /// Get parameter set for a given subspace.
    pub fn get_param_set<P: Message + Default>(
        &self,
        subspace: &str,
        type_url: &str,
    ) -> RunnerResult<P> {
        self.inner.get_param_set(subspace, type_url)
    }
}

impl<'a> Runner<'a> for AuraTestApp {
    fn execute_multiple<M, R>(
        &self,
        msgs: &[(M, &str)],
        signer: &SigningAccount,
    ) -> RunnerExecuteResult<R>
    where
        M: ::prost::Message,
        R: ::prost::Message + Default,
    {
        self.inner.execute_multiple(msgs, signer)
    }

    fn query<Q, R>(&self, path: &str, q: &Q) -> RunnerResult<R>
    where
        Q: ::prost::Message,
        R: ::prost::Message + Default,
    {
        self.inner.query(path, q)
    }
}

#[cfg(test)]
mod tests {
    use std::option::Option::None;
    use cosmwasm_std::coins;
    use crate::module::Wasm;
    use crate::module::SmartAccount;
    use crate::runner::app::AuraTestApp;
    use test_tube::account::Account;
    use test_tube::runner::*;
    use test_tube::module::Module;

    use cosmrs::proto::cosmos::bank::v1beta1::{
        QueryAllBalancesRequest, QueryAllBalancesResponse
    };

    #[test] 
    fn test_query() {
        let app = AuraTestApp::default();

        let acc = app.init_account(&coins(100_000_000_000, "uaura")).unwrap();
        let addr = acc.address();

        let acc_balance = get_account_balances(&app, addr, "uaura");

        assert_eq!(acc_balance, 100_000_000_000u128);
    }

    fn get_account_balances(app: &AuraTestApp, address: String, denom: &str) -> u128 {
        let acc_balance = app.query::<QueryAllBalancesRequest,QueryAllBalancesResponse>(
            "/cosmos.bank.v1beta1.Query/AllBalances",
            &QueryAllBalancesRequest {
                address,
                pagination: None,
            },
        )
        .unwrap()
        .balances
        .into_iter()
        .find(|c| c.denom == denom)
        .unwrap()
        .amount
        .parse::<u128>()
        .unwrap();

        return acc_balance;
    }

    #[test]
    fn test_smartaccount() {
        let app = AuraTestApp::default();

        let acc = app.init_account(&coins(100_000_000_000, "uaura")).unwrap();
        
        let wasm = Wasm::new(&app);
        let smartaccount = SmartAccount::new(&app);

        let test_code = std::fs::read("./test_artifacts/base.wasm").unwrap(); // load contract wasm 

        let test_code_id = wasm
            .store_code(
                &test_code, 
                None, 
                &acc  
            )
            .unwrap()
            .data
            .code_id; 
        assert_eq!(test_code_id, 1);


        let pub_key = aura_std::shim::Any {
            type_url: String::from("/cosmos.crypto.secp256k1.PubKey"),
            value: acc.public_key().to_bytes()
        };

        let asAddr = smartaccount.query_generate_account(
            test_code_id, 
            "test salt".as_bytes().to_vec(), 
            "{}".as_bytes().to_vec(), 
            Some(pub_key)
        );
        println!("{}", asAddr.unwrap())
        
    }
}