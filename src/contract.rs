use cosmwasm_std::{
    entry_point, to_binary, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Response, StdError, StdResult, WasmMsg,
};

use crate::coin_helpers::assert_sent_sufficient_coin;
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, ResolveListingResponse};
use crate::state::{config, config_read, list_resolver, list_resolver_read, Config, Listing};
use cw721::{
    Cw721ExecuteMsg::{Approve, TransferNft},
    Expiration,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, StdError> {
    let config_state = Config { listing_count: 0 };
    // Initiate listing_id with 0
    config(deps.storage).save(&config_state)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {

        // Route messages to appropriate handlers
        ExecuteMsg::PlaceListing {
            nft_contract_address,
            id,
            minimum_bid,
        } => execute_place_listing(deps, env, info, nft_contract_address, id, minimum_bid),
        ExecuteMsg::BidListing { listing_id } => execute_bid_listing(deps, env, info, listing_id),
        ExecuteMsg::WithdrawListing { listing_id } => {
            execute_withdraw_listing(deps, env, info, listing_id)
        }
    }
}

pub fn execute_bid_listing(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    listing_id: String,
) -> Result<Response, ContractError> {

    // Fetch listing from listing_id
    let key = listing_id.as_bytes();
    let mut listing = list_resolver_read(deps.storage).load(key)?;
    if listing.block_limit < _env.block.height {
        return Err(ContractError::AuctionEnded {});
    }

    // check if current bid exceeds the previous one
    let sent_coin = assert_sent_sufficient_coin(&info.funds, listing.max_bid.clone())?;
    let last_bid = listing.max_bid;
    let last_bidder = listing.max_bidder;

    // update bidder
    listing.max_bidder = info.sender.clone();
    listing.max_bid = sent_coin;
    list_resolver(deps.storage).save(key, &listing)?;

    // return money to last bidder
    Ok(Response::new()
        .add_attribute("Bidding", listing_id)
        .add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: last_bidder.to_string(),
            amount: vec![last_bid.unwrap()],
        })))
}

pub fn execute_place_listing(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    nft_contract_address: String,
    id: String,
    minimum_bid: Option<Coin>,
) -> Result<Response, ContractError> {
    // update listing id in store
    let config_state = config(deps.storage).load()?;
    let listing_count = config_state.listing_count + 1;
    let nft_contract = deps.api.addr_validate(&nft_contract_address)?;
    
    // Each auction has a limit for 50000 blocks
    let listing = Listing {
        token_id: id.clone(),
        contract_addr: nft_contract,
        seller: info.sender.clone(),
        max_bid: minimum_bid,
        max_bidder: info.sender.clone(),
        block_limit: _env.block.height + 50000,
    };

    let key = listing_count.to_string();
    // save listing to store
    list_resolver(deps.storage).save(key.as_bytes(), &listing)?;

    // lock nft to contract
    Ok(Response::new()
        .add_attribute("place_listing", id.to_string())
        .add_messages(vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: nft_contract_address.clone(),
                funds: vec![],
                msg: to_binary(&Approve {
                    spender: _env.contract.address.to_string(),
                    token_id: id.clone(),
                    expires: Some(Expiration::AtHeight(_env.block.height + 20000)),
                })?,
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: nft_contract_address,
                funds: vec![],
                msg: to_binary(&TransferNft {
                    recipient: String::from(_env.contract.address.as_str()),
                    token_id: id,
                })?,
            }),
        ]))
}

pub fn execute_withdraw_listing(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    listing_id: String,
) -> Result<Response, ContractError> {
    let key = listing_id.as_bytes();
    let listing = list_resolver_read(deps.storage).load(key)?;

    // Check if the auction ended or not
    if listing.block_limit >= _env.block.height {
        return Err(ContractError::AuctionNotEnded {});
    }
    // remove listing from the store
    list_resolver(deps.storage).remove(key);
    
    // If noone has put a bid then since the max_bidder was initialised with the seller then he will be sent back with his NFT
    // Transfer the locked NFT to highest bidder and bid amount to the seller
    Ok(Response::new()
        .add_attribute("listing_ended", listing_id.to_string())
        .add_messages(vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: listing.contract_addr.to_string(),
                funds: vec![],
                msg: to_binary(&TransferNft {
                    recipient: listing.max_bidder.to_string(),
                    token_id: listing_id.clone(),
                })?,
            }),
            CosmosMsg::Bank(BankMsg::Send {
                to_address: listing.max_bidder.to_string(),
                amount: vec![listing.max_bid.unwrap()],
            }),
        ]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&config_read(deps.storage).load()?),
        QueryMsg::ResolveListing { id } => query_list_resolver(deps, env, id),
    }
}

fn query_list_resolver(deps: Deps, _env: Env, id: String) -> StdResult<Binary> {

    // Fetch listing from listing_id
    let key = id.as_bytes();

    let resp = match list_resolver_read(deps.storage).may_load(key)? {
        Some(listing) => Some(listing),
        None => None,
    };
    let unwrapped_resp = resp.unwrap();
    let resolve_listing = ResolveListingResponse {
        token_id: unwrapped_resp.token_id,
        contract_addr: unwrapped_resp.contract_addr,
        seller: unwrapped_resp.seller,
        max_bid: unwrapped_resp.max_bid,
        max_bidder: unwrapped_resp.max_bidder,
        block_limit: unwrapped_resp.block_limit,
    };
    to_binary(&resolve_listing)
}
