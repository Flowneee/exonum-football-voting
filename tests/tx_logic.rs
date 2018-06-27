extern crate exonum;
extern crate football_voting;
#[macro_use] extern crate exonum_testkit;


mod constants;


use constants::{ALICE_NAME, BOB_NAME};
use exonum::blockchain::Transaction;
use exonum::crypto::{self, PublicKey, SecretKey};
use exonum_testkit::{TestKit, TestKitBuilder};
use football_voting::schema::{CurrencySchema, Wallet};
use football_voting::transactions::{TxCreateWallet, TxTransfer};
use football_voting::service::CurrencyService;


fn init_testkit() -> TestKit {
    TestKitBuilder::validator()
        .with_service(CurrencyService)
        .create()
}


#[test]
fn test_create_wallet() {
    let mut testkit = init_testkit();
    let (pubkey, key) = crypto::gen_keypair();
    testkit.create_block_with_transactions(txvec![
        TxCreateWallet::new(&pubkey, "Alice", &key),
    ]);
    let wallet = {
        let snapshot = testkit.snapshot();
        CurrencySchema::new(&snapshot).wallet(&pubkey).expect(
            "No wallet persisted",
        )
    };
    assert_eq!(*wallet.pub_key(), pubkey);
    assert_eq!(wallet.name(), "Alice");
    assert_eq!(wallet.balance(), 100);
}


#[test]
fn test_transfer() {
    let mut testkit = init_testkit();
    let (alice_pubkey, alice_key) = crypto::gen_keypair();
    let (bob_pubkey, bob_key) = crypto::gen_keypair();
    testkit.create_block_with_transactions(txvec![
        TxCreateWallet::new(&alice_pubkey, "Alice", &alice_key),
        TxCreateWallet::new(&bob_pubkey, "Bob", &bob_key),
        TxTransfer::new(
            &alice_pubkey,
            &bob_pubkey,
            10,
            0,
            &alice_key,
        ),
    ]);
    let wallets = {
        let snapshot = testkit.snapshot();
        let schema = CurrencySchema::new(&snapshot);
        (schema.wallet(&alice_pubkey), schema.wallet(&bob_pubkey))
    };
    if let (Some(alice_wallet), Some(bob_wallet)) = wallets {
        assert_eq!(alice_wallet.balance(), 90);
        assert_eq!(bob_wallet.balance(), 110);
    } else {
        panic!("Wallets not persisted");
    }
}


#[test]
fn test_transfer_to_nonexisting_wallet() {
    let mut testkit = init_testkit();
    let (alice_pubkey, alice_key) = crypto::gen_keypair();
    let (bob_pubkey, bob_key) = crypto::gen_keypair();
    testkit.create_block_with_transactions(txvec![
        TxCreateWallet::new(&alice_pubkey, "Alice", &alice_key),
        TxTransfer::new(&alice_pubkey, &bob_pubkey, 10, 0, &alice_key),
        TxCreateWallet::new(&bob_pubkey, "Bob", &bob_key),
    ]);
    let wallets = {
        let snapshot = testkit.snapshot();
        let schema = CurrencySchema::new(&snapshot);
        (schema.wallet(&alice_pubkey), schema.wallet(&bob_pubkey))
    };
    if let (Some(alice_wallet), Some(bob_wallet)) = wallets {
        assert_eq!(alice_wallet.balance(), 100);
        assert_eq!(bob_wallet.balance(), 100);
    } else {
        panic!("Transfer occurred");
    }
}
