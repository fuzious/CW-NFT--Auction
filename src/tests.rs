#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coin, coins, from_binary, Addr, Coin, Deps, DepsMut};

    use crate::contract::{execute, instantiate, query};
    use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
    use crate::state::Config;

    fn assert_config_state(deps: Deps, expected: Config) {
        let res = query(deps, mock_env(), QueryMsg::Config {}).unwrap();
        let value: Config = from_binary(&res).unwrap();
        assert_eq!(value, expected);
    }

    fn mock_init(deps: DepsMut) {
        let msg = InstantiateMsg {};

        let info = mock_info("creator", &coins(0, "utst"));
        let _res = instantiate(deps, mock_env(), info, msg)
            .expect("contract successfully handles InstantiateMsg");
    }

    fn mock_alice_place_listing(deps: DepsMut, sent: &[Coin]) {
        // alice can register an available name
        let info = mock_info("bob_key", sent);
        let msg = ExecuteMsg::PlaceListing {
            nft_contract_address: Addr::unchecked("contract").to_string(),
            id: "1".to_string(),
            minimum_bid: Some(coin(3, "utst")),
        };
        let _res = execute(deps, mock_env(), info, msg)
            .expect("contract successfully handles PlaceListing message");
    }

    fn mock_alice_place_bid(deps: DepsMut, sent: &[Coin]) {
        let info = mock_info("alice_key", sent);
        let msg = ExecuteMsg::BidListing {
            listing_id: "1".to_string(),
        };
        let _res = execute(deps, mock_env(), info, msg)
            .expect("contract successfully handles BidListing message");
    }

    fn mock_alice_withdraw_listing(deps: DepsMut, sent: &[Coin]) {
        let info = mock_info("alice_key", sent);
        let msg = ExecuteMsg::WithdrawListing {
            listing_id: "1".to_string(),
        };
        let mut env = mock_env();
        env.block.height = env.block.height + 70000;
        let _res = execute(deps, env, info, msg)
            .expect("contract successfully handles WithdrawListing message");
    }

    // instantiates the auction contract
    #[test]
    fn proper_init() {
        let mut deps = mock_dependencies(&[]);

        mock_init(deps.as_mut());

        assert_config_state(deps.as_ref(), Config { listing_count: 0 });
    }

    // Puts an NFT for Auction
    #[test]
    fn place_listing() {
        let mut deps = mock_dependencies(&[]);
        mock_init(deps.as_mut());
        mock_alice_place_listing(deps.as_mut(), &coins(0, "utst"));
    }

    // Puts an NFT for Auction
    // Places a bid on that NFT
    #[test]
    fn place_bid() {
        let mut deps = mock_dependencies(&[]);
        mock_init(deps.as_mut());
        mock_alice_place_listing(deps.as_mut(), &coins(0, "utst"));
        mock_alice_place_bid(deps.as_mut(), &coins(4, "utst"));
    }

    // Test should fail since the bid placed is of a lesser amount
    #[test]
    fn fails_on_place_bid() {
        let mut deps = mock_dependencies(&[]);
        mock_init(deps.as_mut());
        mock_alice_place_listing(deps.as_mut(), &coins(0, "utst"));

        // less bid amount
        let info = mock_info("alice_key", &coins(3, "utst"));
        let msg = ExecuteMsg::BidListing {
            listing_id: "1".to_string(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg);
        match _res {
            Ok(_) => panic!("Must return error"),
            Err(_) => {}
        }
    }

    // Withdraws a listing and transfers the listing and token to the appropriate parties
    #[test]
    fn withdraw_listing() {
        let mut deps = mock_dependencies(&[]);
        mock_init(deps.as_mut());
        mock_alice_place_listing(deps.as_mut(), &coins(0, "utst"));
        mock_alice_place_bid(deps.as_mut(), &coins(4, "utst"));
        mock_alice_withdraw_listing(deps.as_mut(), &coins(0, "utst"))
    }

    // Test should fail since this simulates an environment of 40000 blocks from auction start while auction ends
    #[test]
    fn fails_on_withdraw_listing() {
        let mut deps = mock_dependencies(&[]);
        mock_init(deps.as_mut());
        mock_alice_place_listing(deps.as_mut(), &coins(0, "utst"));
        mock_alice_place_bid(deps.as_mut(), &coins(4, "utst"));
        // mock_alice_withdraw_listing(deps.as_mut(), &coins(0, "utst"))
        let info = mock_info("alice_key", &coins(0, "utst"));
        let msg = ExecuteMsg::WithdrawListing {
            listing_id: "1".to_string(),
        };
        let mut env = mock_env();

        // auction lasts for 50000 blocks so should fail for 40000
        env.block.height = env.block.height + 40000;
        let _res = execute(deps.as_mut(), env, info, msg);
        match _res {
            Ok(_) => panic!("Must return error"),
            Err(_) => {}
        }
    }
}
