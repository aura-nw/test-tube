#[cfg(test)]
mod unit_tests {
    use std::option::Option::None;
    use cosmos_sdk_proto::cosmos::base::v1beta1::Coin;
    use cosmos_sdk_proto::traits::MessageExt;
    use cosmos_sdk_proto::cosmos::bank::v1beta1::{MsgSend, MsgSendResponse};
    use cosmwasm_schema::cw_serde;
    use aura_test_tube::RunnerExecuteResult;
    use aura_proto::types::smartaccount::v1beta1::{Params, CodeID};
    use cosmwasm_std::coins;
    use aura_test_tube::{Wasm, AuraTestApp, SmartAccount};
    use cosmos_sdk_proto::cosmos::bank::v1beta1::{
        QueryAllBalancesRequest, QueryAllBalancesResponse
    };
    use aura_test_tube::init_local_smart_account;
    use aura_test_tube::{Runner, Module, Account, SigningAccount};

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

    fn send_coin(app: &AuraTestApp, from: &SigningAccount, to: String, amounts: Vec<Coin>) -> RunnerExecuteResult<MsgSendResponse> {
        app.execute(
            MsgSend {
                from_address: from.address(),
                to_address: to,
                amount: amounts,
            }, 
            "/cosmos.bank.v1beta1.MsgSend", 
            from
        )
    }

    #[cw_serde]
    struct EmptyInit {}

    #[cw_serde]
    struct Listen {
        listen: EmptyInit
    }

    #[test]
    fn test_smartaccount() {
        // default chain
        // id: aura-testnet
        // denom: uaura
        let app = AuraTestApp::default();

        let acc = app.init_base_account(&coins(100_000_000_000, "uaura")).unwrap();
        let acc_balance = get_account_balances(&app, acc.address(), "uaura");
        assert_eq!(acc_balance, 100_000_000_000u128);
        
        let wasm = Wasm::new(&app);
        let smartaccount = SmartAccount::new(&app);
        
        let test_code = std::fs::read("../../artifacts/spend_limit.wasm").unwrap(); // load contract wasm 

        // store wasm for smartaccount initialization
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
        
        // set whitelist for code id, don't need government
        let params = Params {
            whitelist_code_id: vec![CodeID{
                code_id: test_code_id,
                status: true
            }],
            disable_msgs_list: vec![],
            max_gas_execute: 2000000,
        };
        let param_set = aura_proto::shim::Any{
            type_url: String::from("/aura.smartaccount.v1beta1.Params"),
            value: params.to_bytes().unwrap()
        };
        let _ = app.set_param_set("smartaccount", param_set.into()).unwrap();
        // query smartaccount module param set
        let sa_params = smartaccount.query_params().unwrap();
        assert_eq!(sa_params.params, Some(params));

        
        // generate smartaccount address
        let pub_key = aura_proto::shim::Any {
            type_url: String::from("/cosmos.crypto.secp256k1.PubKey"),
            value: cosmos_sdk_proto::cosmos::crypto::secp256k1::PubKey { 
                key: acc.public_key().to_bytes()
            }.to_bytes().unwrap()
        };
        // or simple
        // let pub_key = acc.public_key().to_any().unwrap();
        let salt = "test account".as_bytes().to_vec();
        let init_msg = "{\"limit\":{\"denom\":\"uaura\",\"amount\":\"10000\"}}".as_bytes().to_vec();

        let sa_addr = smartaccount.query_generate_account(
            test_code_id, 
            salt.clone(), 
            init_msg.clone(), 
            pub_key.clone()
        ).unwrap();

        // send coin to smartaccount
        let banksend_res: RunnerExecuteResult<MsgSendResponse> = send_coin(
            &app,
        &acc,
        sa_addr.clone(),
        vec![Coin{denom: "uaura".to_string(),amount: "10000000".to_string(),}]
        );       
        assert!(banksend_res.is_ok());
        let acc_balance = get_account_balances(&app, sa_addr.clone(), "uaura");
        assert_eq!(acc_balance, 10000000u128);
        
        // local account which has not been initialized on-chain
        let sa_acc = init_local_smart_account(sa_addr.clone(), acc.private_key()).unwrap();
        // initializ smartaccount on-chain
        // execute with default gas setting
        // gas: 2000000
        // gas_price: 0.025
        // in order to modify gas setting, use &sa_acc.with_fee_setting(fee_setting) instead
        let _ = smartaccount.activate_account(
            test_code_id, 
            salt, 
            init_msg, 
            pub_key, 
            &sa_acc,
        ).unwrap();

        let acc2 = app.init_base_account(&coins(10, "uaura")).unwrap();

        let banksend_res: RunnerExecuteResult<MsgSendResponse> = send_coin(
            &app,
        &sa_acc,
        acc2.address(),
        vec![Coin{denom: "uaura".to_string(),amount: "5000".to_string(),}]
        );       
        println!("{:?}", banksend_res);
        assert!(banksend_res.is_ok());

        // send coin from smartaccount success
        let acc_balance = get_account_balances(&app, acc2.address(), "uaura");
        assert_eq!(acc_balance, 5010u128);

        let banksend_res: RunnerExecuteResult<MsgSendResponse> = send_coin(
            &app,
        &sa_acc,
        acc2.address(),
        vec![Coin{denom: "uaura".to_string(),amount: "5001".to_string(),}]
        );       
        assert!(banksend_res.is_err());

        // send coin from smartaccount fail, reach spend-limit 
        let acc_balance = get_account_balances(&app, acc2.address(), "uaura");
        assert_eq!(acc_balance, 5010u128);

        _ = app.skip_time(3600); // skip 1hours

        // re-send after 1hours
        let banksend_res: RunnerExecuteResult<MsgSendResponse> = send_coin(
            &app,
        &sa_acc,
        acc2.address(),
        vec![Coin{denom: "uaura".to_string(),amount: "5001".to_string(),}]
        );       
        assert!(banksend_res.is_ok());

        // send coin from smartaccount success, spend-limit outdated
        let acc_balance = get_account_balances(&app, acc2.address(), "uaura");
        assert_eq!(acc_balance, 10011u128);


        let listener_code = std::fs::read("../../artifacts/listener.wasm").unwrap(); // load contract wasm 

        let listener_code_id = wasm
            .store_code(
                &listener_code, 
                None, 
                &acc  
            )
            .unwrap()
            .data
            .code_id; 
        assert_eq!(listener_code_id, 2);

        let wasm_instantiate = wasm.instantiate(
            listener_code_id,
            &EmptyInit{},
            None,
            Some("listener"), 
            &[], 
            &acc, 
        ).unwrap();

        // use smartaccount to execute contract
        let _ = wasm.execute(&wasm_instantiate.data.address, &Listen{
            listen: EmptyInit{}
        }, &[], &sa_acc).unwrap();
    }
}