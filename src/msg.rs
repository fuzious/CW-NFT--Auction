use cosmwasm_std::{Addr, Coin};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    // Place an NFT on Auction
    PlaceListing {
        nft_contract_address: String,
        id: String,
        minimum_bid: Option<Coin>,
    },
    // Bid on an NFT already put on Auction
    BidListing {
        listing_id: String,
    },
    // Withdraw an ended Auction
    WithdrawListing {
        listing_id: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    // Resolve listing returns all the details of a listing
    ResolveListing { id: String },
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ResolveListingResponse {
    pub token_id: String,

    pub contract_addr: Addr,

    pub seller: Addr,

    pub max_bid: Option<Coin>,

    pub max_bidder: Addr,

    pub block_limit: u64,
}
