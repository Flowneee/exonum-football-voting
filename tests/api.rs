#[macro_use] extern crate assert_matches;
extern crate exonum;
extern crate football_voting;
#[macro_use] extern crate exonum_testkit;
#[macro_use] extern crate serde_json;


use exonum::crypto::{self, PublicKey, SecretKey, Hash, CryptoHash};
use exonum_testkit::{ApiKind, TestKit, TestKitApi, TestKitBuilder};
use football_voting::transactions::{TxCreateWallet, TxVote};
use football_voting::service::VotesService;
use football_voting::constants::SERVICE_NAME;
use football_voting::wallet::{FanWallet, TeamWallet};


struct VotesApi {
    inner: TestKitApi,
}


fn create_testkit() -> (TestKit, VotesApi) {
    let testkit = TestKitBuilder::validator()
        .with_service(VotesService)
        .create();
    let api = VotesApi {
        inner: testkit.api(),
    };
    (testkit, api)
}


impl VotesApi{
    fn create_fan_wallet(&self, name: &str) -> (TxCreateWallet, SecretKey) {
        let (pubkey, key) = crypto::gen_keypair();
        let tx = TxCreateWallet::new(&pubkey, name, false, &key);
        let tx_info: serde_json::Value = self.inner.post(
            ApiKind::Service(SERVICE_NAME), "v1/create", &tx
        );
        assert_eq!(tx_info, json!({ "tx_hash": tx.hash() }));
        (tx, key)
    }

    fn create_team_wallet(&self, name: &str) -> (TxCreateWallet, SecretKey) {
        let (pubkey, key) = crypto::gen_keypair();
        let tx = TxCreateWallet::new(&pubkey, name, true, &key);
        let tx_info: serde_json::Value = self.inner.post(
            ApiKind::Service(SERVICE_NAME), "v1/create", &tx
        );
        assert_eq!(tx_info, json!({ "tx_hash": tx.hash() }));
        (tx, key)
    }

    fn get_fan_wallet(&self, pubkey: &PublicKey) -> FanWallet {
        self.inner.get(
            ApiKind::Service(SERVICE_NAME),
            &format!("v1/fan/wallet/{}", pubkey.to_string()),
        )
    }

    fn get_team_wallet(&self, pubkey: &PublicKey) -> TeamWallet {
        self.inner.get(
            ApiKind::Service(SERVICE_NAME),
            &format!("v1/team/wallet/{}", pubkey.to_string()),
        )
    }
}


#[test]
fn test_create_fan_wallet() {
    let (mut testkit, api) = create_testkit();
    let (tx, _) = api.create_fan_wallet("Alice");
    testkit.create_block();
    let wallet = api.get_fan_wallet(tx.pub_key());
    assert_eq!(wallet.pub_key(), tx.pub_key());
    assert_eq!(wallet.name(), tx.name());
    assert_eq!(wallet.voted(), false);
    assert_eq!(wallet.vote_hash(), &Hash::zero().to_hex());
}


#[test]
fn test_create_team_wallet() {
    let (mut testkit, api) = create_testkit();
    let (tx, _) = api.create_team_wallet("Wonderland");
    testkit.create_block();
    let wallet = api.get_team_wallet(tx.pub_key());
    assert_eq!(wallet.pub_key(), tx.pub_key());
    assert_eq!(wallet.name(), tx.name());
    assert_eq!(wallet.votes(), 0);
}


#[test]
fn test_vote() {
    let (mut testkit, api) = create_testkit();
    let (fan_tx, fan_key) = api.create_fan_wallet("Alice");
    let (team_tx, team_key) = api.create_team_wallet("Wonderland");
    testkit.create_block();
    let vote_tx = TxVote::new(fan_tx.pub_key(), team_tx.pub_key(), 0, &fan_key);
    let vote_tx_info: serde_json::Value = api.inner.post(
        ApiKind::Service(SERVICE_NAME), "v1/vote", &vote_tx
    );
    testkit.create_block();
    assert_eq!(vote_tx_info, json!({ "tx_hash": vote_tx.hash() }));
    // check fan wallet
    let fan_wallet = api.get_fan_wallet(fan_tx.pub_key());
    assert_eq!(fan_wallet.voted(), true);
    assert_eq!(fan_wallet.vote_hash(), vote_tx.hash().to_hex());
    // check team walelt
    let team_wallet = api.get_team_wallet(team_tx.pub_key());
    assert_eq!(team_wallet.votes(), 1);
}


#[test]
fn test_vote_for_non_existing_team() {
    let (mut testkit, api) = create_testkit();
    let (fan_tx, fan_key) = api.create_fan_wallet("Alice");
    let (team_pubkey, team_key) = crypto::gen_keypair();
    testkit.create_block();
    let vote_tx = TxVote::new(fan_tx.pub_key(), &team_pubkey, 0, &fan_key);
    let vote_tx_info: serde_json::Value = api.inner.post(
        ApiKind::Service(SERVICE_NAME), "v1/vote", &vote_tx
    );
    testkit.create_block();
    assert_eq!(vote_tx_info, json!({ "tx_hash": vote_tx.hash() }));
    // check fan wallet
    let fan_wallet = api.get_fan_wallet(fan_tx.pub_key());
    assert_eq!(fan_wallet.voted(), false);
    assert_eq!(fan_wallet.vote_hash(), &Hash::zero().to_hex());
}


#[test]
fn test_vote_for_as_existing_fan() {
    let (mut testkit, api) = create_testkit();
    let (team_tx, team_key) = api.create_team_wallet("Wonderland");
    let (fan_pubkey, fan_key) = crypto::gen_keypair();
    testkit.create_block();
    let vote_tx = TxVote::new(&fan_pubkey, team_tx.pub_key(), 0, &fan_key);
    let vote_tx_info: serde_json::Value = api.inner.post(
        ApiKind::Service(SERVICE_NAME), "v1/vote", &vote_tx
    );
    testkit.create_block();
    assert_eq!(vote_tx_info, json!({ "tx_hash": vote_tx.hash() }));
    // check fan wallet
    let team_wallet = api.get_team_wallet(team_tx.pub_key());
    assert_eq!(team_wallet.votes(), 0);
}


#[test]
fn test_get_rating() {
    let (mut testkit, api) = create_testkit();
    let (fan_tx, fan_key) = api.create_fan_wallet("Alice");
    let (team1_tx, team1_key) = api.create_team_wallet("Wonderland");
    let (team2_tx, team2_key) = api.create_team_wallet("Underland");
    testkit.create_block();
    let vote_tx = TxVote::new(fan_tx.pub_key(), team1_tx.pub_key(), 0, &fan_key);
    let vote_tx_info: serde_json::Value = api.inner.post(
        ApiKind::Service(SERVICE_NAME), "v1/vote", &vote_tx
    );
    testkit.create_block();
    let rating_info: serde_json::Value = api.inner.get(
        ApiKind::Service(SERVICE_NAME), "v1/rating"
    );
    println!("{:?}", rating_info);
    let teams = match rating_info.as_array() {
        Some(x) => x,
        _ => panic!("Rating is not an array")
    };
    for team in teams {
        let team_obj = match team.as_object() {
            Some(x) => x,
            _ => panic!("Team info is not an object")
        };
        match (team_obj["name"].as_str(), team_obj["votes"].as_str().unwrap().parse::<u64>()) {
            (Some("Wonderland"), Ok(x)) => if x != 1 {
                panic!("Wonderland have incorrect amount of votes")
            },
            (Some("Underland"), Ok(x)) => if x != 0 {
                panic!("Underland have incorrect amount of votes")
            },
            _ => panic!(format!("Incorrect team description: {:?}", team_obj))
        }
    }
}
