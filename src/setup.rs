use super::config::Config;
use super::scrypto_helpers;
use super::CONFIG;
use super::LEDGER;

use parking_lot::{RwLock, RwLockWriteGuard};

use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

use std::collections::HashMap;
use std::sync::{atomic, Arc};

#[derive(serde::Serialize, serde::Deserialize)]
struct PackageSetup {
    name: String,
    path_to_wasm: String,
    call_new: bool,
    args: Vec<String>,
    components: Vec<String>,
    resources: Vec<String>,
}

impl PackageSetup {
    fn new(
        name: &str,
        path_to_wasm: &str,
        call_new: bool,
        args: Vec<&str>,
        components: Vec<&str>,
        resources: Vec<&str>,
    ) -> PackageSetup {
        let args_owned: Vec<String> = args.into_iter().map(|x| x.to_owned()).collect();
        let comps_owned: Vec<String> = components.into_iter().map(|x| x.to_owned()).collect();
        let resources_owned: Vec<String> = resources.into_iter().map(|x| x.to_owned()).collect();
        PackageSetup {
            name: name.to_owned(),
            path_to_wasm: path_to_wasm.to_owned(),
            call_new,
            args: args_owned,
            components: comps_owned,
            resources: resources_owned,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct PackagesSetup {
    packages: Vec<PackageSetup>,
}

pub fn run_setup() {
    // Everything in the setup uses unwrap() hard failure is the desired outcome

    println!("Setting up ledger");
    // Load Config.json
    let package_file = std::fs::File::open("Config.json").unwrap();
    let json: PackagesSetup = jsonrpc_core::serde_json::from_reader(package_file).unwrap();

    // Create admin account, dummy values
    let mut admin_key: Address =
        Address::from_str("02b8dd9f4232ce3c00dcb3496956fb57096d5d50763b989ca56f3b").unwrap();
    let mut admin_account: Address =
        Address::from_str("02b9f7c0c44a6e2162403cea3fa44500dff50eb18fd4ff5a9dd079").unwrap();

    let writer_lock = LEDGER.write();

    // We use a closure to access the inner of an RwLock, notice we return the &mut ledger
    // at the very end to put it back in the RwLock, ONLY CALL THIS WHEN YOU HAVE ALREADY ACQUIRED
    // THE LOCK
    let mapped = RwLockWriteGuard::map(writer_lock, |ledger| {
        let mut executor = TransactionExecutor::new(ledger, 0, 0);

        //Create admin account real values
        admin_key = executor.new_public_key();
        admin_account = executor.new_account(admin_key);
        println!(
            "admin_key: {} \n admin_account: {}",
            &admin_key, &admin_account
        );

        let mut resources_hm: HashMap<String, String> = HashMap::new();
        let mut components_hm: HashMap<String, String> = HashMap::new();
        let mut packages_hm: HashMap<String, String> = HashMap::new();

        // For all entries in Config.json
        for pack in json.packages {
            // Publish the packages
            let bytes = std::fs::read(pack.path_to_wasm).unwrap();
            let code = &bytes[..];

            let signers = scrypto_helpers::address_to_signers(admin_key).unwrap();
            let transaction = TransactionBuilder::new(&executor)
                .publish_package(&code)
                .build(signers)
                .unwrap();

            let receipt = executor.run(transaction, false).unwrap();

            // Grab the package
            let package_address = receipt.package(0).unwrap();
            println!("Package {} published: {:?}", &pack.name, package_address);
            packages_hm.insert(pack.name.to_owned(), package_address.to_string());

            // Call the function new on the packages where the bool is enabled, all those that
            // don't get instantiated from another component
            if !pack.call_new {
                continue;
            }

            let transaction = TransactionBuilder::new(&executor)
                .call_function(
                    package_address,
                    &pack.name,
                    "new",
                    pack.args,
                    Some(admin_account),
                )
                .drop_all_bucket_refs()
                .deposit_all_buckets(admin_account)
                .build(vec![admin_key])
                .unwrap();

            let receipt = executor.run(transaction, false).unwrap();

            let (resources, components, _) = scrypto_helpers::get_call_results(receipt);
            assert_eq!(resources.len(), pack.resources.len());
            assert_eq!(components.len(), pack.components.len());

            // Push everything to hashmap to store in db
            for (address, name) in resources.iter().zip(pack.resources) {
                println!("Resource {} instantiated: {:?}", &name, &address);
                resources_hm.insert(name.to_owned(), address.to_string());
            }

            for (address, name) in components.iter().zip(pack.components) {
                println!("Component {} instantiated: {:?}", name, address);
                components_hm.insert(name.to_owned(), address.to_string());
            }
        }

        push_all_serverless(
            resources_hm,
            components_hm,
            admin_account.to_string(),
            admin_key.to_string(),
        );

        let write = CONFIG.write();
        let _ = RwLockWriteGuard::map(write, |config| {
            config.store_nonce(&executor);
            config
        });

        //Pass back the ledger to the RwLockWriteGuard
        ledger
    });
}

pub fn create_setup_file_example() {
    let mut packages: Vec<PackageSetup> = Vec::new();
    let package = PackageSetup::new(
        "GumballMachine",
        "/home/user/dev/rs/radixdlt-scrypto/examples/core/gumball-machine/target/wasm32-unknown-unknown/release/gumball_machine.wasm",
        true,
        vec!["0.5"],
        vec!["machine0"],
        vec!["Gumballs"]
    );
    packages.push(package);

    let package = PackageSetup::new(
        "GumballMachine",
        "/home/user/dev/rs/radixdlt-scrypto/examples/core/gumball-machine/target/wasm32-unknown-unknown/release/gumball_machine.wasm",
        false,
        vec!["1.5"],
        vec!["machine1"],
        vec!["Ballz"]
    );

    packages.push(package);

    let example = PackagesSetup { packages };

    let f = std::fs::File::create("Config.json").unwrap();
    let _ = jsonrpc_core::serde_json::to_writer_pretty(f, &example);
}

fn push_all_serverless(
    resources_hm: HashMap<String, String>,
    components_hm: HashMap<String, String>,
    admin_account: String,
    admin_key: String,
) {
}
