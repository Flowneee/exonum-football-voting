extern crate exonum;
extern crate football_voting;
#[macro_use] extern crate exonum_testkit;


use exonum::blockchain::Transaction;
use exonum::crypto::{self, PublicKey, SecretKey, Hash};
use exonum::explorer::CommittedTransaction;
use exonum::blockchain::TransactionError;
use exonum_testkit::{TestKit, TestKitBuilder};
use football_voting::schema::{VotesSchema};
use football_voting::transactions::{TxCreateWallet, TxVote};
use football_voting::service::VotesService;
use football_voting::errors::Error;


fn init_testkit() -> TestKit {
    TestKitBuilder::validator()
        .with_service(VotesService)
        .create()
}


#[test]
fn test_create_fan_wallet() {
    let mut testkit = init_testkit();
    let (pubkey, key) = crypto::gen_keypair();
    testkit.create_block_with_transactions(txvec![
        TxCreateWallet::new(&pubkey, "Alice", false, &key),
    ]);
    let wallet = {
        let snapshot = testkit.snapshot();
        VotesSchema::new(&snapshot).fan_wallet(&pubkey).expect(
            "No wallet persisted",
        )
    };
    assert_eq!(*wallet.pub_key(), pubkey);
    assert_eq!(wallet.name(), "Alice");
    assert_eq!(wallet.voted(), false);
    assert_eq!(wallet.vote_hash(), &Hash::zero().to_hex());
}


#[test]
#[should_panic]
fn test_create_duplicate_fan_wallet() {
    let mut testkit = init_testkit();
    let (pubkey, key) = crypto::gen_keypair();
    testkit.create_block_with_transactions(txvec![
        TxCreateWallet::new(&pubkey, "Alice", false, &key),
    ]);
    let block = testkit.create_block_with_transactions(txvec![
        TxCreateWallet::new(&pubkey, "Alice", false, &key),
    ]);
    block.transactions[0].status();
}



#[test]
fn test_create_team_wallet() {
    let mut testkit = init_testkit();
    let (pubkey, key) = crypto::gen_keypair();
    testkit.create_block_with_transactions(txvec![
        TxCreateWallet::new(&pubkey, "Wonderland", true, &key),
    ]);
    let wallet = {
        let snapshot = testkit.snapshot();
        VotesSchema::new(&snapshot).team_wallet(&pubkey).expect(
            "No wallet persisted",
        )
    };
    assert_eq!(*wallet.pub_key(), pubkey);
    assert_eq!(wallet.name(), "Wonderland");
    assert_eq!(wallet.votes(), 0);
}


#[test]
#[should_panic]
fn test_create_duplicate_team_wallet() {
    let mut testkit = init_testkit();
    let (pubkey, key) = crypto::gen_keypair();
    testkit.create_block_with_transactions(txvec![
        TxCreateWallet::new(&pubkey, "Wonderland", true, &key),
    ]);
    let block = testkit.create_block_with_transactions(txvec![
        TxCreateWallet::new(&pubkey, "Wonderland", true, &key),
    ]);
    block.transactions[0].status();
}


#[test]
fn test_vote() {
    let mut testkit = init_testkit();
    let (alice_pubkey, alice_key) = crypto::gen_keypair();
    let (wonderland_pubkey, wonderland_key) = crypto::gen_keypair();
    testkit.create_block_with_transactions(txvec![
        TxCreateWallet::new(&alice_pubkey, "Alice", false, &alice_key),
        TxCreateWallet::new(&wonderland_pubkey, "Wonderland", true, &wonderland_key),
        TxVote::new(&alice_pubkey, &wonderland_pubkey, 0, &alice_key),
    ]);
    let wallets = {
        let snapshot = testkit.snapshot();
        let schema = VotesSchema::new(&snapshot);
        (schema.fan_wallet(&alice_pubkey), schema.team_wallet(&wonderland_pubkey))
    };
    if let (Some(alice_wallet), Some(wonderland_wallet)) = wallets {
        assert_eq!(alice_wallet.voted(), true);
        assert_eq!(wonderland_wallet.votes(), 1);
    } else {
        panic!("Wallets not persisted");
    }
}


#[test]
fn test_multiple_vote() {
    let mut testkit = init_testkit();
    let (alice_pubkey, alice_key) = crypto::gen_keypair();
    let (wonderland_pubkey, wonderland_key) = crypto::gen_keypair();
    testkit.create_block_with_transactions(txvec![
        TxCreateWallet::new(&alice_pubkey, "Alice", false, &alice_key),
        TxCreateWallet::new(&wonderland_pubkey, "Wonderland", true, &wonderland_key),
        TxVote::new(&alice_pubkey, &wonderland_pubkey, 0, &alice_key),
        TxVote::new(&alice_pubkey, &wonderland_pubkey, 1, &alice_key),
    ]);
    let wallet = {
        let snapshot = testkit.snapshot();
        let schema = VotesSchema::new(&snapshot);
        schema.team_wallet(&wonderland_pubkey)
    };
    if let Some(wonderland_wallet) = wallet {
        assert_eq!(wonderland_wallet.votes(), 1);
    } else {
        panic!("Wallets not persisted");
    }
}


#[test]
fn test_vote_with_for_existing_team() {
    let mut testkit = init_testkit();
    let (alice_pubkey, alice_key) = crypto::gen_keypair();
    let (wonderland_pubkey, wonderland_key) = crypto::gen_keypair();
    testkit.create_block_with_transactions(txvec![
        TxCreateWallet::new(&alice_pubkey, "Alice", false, &alice_key),
        TxVote::new(&alice_pubkey, &wonderland_pubkey, 0, &alice_key),
        TxCreateWallet::new(&wonderland_pubkey, "Wonderland", true, &wonderland_key),
    ]);
    let wallets = {
        let snapshot = testkit.snapshot();
        let schema = VotesSchema::new(&snapshot);
        (schema.fan_wallet(&alice_pubkey), schema.team_wallet(&wonderland_pubkey))
    };
    if let (Some(alice_wallet), Some(wonderland_wallet)) = wallets {
        assert_eq!(alice_wallet.voted(), false);
        assert_eq!(wonderland_wallet.votes(), 0);
    } else {
        panic!("Transfer occurred");
    }
}


#[test]
fn test_vote_as_non_existing_fan() {
    let mut testkit = init_testkit();
    let (alice_pubkey, alice_key) = crypto::gen_keypair();
    let (wonderland_pubkey, wonderland_key) = crypto::gen_keypair();
    testkit.create_block_with_transactions(txvec![
        TxCreateWallet::new(&wonderland_pubkey, "Wonderland", true, &wonderland_key),
        TxVote::new(&alice_pubkey, &wonderland_pubkey, 0, &alice_key),
        TxCreateWallet::new(&alice_pubkey, "Alice", false, &alice_key),
    ]);
    let wallets = {
        let snapshot = testkit.snapshot();
        let schema = VotesSchema::new(&snapshot);
        (schema.fan_wallet(&alice_pubkey), schema.team_wallet(&wonderland_pubkey))
    };
    if let (Some(alice_wallet), Some(wonderland_wallet)) = wallets {
        assert_eq!(alice_wallet.voted(), false);
        assert_eq!(wonderland_wallet.votes(), 0);
    } else {
        panic!("Transfer occurred");
    }
}
