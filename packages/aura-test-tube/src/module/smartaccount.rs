use aura_std::types::smartaccount::v1beta1::{
    MsgActivateAccountRequest, MsgActivateAccountResponse, 
    QueryGenerateAccountRequest, QueryGenerateAccountResponse,
    QueryParamsRequest, QueryParamsResponse,
    MsgRecoverRequest, MsgRecoverResponse
};
use prost_types::Any;

use cosmwasm_std::Coin;
use serde::{de::DeserializeOwned, Serialize};

use test_tube::runner::error::{DecodeError, EncodeError, RunnerError};
use test_tube::runner::result::{RunnerExecuteResult, RunnerResult};
use test_tube::{
    account::{Account, SigningAccount},
    runner::Runner,
};

pub struct SmartAccount<'a, R: Runner<'a>> {
    runner: &'a R,
}

impl<'a, R: Runner<'a>> super::Module<'a, R> for SmartAccount<'a, R> {
    fn new(runner: &'a R) -> Self {
        Wasm { runner }
    }
}

impl<'a, R> SmartAccount<'a, R>
where
    R: Runner<'a>,
{
    pub fn recover(
        &self,
        address: String,
        public_key: Any,
        credentials: Vec<u8>,
        signer: &SigningAccount,
    ) -> RunnerExecuteResult<MsgRecoverResponse> {
        self.runner.execute(
            MsgRecoverRequest {
                creator: signer.address(),
                address,
                public_key,
                credentials
            },
            "/aura.smartaccount.v1beta1.MsgRecover",
            signer,
        )
    }

    pub fn activate_account<M>(
        &self,
        code_id: u64,
        salt: Vec<u8>,
        init_msg: &M,
        pub_key: Option<aura_std::shim::Any>,
        signer: &SigningAccount,
    ) -> RunnerExecuteResult<MsgActivateAccountResponse>
    where
        M: ?Sized + Serialize,
    {
        self.runner.execute(
            MsgActivateAccountRequest {
                account_address: signer.address(),
                code_id,
                salt,
                init_msg,
                pub_key,
            },
            "/aura.smartaccount.v1beta1.MsgActivateAccount",
            signer,
        )
    }

    pub fn query_generate_account<M, Res>(
        &self, 
        code_id: u64,
        salt: Vec<u8>, 
        init_msg: &M,
        pub_key: Option<aura_std::shim::Any>
    ) -> RunnerResult<Res>
    where
        M: ?Sized + Serialize,
        Res: ?Sized + DeserializeOwned,
    {
        let res = self
            .runner
            .query::<QueryGenerateAccountRequest, QueryGenerateAccountResponse>(
                "/aura.smartaccount.v1beta1.Query/GenerateAccount",
                &QueryGenerateAccountRequest {
                    code_id,
                    salt,
                    init_msg,
                    pub_key,
                },
            )?;

        serde_json::from_slice(&res.data)
            .map_err(DecodeError::JsonDecodeError)
            .map_err(RunnerError::DecodeError)
    }

    pub fn query_params<M, Res>(&self, ) -> RunnerResult<Res>
    where
        M: ?Sized + Serialize,
        Res: ?Sized + DeserializeOwned,
    {
        let res = self
            .runner
            .query::<QueryParamsRequest, QueryParamsResponse>(
                "/aura.smartaccount.v1beta1.Query/Params",
                &QueryParamsRequest {},
            )?;

        serde_json::from_slice(&res.data)
            .map_err(DecodeError::JsonDecodeError)
            .map_err(RunnerError::DecodeError)
    }
}
