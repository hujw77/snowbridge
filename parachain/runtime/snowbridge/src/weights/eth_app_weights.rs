//! Autogenerated weights for eth_app
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 3.0.0
//! DATE: 2021-05-08, STEPS: `[50, ]`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN:
//! Some("/tmp/snowbridge-benchmark-tce/spec.json"), DB CACHE: 128

// Executed Command:
// target/release/snowbridge
// benchmark
// --chain
// /tmp/snowbridge-benchmark-tce/spec.json
// --execution
// wasm
// --wasm-execution
// compiled
// --pallet
// eth_app
// --extrinsic
// *
// --repeat
// 20
// --steps
// 50
// --output
// runtime/snowbridge/src/weights/eth_app_weights.rs

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for eth_app.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> eth_app::WeightInfo for WeightInfo<T> {
	fn burn() -> Weight {
		(86_445_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	fn mint() -> Weight {
		(48_462_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
}
