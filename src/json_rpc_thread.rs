use crate::formatter::format_data_with_ledger;

use super::formatter;
use super::scrypto_helpers;

use super::CONFIG;

use ::jsonrpc_core::serde_json::json;
use jsonrpc_core::serde::Deserialize;
use jsonrpc_core::serde_json::Number;
use jsonrpc_core::*;
use jsonrpc_http_server::*;

use parking_lot::RwLockWriteGuard;

use radix_engine::ledger::*;
use radix_engine::model::Actor::SuperUser;
use radix_engine::transaction::*;
use scrypto::prelude::*;

// Structs to use as Params for our functions
#[derive(Deserialize)]
struct CallFunction {
    address: String,
    name: String,
    function: String,
    args: Vec<String>,
    account_address: String,
    key: String,
}

#[derive(Deserialize)]
struct CallMethod {
    address: String,
    method: String,
    args: Vec<String>,
    account_address: String,
    key: String,
}

#[derive(Deserialize)]
struct GetBalance {
    address: String,
}

pub fn rpc_thread() {
    // Create new IoHandler
    let mut io = IoHandler::default();

    // Add all methods that should be callable through the JSON-RPC server
    io.add_method("new_account", |_params| async move { new_account().await });

    io.add_method("call_function", |params: Params| async move {
        let parsed = params.parse().ok();
        match parsed {
            Some(v) => call_function(v).await,
            None => return parse_err(),
        }
    });

    io.add_method("call_method", |params: Params| async move {
        let parsed = params.parse().ok();
        match parsed {
            Some(v) => call_method(v).await,
            None => return parse_err(),
        }
    });

    io.add_method("get_balance", |params: Params| async move {
        let parsed = params.parse().ok();
        match parsed {
            Some(v) => get_balance(v).await,
            None => return parse_err(),
        }
    });

    // Start the server
    let server = ServerBuilder::new(io)
        .threads(4)
        .cors(DomainsValidation::AllowOnly(vec![
            AccessControlAllowOrigin::Any,
        ]))
        .start_http(&"127.0.0.1:3030".parse().unwrap())
        .expect("Unable to start RPC server");
    server.wait();
}

async fn new_account() -> jsonrpc_core::Result<jsonrpc_core::Value> {
    //Instantiate with dummy values to move into closure
    let mut key: Address =
        Address::from_str("02b8dd9f4232ce3c00dcb3496956fb57096d5d50763b989ca56f3b").unwrap();
    let mut account: Address =
        Address::from_str("02b9f7c0c44a6e2162403cea3fa44500dff50eb18fd4ff5a9dd079").unwrap();

    let write_lock_conf = CONFIG.write();
    let _ = RwLockWriteGuard::map(write_lock_conf, |config| {
        let (epoch, nonce, ledger) = config.load();
        //Do transaction
        let mut executor = TransactionExecutor::new(ledger, epoch, nonce);
        key = executor.new_public_key();
        account = executor.new_account(key);

        //Store the nonce and return the ledger and config
        let nonce = executor.nonce();
        config.store_nonce(nonce);
        config
    });
    Ok(json!({"key": key.to_string(), "account": account.to_string()}))
}

async fn call_function(params: CallFunction) -> jsonrpc_core::Result<jsonrpc_core::Value> {
    // Parse all values
    let package: Address;
    match Address::from_str(&params.address) {
        Ok(v) => package = v,
        Err(_) => return invalid_params_err("Package address wrong format"),
    }

    let account: Address;
    match Address::from_str(&params.account_address) {
        Ok(v) => account = v,
        Err(_) => return invalid_params_err("Account wrong format"),
    }

    let signer: Address;
    match Address::from_str(&params.key) {
        Ok(v) => signer = v,
        Err(_) => return invalid_params_err("Signer key wrong format"),
    }
    let signers = vec![signer];

    // Declare receipt here so we can return the lock faster
    let mut receipt: Option<Receipt> = None;

    let write_lock_conf = CONFIG.write();
    let _ = RwLockWriteGuard::map(write_lock_conf, |config| {
        let (epoch, nonce, ledger) = config.load();
        //Do transaction
        let mut executor = TransactionExecutor::new(ledger, epoch, nonce);
        let transaction = TransactionBuilder::new(&executor)
            .call_function(
                package,
                &params.name,
                &params.function,
                params.args,
                Some(account),
            )
            .drop_all_bucket_refs()
            .deposit_all_buckets(account)
            .build(signers);
        if let Ok(transaction) = transaction {
            receipt = Some(executor.run(transaction, false).unwrap());
        }
        //Store the nonce and return the ledger and config
        let nonce = executor.nonce();
        config.store_nonce(nonce);
        config
    });

    match receipt {
        Some(receipt) => {
            let (resources, components, _) = scrypto_helpers::get_call_results(receipt);
            let res: Vec<String> = resources.iter().map(|x| x.to_string()).collect();
            let com: Vec<String> = components.iter().map(|x| x.to_string()).collect();

            Ok(json!({ "resources": res, "components": com }))
        }
        None => transaction_err(),
    }
}

async fn call_method(params: CallMethod) -> jsonrpc_core::Result<Value> {
    // Parse all values
    let component: Address;
    match Address::from_str(&params.address) {
        Ok(v) => component = v,
        Err(_) => return invalid_params_err("Package address wrong format"),
    }

    // parse values
    let account: Address;
    match Address::from_str(&params.account_address) {
        Ok(v) => account = v,
        Err(_) => return invalid_params_err("Account wrong format"),
    }

    let signer: Address;
    match Address::from_str(&params.key) {
        Ok(v) => signer = v,
        Err(_) => return invalid_params_err("Signer key wrong format"),
    }
    let signers = vec![signer];

    // Declare receipt here so we can return the lock faster
    let mut receipt: Option<Receipt> = None;

    let write_lock_conf = CONFIG.write();
    let _ = RwLockWriteGuard::map(write_lock_conf, |config| {
        let (epoch, nonce, ledger) = config.load();
        //Do transaction
        let mut executor = TransactionExecutor::new(ledger, epoch, nonce);
        let transaction = TransactionBuilder::new(&executor)
            .call_method(component, &params.method, params.args, Some(account))
            .drop_all_bucket_refs()
            .deposit_all_buckets(account)
            .build(signers);
        if let Ok(transaction) = transaction {
            if let Ok(r) = executor.run(transaction, false) {
                receipt = Some(r);
            }
        }
        //Store the nonce and return the config
        let nonce = executor.nonce();
        config.store_nonce(nonce);

        config
    });

    if let Some(r) = receipt {
        let mut decoded_results = Vec::new();
        for result in r.results {
            match result {
                Ok(v) => {
                    if let Some(sv) = v {
                        let bytes = &sv.encoded[..];
                        let mut vaults: Vec<Vid> = Vec::new();
                        let lock = CONFIG.read();
                        let mut err = 0;
                        //TODO: Let the formatter not depend on ledger, it only needs it to decode
                        //LazyMaps
                        let _ = parking_lot::RwLockReadGuard::map(lock, |config| {
                            let ledger = config.load_immutable();
                            match formatter::format_data_with_ledger(bytes, ledger, &mut vaults) {
                                Ok(decoded) => decoded_results.push(decoded),
                                Err(e) => {
                                    println!("{:?}", e);
                                    err = 1;
                                }
                            }
                            config
                        });
                        if err == 1 {
                            return decode_err();
                        }
                    }
                }
                Err(e) => return result_err(&e.to_string()),
            }
        }
        return Ok(json!(decoded_results));
    }
    //Should not hit this
    transaction_err()
}

async fn get_balance(params: GetBalance) -> jsonrpc_core::Result<Value> {
    let address: Address;
    match Address::from_str(&params.address) {
        Ok(v) => address = v,
        Err(e) => return invalid_params_err("Component address wrong format"),
    }

    if !address.is_component() {
        return invalid_params_err("Addres isn't a component");
    }

    let mut vids: Vec<Vid> = Vec::new();
    let mut amounts: Option<HashMap<String, Number>> = None;
    let lock = CONFIG.read();
    let _ = parking_lot::RwLockReadGuard::map(lock, |config| {
        let ledger = config.load_immutable();
        if let Some(component) = ledger.get_component(address) {
            if let Ok(state) = component.state(SuperUser) {
                if let Ok(_) = format_data_with_ledger(&state, ledger, &mut vids) {
                    amounts = Some(
                        vids.drain(..)
                            .map(|vid| get_vault_info(ledger, vid))
                            .collect(),
                    );
                }
            }
        }
        ledger
    });

    if let Some(amounts) = amounts {
        return Ok(json!(amounts));
    } else {
        return result_err("Can't get amounts for address");
    }
}

fn get_vault_info(ledger: &InMemoryLedger, vid: Vid) -> (String, Number) {
    if let Some(vault) = ledger.get_vault(vid) {
        if let Ok(amount) = vault.amount(SuperUser) {
            if let Ok(resource_def_address) = vault.resource_address(SuperUser) {
                if let Ok(amount_numb) = Number::from_str(&amount.to_string()) {
                    return (hex::encode(resource_def_address.to_vec()), amount_numb);
                }
            }
        }
    }

    //Should never hit this!!!
    ("deadb33f".to_string(), Number::from_str("-1").unwrap())
}
fn invalid_params_err(slice: &str) -> jsonrpc_core::Result<Value> {
    Err(Error {
        code: ErrorCode::InvalidParams,
        message: slice.to_owned(),
        data: None,
    })
}

fn decode_err() -> jsonrpc_core::Result<Value> {
    Err(Error {
        code: ErrorCode::InternalError,
        message: "DecodeError".to_string(),
        data: None,
    })
}

fn result_err(slice: &str) -> jsonrpc_core::Result<Value> {
    Err(Error {
        code: ErrorCode::InternalError,
        message: slice.to_owned(),
        data: None,
    })
}

fn transaction_err() -> jsonrpc_core::Result<Value> {
    //println!("{:?}", e);
    Err(Error {
        code: ErrorCode::InvalidParams,
        message: "Error while building transaction".to_string(),
        data: None,
    })
}

fn parse_err() -> jsonrpc_core::Result<Value> {
    return Err(jsonrpc_core::Error {
        code: ErrorCode::ParseError,
        message: "Can't parse parameters".to_string(),
        data: None,
    });
}
