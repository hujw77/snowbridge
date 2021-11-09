use frame_support::dispatch::DispatchResult;

use codec::{Decode, Encode, MaxEncodedLen};
use sp_core::{RuntimeDebug, H160, U256};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, PartialOrd, MaxEncodedLen, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum AssetId {
	Ether,
	Token(H160),
}

impl Default for AssetId {
	fn default() -> Self {
		AssetId::Ether
	}
}

pub trait MultiAsset<AccountId> {
	fn total_issuance(asset_id: AssetId) -> U256;

	fn balance(asset_id: AssetId, who: &AccountId) -> U256;

	fn transfer(
		asset_id: AssetId,
		from: &AccountId,
		to: &AccountId,
		amount: U256,
	) -> DispatchResult;

	fn withdraw(asset_id: AssetId, who: &AccountId, amount: U256) -> DispatchResult;

	fn deposit(asset_id: AssetId, who: &AccountId, amount: U256) -> DispatchResult;
}

pub trait SingleAsset<AccountId> {
	fn total_issuance() -> U256;

	fn balance(who: &AccountId) -> U256;

	fn transfer(source: &AccountId, dest: &AccountId, amount: U256) -> DispatchResult;

	fn withdraw(who: &AccountId, amount: U256) -> DispatchResult;

	fn deposit(who: &AccountId, amount: U256) -> DispatchResult;
}

pub trait CreateAsset<AccountId, AssetId, Balance> {
	fn create(asset_id: AssetId, owner: &AccountId, is_sufficient: bool, min_balance: Balance);
}
