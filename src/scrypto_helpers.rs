use radix_engine::transaction::Receipt;
use scrypto::prelude::*;

pub fn address_to_signers(address: Address) -> Result<Vec<Address>, Box<dyn std::error::Error>> {
    let mut v = Vec::<Address>::new();
    if address.is_public_key() {
        v.push(address)
    } else {
        return Err("erRori".into());
    }
    Ok(v)
}

pub fn get_call_results(receipt: Receipt) -> (Vec<Address>, Vec<Address>, Vec<Address>) {
    let packages = receipt
        .new_entities
        .iter()
        .filter(|a| matches!(a, Address::Package(_)))
        .map(Clone::clone)
        .collect();

    let components = receipt
        .new_entities
        .iter()
        .filter(|a| matches!(a, Address::Component(_)))
        .map(Clone::clone)
        .collect();

    let resources = receipt
        .new_entities
        .iter()
        .filter(|a| matches!(a, Address::ResourceDef(_)))
        .map(Clone::clone)
        .collect();

    (resources, components, packages)
}
