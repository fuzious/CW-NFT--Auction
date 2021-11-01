use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Coin, Storage};
use cosmwasm_storage::{
    bucket, bucket_read, singleton, singleton_read, Bucket, ReadonlyBucket, ReadonlySingleton,
    Singleton,
};
pub static CONFIG_KEY: &[u8] = b"config";
pub static LIST_RESOLVER_KEY: &[u8] = b"listingresolver";

// pub const OFFERINGS_COUNT: Item<u64> = Item::new(b"num_offerings");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub listing_count: u64,
}

pub fn config(storage: &mut dyn Storage) -> Singleton<Config> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read(storage: &dyn Storage) -> ReadonlySingleton<Config> {
    singleton_read(storage, CONFIG_KEY)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Listing {
    pub token_id: String,

    pub contract_addr: Addr,

    pub seller: Addr,

    pub max_bid: Option<Coin>,

    pub max_bidder: Addr,

    pub block_limit: u64,
}

pub fn list_resolver(storage: &mut dyn Storage) -> Bucket<Listing> {
    bucket(storage, LIST_RESOLVER_KEY)
}

pub fn list_resolver_read(storage: &dyn Storage) -> ReadonlyBucket<Listing> {
    bucket_read(storage, LIST_RESOLVER_KEY)
}
