use cosmrs::proto::tendermint::abci::ResponseFinalizeBlock;
use cosmrs::Any;
use cosmwasm_std::{Coin, Timestamp};
use prost::Message;
use test_tube::account::SigningAccount;
use test_tube::runner::result::{RunnerExecuteResult, RunnerResult};
use test_tube::runner::Runner;
use test_tube::BaseApp;

const FEE_DENOM: &str = "udt";
const DCHAIN_ADDRESS_PREFIX: &str = "dchain";
const CHAIN_ID: &str = "dchain-1";
const DEFAULT_GAS_ADJUSTMENT: f64 = 1.5;
const TEST_PLATFORM_ADMIN: &str = "dchain1zxn04jdhpt9krp6x8tzldv97nu2c6ed5yvrk00";

#[derive(Debug, PartialEq)]
pub struct DchainTestApp {
    inner: BaseApp,
}

impl Default for DchainTestApp {
    fn default() -> Self {
        DchainTestApp::new_with_platform_admin(TEST_PLATFORM_ADMIN)
    }
}

impl DchainTestApp {
    pub fn new() -> Self {
        Self {
            inner: BaseApp::new(
                FEE_DENOM,
                CHAIN_ID,
                DCHAIN_ADDRESS_PREFIX,
                DEFAULT_GAS_ADJUSTMENT,
            ),
        }
    }

    pub fn new_with_platform_admin(platform_admin: &str) -> Self {
        Self {
            inner: BaseApp::new_with_platform_admin(
                FEE_DENOM,
                CHAIN_ID,
                DCHAIN_ADDRESS_PREFIX,
                DEFAULT_GAS_ADJUSTMENT,
                platform_admin,
            ),
        }
    }

    /// Get the current block time as a timestamp
    pub fn get_block_timestamp(&self) -> Timestamp {
        self.inner.get_block_timestamp()
    }

    /// Get the current block time in nanoseconds
    pub fn get_block_time_nanos(&self) -> i64 {
        self.inner.get_block_time_nanos()
    }

    /// Get the current block time in seconds
    pub fn get_block_time_seconds(&self) -> i64 {
        self.inner.get_block_time_nanos() / 1_000_000_000i64
    }

    /// Get the current block height
    pub fn get_block_height(&self) -> i64 {
        self.inner.get_block_height()
    }

    /// Get the first validator address
    pub fn get_first_validator_address(&self) -> RunnerResult<String> {
        self.inner.get_first_validator_address()
    }

    /// Get the first validator signing account
    pub fn get_first_validator_signing_account(&self) -> RunnerResult<SigningAccount> {
        self.inner.get_first_validator_signing_account()
    }

    /// Increase the time of the blockchain by the given number of seconds.
    pub fn increase_time(&self, seconds: u64) {
        self.inner.increase_time(seconds)
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
    pub fn set_param_set(&self, subspace: &str, pset: impl Into<Any>) -> RunnerResult<()> {
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

    /// Directly trigger sudo entrypoint on a given contract.
    ///
    /// # Caution
    ///
    /// This function bypasses standard state changes and processes within the chain logic that might occur in normal situation,
    /// It is primarily intended for internal system logic where necessary state adjustments are handled.
    /// Use only with full understanding of the function's impact on system state and testing validity.
    /// Improper use may result in misleading test outcomes, including false positives or negatives.
    #[cfg(feature = "wasm-sudo")]
    pub fn wasm_sudo<M: serde::Serialize>(
        &self,
        contract_address: &str,
        sudo_msg: M,
    ) -> RunnerResult<Vec<u8>> {
        self.inner.wasm_sudo(contract_address, sudo_msg)
    }
    //pub fn execute_with_selected_authenticators<I>(
    //    &self,
    //    msgs: I,
    //    account: &SigningAccount,
    //    signer: &SigningAccount,
    //    selected_authenticators: &[u64],
    //) -> RunnerResult<ResponseFinalizeBlock>
    //where
    //    I: IntoIterator<Item = cosmrs::Any> + Clone,
    //{
    //    // create authenticator tx with zero fee
    //    let sim_tx_bytes = self.create_authenticator_tx(
    //        msgs.clone(),
    //        account,
    //        signer,
    //        self.inner.default_simulation_fee(),
    //        selected_authenticators,
    //    )?;
    //    let calculated_fee = self.inner.calculate_fee(&sim_tx_bytes, account)?;

    //    // create tx with calculated fee
    //    let tx_bytes = self.create_authenticator_tx(
    //        msgs,
    //        account,
    //        signer,
    //        calculated_fee,
    //        selected_authenticators,
    //    )?;

    //    self.execute_tx(&tx_bytes)
    //}

    //fn create_authenticator_tx<I>(
    //    &self,
    //    msgs: I,
    //    account: &SigningAccount, // account to execute on behalf of, this must be first msg first required signer
    //    signer: &SigningAccount, // used for creating signature, but does not have to be the same as account
    //    fee: Fee,
    //    selected_authenticators: &[u64],
    //) -> RunnerResult<Vec<u8>>
    //where
    //    I: IntoIterator<Item = cosmrs::Any>,
    //{
    //    let ext = TxExtension {
    //        selected_authenticators: selected_authenticators.to_vec(),
    //    }
    //    .to_any();
    //    let ext_ops = vec![cosmrs::Any {
    //        type_url: ext.type_url,
    //        value: ext.value,
    //    }];
    //    let tx_body = tx::Body {
    //        messages: msgs.into_iter().map(Into::into).collect(),
    //        non_critical_extension_options: ext_ops,
    //        ..Default::default()
    //    };

    //    let account_addr = account.address();
    //    let seq = self.inner.get_account_sequence(&account_addr);
    //    let account_number = self.inner.get_account_number(&account_addr);

    //    let signer_info = SignerInfo::single_direct(Some(signer.public_key()), seq);
    //    let auth_info = signer_info.auth_info(fee);

    //    let chain_id = self
    //        .inner
    //        .get_chain_id()
    //        .parse()
    //        .expect("parse const str of chain id should never fail");

    //    let sign_doc = tx::SignDoc::new(&tx_body, &auth_info, &chain_id, account_number)
    //        .map_err(EncodeError::from_proto_error_report)?;

    //    let tx_raw = sign_doc
    //        .sign(signer.signing_key())
    //        .map_err(EncodeError::from_proto_error_report)?;

    //    tx_raw
    //        .to_bytes()
    //        .map_err(EncodeError::from_proto_error_report)
    //        .map_err(RunnerError::EncodeError)
    //}
}

impl<'a> Runner<'a> for DchainTestApp {
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

    fn execute_tx(&self, tx_bytes: &[u8]) -> RunnerResult<ResponseFinalizeBlock> {
        self.inner.execute_tx(tx_bytes)
    }

    fn execute_multiple_raw<R>(
        &self,
        msgs: Vec<cosmrs::Any>,
        signer: &SigningAccount,
    ) -> RunnerExecuteResult<R>
    where
        R: prost::Message + Default,
    {
        self.inner.execute_multiple_raw(msgs, signer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dchain_sdk_proto::cosmos::bank::v1beta1::QueryAllBalancesRequest;
    use dchain_sdk_proto::dchain::depository::v1::{QueryGetParamsRequest, QueryGetParamsResponse};
    use dchain_sdk_proto::dchain::notary::v1::{
        GetNotarisationFeeRateRequest, GetNotarisationFeeRateResponse, MsgRegisterNotaryInfo,
        MsgRegisterNotaryInfoResponse,
    };
    use dchain_sdk_proto::traits::TypeUrl;
    use std::option::Option::None;

    use cosmwasm_std::{coins, Coin};

    use crate::bank::Bank;
    use crate::runner::app::DchainTestApp;
    use crate::wasm::Wasm;
    use test_tube::account::{Account, FeeSetting};
    use test_tube::module::Module;
    use test_tube::ExecuteResponse;
    use test_tube::{runner::*, RunnerError};

    #[test]
    fn test_init_accounts() {
        let app = DchainTestApp::default();
        let accounts = app
            .init_accounts(&coins(100_000_000_000, "udt"), 3)
            .unwrap();

        assert!(accounts.first().is_some());
        assert!(accounts.get(1).is_some());
        assert!(accounts.get(2).is_some());
        assert!(accounts.get(3).is_none());
    }

    #[test]
    fn test_get_and_set_block_timestamp() {
        let app = DchainTestApp::default();

        let block_time_nanos = app.get_block_time_nanos();
        let block_time_seconds = app.get_block_time_seconds();

        app.increase_time(10u64);

        assert_eq!(
            app.get_block_time_nanos(),
            block_time_nanos + 10_000_000_000
        );
        assert_eq!(app.get_block_time_seconds(), block_time_seconds + 10);
    }

    #[test]
    fn test_get_block_height() {
        let app = DchainTestApp::default();

        assert_eq!(app.get_block_height(), 1i64);

        app.increase_time(10u64);

        assert_eq!(app.get_block_height(), 2i64);
    }

    #[test]
    fn test_execute() {
        let app = DchainTestApp::default();

        let acc = app
            .init_account(&coins(100_000_000_000_000, "udt"))
            .unwrap();
        let addr = acc.address();

        let msg = MsgRegisterNotaryInfo {
            sender: addr.clone(),
            notary_info_admin: addr.clone(),
            caller_route_and_additional_req: None,
            asset_route_and_additional_req: None,
            asset_type_id: 1,
        };

        let res: ExecuteResponse<MsgRegisterNotaryInfoResponse> = app
            .execute(msg, MsgRegisterNotaryInfo::TYPE_URL, &acc)
            .unwrap();

        let id = &res.data.notary_info_id;

        assert!(id > &0u64);
    }

    #[test]
    fn test_query() {
        let app = DchainTestApp::default();

        let depository_params = app
            .query::<QueryGetParamsRequest, QueryGetParamsResponse>(
                "/d.depository.v1.Query/GetParams",
                &QueryGetParamsRequest {},
            )
            .unwrap()
            .params
            .unwrap();

        assert_eq!(depository_params.platform_admin, TEST_PLATFORM_ADMIN);

        let notarisation_fee_rate = app
            .query::<GetNotarisationFeeRateRequest, GetNotarisationFeeRateResponse>(
                "/d.notary.v1.Query/GetNotarisationFeeRate",
                &GetNotarisationFeeRateRequest {},
            )
            .unwrap()
            .rate;

        assert!(!notarisation_fee_rate.is_empty());
    }

    #[test]
    fn test_currency_conversion_rate_query() {
        use dchain_sdk_proto::dchain::notary::v1::{
            GetCurrencyConversionRateRequest, GetCurrencyConversionRateResponse,
        };

        // Try with platform admin to ensure full genesis initialization
        let app = DchainTestApp::new();

        // Query EUR conversion rate (should be 1.0 based on genesis)
        let eur_rate = app
            .query::<GetCurrencyConversionRateRequest, GetCurrencyConversionRateResponse>(
                "/d.notary.v1.Query/GetCurrencyConversionRate",
                &GetCurrencyConversionRateRequest {
                    currency: "EUR".to_string(),
                },
            )
            .unwrap()
            .rate;

        assert!(!eur_rate.is_empty());
        println!("EUR conversion rate: {}", eur_rate);

        // Query USD conversion rate (should be 0.85 based on genesis)
        let usd_rate = app
            .query::<GetCurrencyConversionRateRequest, GetCurrencyConversionRateResponse>(
                "/d.notary.v1.Query/GetCurrencyConversionRate",
                &GetCurrencyConversionRateRequest {
                    currency: "USD".to_string(),
                },
            )
            .unwrap()
            .rate;

        assert!(!usd_rate.is_empty());
        println!("USD conversion rate: {}", usd_rate);
    }

    #[test]
    fn test_wasm_execute_and_query() {
        use cw1_whitelist::msg::*;

        let app = DchainTestApp::default();
        let accs = app
            .init_accounts(&[Coin::new(1_000_000_000_000u128, "udt")], 2)
            .unwrap();
        let admin = &accs[0];
        let new_admin = &accs[1];

        let wasm = Wasm::new(&app);
        let wasm_byte_code = std::fs::read("./test_artifacts/cw1_whitelist.wasm").unwrap();

        let existing_codes = wasm.query_stored_codes().unwrap().code_infos;

        let code_id = wasm
            .store_code(&wasm_byte_code, None, admin)
            .unwrap()
            .data
            .code_id;
        assert_eq!(code_id, existing_codes.len() as u64 + 1);

        // initialize admins and check if the state is correct
        let init_admins = vec![admin.address()];
        let contract_addr = wasm
            .instantiate(
                code_id,
                &InstantiateMsg {
                    admins: init_admins.clone(),
                    mutable: true,
                },
                Some(&admin.address()),
                Some("cw1_whitelist"),
                &[],
                admin,
            )
            .unwrap()
            .data
            .address;
        let admin_list = wasm
            .query::<QueryMsg, AdminListResponse>(&contract_addr, &QueryMsg::AdminList {})
            .unwrap();
        assert_eq!(admin_list.admins, init_admins);
        assert!(admin_list.mutable);

        // update admin and check again
        let new_admins = vec![new_admin.address()];
        wasm.execute::<ExecuteMsg>(
            &contract_addr,
            &ExecuteMsg::UpdateAdmins {
                admins: new_admins.clone(),
            },
            &[],
            admin,
        )
        .unwrap();

        let admin_list = wasm
            .query::<QueryMsg, AdminListResponse>(&contract_addr, &QueryMsg::AdminList {})
            .unwrap();

        assert_eq!(admin_list.admins, new_admins);
        assert!(admin_list.mutable);
    }

    #[test]
    fn test_custom_fee() {
        let app = DchainTestApp::default();
        let initial_balance = 1_000_000_000_000;
        let alice = app.init_account(&coins(initial_balance, "udt")).unwrap();
        let bob = app.init_account(&coins(initial_balance, "udt")).unwrap();

        let amount = Coin::new(1_000_000u128, "udt");
        let gas_limit = 100_000_000;

        // use FeeSetting::Auto by default, so should not equal newly custom fee setting
        let wasm = Wasm::new(&app);
        let wasm_byte_code = std::fs::read("./test_artifacts/cw1_whitelist.wasm").unwrap();
        let res = wasm.store_code(&wasm_byte_code, None, &alice).unwrap();

        assert_ne!(res.gas_info.gas_wanted, gas_limit);

        //update fee setting
        let bob = bob.with_fee_setting(FeeSetting::Custom {
            amount: amount.clone(),
            gas_limit,
        });
        wasm.store_code(&wasm_byte_code, None, &bob).unwrap();

        let bob_balance = Bank::new(&app)
            .query_all_balances(&QueryAllBalancesRequest {
                address: bob.address(),
                pagination: None,
                resolve_denom: false,
            })
            .unwrap()
            .balances
            .into_iter()
            .find(|c| c.denom == "udt")
            .unwrap()
            .amount
            .parse::<u128>()
            .unwrap();

        assert_eq!(
            bob_balance,
            initial_balance - amount.amount.to_string().parse::<u128>().unwrap()
        );

        // run with low gas limit should fail
        let bob = bob.with_fee_setting(FeeSetting::Custom {
            amount: amount.clone(),
            gas_limit: 100_000,
        });
        let err = wasm.store_code(&wasm_byte_code, None, &bob).unwrap_err();
        assert_eq!(
            err,
            RunnerError::ExecuteError {
                msg: "out of gas in location: txSize; gasWanted: 100000, gasUsed: 1871761: out of gas".to_string()
            }
        );
    }

    #[cfg(feature = "wasm-sudo")]
    #[test]
    fn test_wasm_sudo() {
        let app = DchainTestApp::default();
        let wasm = Wasm::new(&app);

        let wasm_byte_code = std::fs::read("./test_artifacts/simple_sudo.wasm").unwrap();
        let alice = app.init_account(&coins(1_000_000_000_000, "udt")).unwrap();

        let code_id = wasm
            .store_code(&wasm_byte_code, None, &alice)
            .unwrap()
            .data
            .code_id;

        let contract_addr = wasm
            .instantiate(
                code_id,
                &simple_sudo::msg::InstantiateMsg {},
                None,
                Some("simple_sudo"),
                &[],
                &alice,
            )
            .unwrap()
            .data
            .address;

        let res = app
            .wasm_sudo(
                &contract_addr,
                simple_sudo::msg::SudoMsg::SetRandomData {
                    key: "x".to_string(),
                    value: "1".to_string(),
                },
            )
            .unwrap();

        assert_eq!(String::from_utf8(res).unwrap(), "x=1");

        let res: simple_sudo::msg::RandomDataResponse = wasm
            .query(
                &contract_addr,
                &simple_sudo::msg::QueryMsg::GetRandomData {
                    key: "x".to_string(),
                },
            )
            .unwrap();
        assert_eq!(res.value, "1");
    }
}
