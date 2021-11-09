use frame_support::dispatch::DispatchResult;

use codec::{Decode, Encode};
use sp_core::{RuntimeDebug, H160};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, PartialOrd, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum AssetId {
	ETH,
	Token(H160),
}

pub trait MultiAsset<AccountId> {
	fn total_issuance(asset_id: AssetId) -> u128;

	fn balance(asset_id: AssetId, who: &AccountId) -> u128;

	fn transfer(
		asset_id: AssetId,
		from: &AccountId,
		to: &AccountId,
		amount: u128,
	) -> DispatchResult;

	fn withdraw(asset_id: AssetId, who: &AccountId, amount: u128) -> DispatchResult;

	fn deposit(asset_id: AssetId, who: &AccountId, amount: u128) -> DispatchResult;
}

pub trait SingleAsset<AccountId> {
	fn total_issuance() -> u128;

	fn balance(who: &AccountId) -> u128;

	fn transfer(source: &AccountId, dest: &AccountId, amount: u128) -> DispatchResult;

	fn withdraw(who: &AccountId, amount: u128) -> DispatchResult;

	fn deposit(who: &AccountId, amount: u128) -> DispatchResult;
}
