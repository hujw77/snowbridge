use crate::mock::{new_tester, AccountId, Assets, Test};
use frame_support::{assert_ok, assert_noop};
use sp_keyring::AccountKeyring as Keyring;
use crate::{Balances, TotalIssuance};
use snowbridge_core::{AssetId, MultiAsset};

use super::*;

fn set_balance(asset_id: AssetId, account_id: &AccountId, amount: u128)
{
	Balances::<Test>::insert(asset_id, &account_id, amount);
	TotalIssuance::insert(asset_id, amount);
}

#[test]
fn deposit_should_increase_balance_and_total_issuance() {
	new_tester().execute_with(|| {
		let asset_id = AssetId::ETH;
		let alice: AccountId = Keyring::Alice.into();
		assert_ok!(<Assets as MultiAsset<_>>::deposit(asset_id, &alice, 500));
		assert_eq!(Balances::<Test>::get(&asset_id, &alice), 500);
		assert_eq!(TotalIssuance::get(&asset_id), 500);

		assert_ok!(<Assets as MultiAsset<_>>::deposit(asset_id, &alice, 20));
		assert_eq!(Balances::<Test>::get(&asset_id, &alice), 520);
		assert_eq!(TotalIssuance::get(&asset_id), 520);
	});
}

#[test]
fn deposit_should_raise_total_issuance_overflow_error() {
	new_tester().execute_with(|| {
		let asset_id = AssetId::ETH;
		let alice: AccountId = Keyring::Alice.into();
		TotalIssuance::insert(&asset_id, u128::MAX);
		assert_noop!(
			<Assets as MultiAsset<_>>::deposit(asset_id, &alice, 1),
			Error::<Test>::TotalIssuanceOverflow
		);
	});
}

#[test]
fn deposit_should_raise_balance_overflow_error() {
	new_tester().execute_with(|| {
		let asset_id = AssetId::ETH;
		let alice: AccountId = Keyring::Alice.into();
		Balances::<Test>::insert(&asset_id, &alice, u128::MAX);

		assert_noop!(
			<Assets as MultiAsset<_>>::deposit(asset_id, &alice, 1),
			Error::<Test>::BalanceOverflow
		);
	});
}

#[test]
fn withdrawal_should_decrease_balance_and_total_issuance() {
	new_tester().execute_with(|| {
		let alice: AccountId = Keyring::Alice.into();
		set_balance(AssetId::ETH, &alice, 500);

		assert_ok!(<Assets as MultiAsset<_>>::withdraw(AssetId::ETH, &alice, 20));
		assert_eq!(Balances::<Test>::get(AssetId::ETH, &alice), 480);
		assert_eq!(TotalIssuance::get(AssetId::ETH), 480);
	});
}

#[test]
fn withdrawal_should_raise_total_issuance_underflow_error() {
	new_tester().execute_with(|| {
		let asset_id = AssetId::ETH;
		let alice: AccountId = Keyring::Alice.into();
		TotalIssuance::insert(&asset_id, 1);

		assert_noop!(
			<Assets as MultiAsset<_>>::withdraw(asset_id, &alice, 10),
			Error::<Test>::TotalIssuanceUnderflow
		);

	});
}

#[test]
fn withdrawal_should_raise_balance_underflow_error() {
	new_tester().execute_with(|| {
		let asset_id = AssetId::ETH;
		let alice: AccountId = Keyring::Alice.into();
		TotalIssuance::insert(&asset_id, 500);

		assert_noop!(
			<Assets as MultiAsset<_>>::withdraw(asset_id, &alice, 10),
			Error::<Test>::InsufficientBalance
		);

	});
}

#[test]
fn transfer_free_balance() {
	new_tester().execute_with(|| {

		let asset_id = AssetId::ETH;
		let alice: AccountId = Keyring::Alice.into();
		let bob: AccountId = Keyring::Bob.into();

		assert_ok!(<Assets as MultiAsset<_>>::deposit(asset_id, &alice, 500));
		assert_ok!(<Assets as MultiAsset<_>>::deposit(asset_id, &bob, 500));
		assert_ok!(<Assets as MultiAsset<_>>::transfer(asset_id, &alice, &bob, 250));

		assert_eq!(Balances::<Test>::get(&asset_id, &alice), 250);
		assert_eq!(Balances::<Test>::get(&asset_id, &bob), 750);
		assert_eq!(TotalIssuance::get(&asset_id), 1000);
	});
}

#[test]
fn transfer_should_raise_insufficient_balance() {
	new_tester().execute_with(|| {

		let asset_id = AssetId::ETH;
		let alice: AccountId = Keyring::Alice.into();
		let bob: AccountId = Keyring::Bob.into();

		assert_ok!(<Assets as MultiAsset<_>>::deposit(asset_id, &alice, 500));
		assert_ok!(<Assets as MultiAsset<_>>::deposit(asset_id, &bob, 500));

		assert_noop!(
			<Assets as MultiAsset<_>>::transfer(asset_id, &alice, &bob, 1000),
			Error::<Test>::InsufficientBalance,
		);
	});
}
