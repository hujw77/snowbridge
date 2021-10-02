use frame_support::{
	decl_error, decl_event, decl_module, decl_storage,
	dispatch::DispatchResult,
	traits::{
		fungible, fungible::Balanced,
		Get,
		Imbalance,
		EnsureOrigin,
	},
	storage::StorageValue,
	log,
	weights::Weight,
};
use frame_system::{self as system, ensure_signed};
use sp_core::{U256, H160};
use sp_std::prelude::*;
use sp_std::convert::TryFrom;
use snowbridge_core::{
	ChannelId, Message, MessageId,
	MessageDispatch, Verifier,
};

use envelope::Envelope;

use sp_runtime::{Perbill, traits::{Zero, Convert}};

mod benchmarking;

#[cfg(test)]
mod test;

mod envelope;

type BalanceOf<T> = <<T as Config>::FeeAsset as fungible::Inspect<<T as frame_system::Config>::AccountId>>::Balance;

/// Weight functions needed for this pallet.
pub trait WeightInfo {
	fn submit() -> Weight;
	fn set_reward_fraction() -> Weight;
}

impl WeightInfo for () {
	fn submit() -> Weight { 0 }
	fn set_reward_fraction() -> Weight { 0 }
}

pub trait Config: system::Config {
	type Event: From<Event> + Into<<Self as system::Config>::Event>;

	/// Verifier module for message verification.
	type Verifier: Verifier;

	/// Verifier module for message verification.
	type MessageDispatch: MessageDispatch<Self, MessageId>;

	type FeeAsset: fungible::Balanced<Self::AccountId>;

	/// Source of funds to pay relayers
	type SourceAccount: Get<Self::AccountId>;

	/// Treasury Account
	type TreasuryAccount: Get<Self::AccountId>;

	type FeeConverter: Convert<U256, Option<BalanceOf<Self>>>;

	/// The origin which may update reward related params
	type UpdateOrigin: EnsureOrigin<Self::Origin>;

	/// Weight information for extrinsics in this pallet
	type WeightInfo: WeightInfo;
}

decl_storage! {
	trait Store for Module<T: Config> as IncentivizedInboundModule {
		pub SourceChannel get(fn source_channel) config(): H160;
		pub Nonce: u64;
		pub RewardFraction get(fn reward_fraction) config(): Perbill;

	}
}

decl_event! {
	pub enum Event {
	}
}

decl_error! {
	pub enum Error for Module<T: Config> {
		/// Message came from an invalid outbound channel on the Ethereum side.
		InvalidSourceChannel,
		/// Message has an invalid envelope.
		InvalidEnvelope,
		/// Message has an unexpected nonce.
		InvalidNonce,
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {

		type Error = Error<T>;

		fn deposit_event() = default;

		#[weight = T::WeightInfo::submit()]
		pub fn submit(origin, message: Message) -> DispatchResult {
			let relayer = ensure_signed(origin)?;
			// submit message to verifier for verification
			let log = T::Verifier::verify(&message)?;

			// Decode log into an Envelope
			let envelope: Envelope<T> = Envelope::try_from(log).map_err(|_| Error::<T>::InvalidEnvelope)?;

			// Verify that the message was submitted to us from a known
			// outbound channel on the ethereum side
			if envelope.channel != SourceChannel::get() {
				return Err(Error::<T>::InvalidSourceChannel.into())
			}

			// Verify message nonce
			Nonce::try_mutate(|nonce| -> DispatchResult {
				if envelope.nonce != *nonce + 1 {
					Err(Error::<T>::InvalidNonce.into())
				} else {
					*nonce += 1;
					Ok(())
				}
			})?;

			Self::apportion_fee(envelope.fee, &relayer);

			let message_id = MessageId::new(ChannelId::Incentivized, envelope.nonce);
			T::MessageDispatch::dispatch(envelope.source, message_id, &envelope.payload);

			Ok(())
		}

		#[weight = T::WeightInfo::set_reward_fraction()]
		pub fn set_reward_fraction(origin, fraction: Perbill) -> DispatchResult {
			T::UpdateOrigin::ensure_origin(origin)?;
			RewardFraction::set(fraction);
			Ok(())
		}

	}
}

impl<T: Config> Module<T> {
	// Pay the message submission fee into the relayer and treasury account.
	fn apportion_fee(amount: BalanceOf<T>, relayer: &T::AccountId) {
		if amount.is_zero() {
			return;
		}

		let credit = match T::FeeAsset::withdraw(&T::SourceAccount::get(), amount) {
			Ok(credit) => credit,
			Err(err) => {
				log::error!("Unable to withdraw from source account: {:?}", err);
				return;
			}
		};

		let (relayer_portion, treasury_portion) = credit.split(RewardFraction::get().mul_ceil(amount));
		let _ = T::FeeAsset::resolve(relayer, relayer_portion);
		let _ = T::FeeAsset::resolve(&T::TreasuryAccount::get(), treasury_portion);
	}
}
