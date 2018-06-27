#[macro_use] extern crate assert_matches;
extern crate exonum;
extern crate football_voting as football_voting;
extern crate exonum_testkit;
#[macro_use] extern crate serde_json;


mod constants;


use constants::{ALICE_NAME, BOB_NAME};
use exonum::{api::ApiError,
             crypto::{self, CryptoHash, Hash, PublicKey, SecretKey}};
use exonum_testkit::{ApiKind, TestKit, TestKitApi, TestKitBuilder};


use football_voting::schema::Wallet;
use football_voting::service::CurrencyService;
use football_voting::transactions::{TxCreateWallet, TxTransfer};
use football_voting::SERVICE_NAME;


fn create_testkit() -> (TestKit, CryptocurrencyApi) {
    let testkit = TestKitBuilder::validator()
        .with_service(CurrencyService)
        .create();
    let api = CryptocurrencyApi {
        inner: testkit.api(),
    };
    (testkit, api)
}


struct CryptocurrencyApi {
    inner: TestKitApi,
}


impl CryptocurrencyApi {
    fn create_wallet(&self, name: &str) -> (TxCreateWallet, SecretKey) {
        let (pubkey, key) = crypto::gen_keypair();
        let tx = TxCreateWallet::new(&pubkey, name, &key);

        let tx_info: serde_json::Value =
            self.inner
            .post(ApiKind::Service(SERVICE_NAME), "v1/wallets", &tx);
        assert_eq!(tx_info, json!({ "tx_hash": tx.hash() }));
        (tx, key)
    }

    fn transfer(&self, tx: &TxTransfer) {
        // Code skipped...
    }

    fn get_wallet(&self, pubkey: &PublicKey) -> Wallet {
        self.inner.get(
            ApiKind::Service(SERVICE_NAME),
            &format!("v1/wallet/{}", pubkey.to_string()),
        )
    }

    fn assert_no_wallet(&self, pubkey: &PublicKey) {
        let err = self.inner.get_err(
            ApiKind::Service(SERVICE_NAME),
            &format!("v1/wallet/{}", pubkey.to_string()),
        );

        assert_matches!(
            err,
            ApiError::NotFound(ref body) if body == "Wallet not found"
        );
    }
}


#[test]
fn test_create_wallet() {
    let (mut testkit, api) = create_testkit();
    let (tx, _) = api.create_wallet(ALICE_NAME);
    testkit.create_block();
    let wallet = api.get_wallet(tx.pub_key());
    assert_eq!(wallet.pub_key(), tx.pub_key());
    assert_eq!(wallet.name(), tx.name());
    assert_eq!(wallet.balance(), 100);
}


#[test]
fn test_transfer_from_nonexisting_wallet() {
    let (mut testkit, api) = create_testkit();
    let (tx_alice, key_alice) = api.create_wallet("Alice");
    let (tx_bob, _) = api.create_wallet("Bob");
    testkit.create_block_with_tx_hashes(&[tx_bob.hash()]);
    api.assert_no_wallet(tx_alice.pub_key());
}
