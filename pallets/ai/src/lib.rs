#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use frame_system::Config as SysConfig;
	use sp_std::{prelude::*, vec, convert::TryInto};

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Origin used to administer the pallet
		type AiCommitteeOrigin: EnsureOrigin<<Self as SysConfig>::RuntimeOrigin>;
	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn latest_nonce)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type LastestNonce<T> = StorageValue<_, u64, ValueQuery>;

	/// The Latest versions that we know various locations support.
	#[pallet::storage]
	#[pallet::getter(fn ask_records)]
	pub(super) type AskRecords<T: Config> = StorageMap<
		_,
		Twox64Concat,
		u64,
		BoundedVec<u8, ConstU32<32>>,
		ValueQuery,
	>;

	/// Last block authored by collator.
	#[pallet::storage]
	#[pallet::getter(fn reply_records)]
	pub type ReplyRecords<T: Config> =
		StorageMap<_, Twox64Concat, u64, BoundedVec<u8, ConstU32<32>>, ValueQuery>;

	/// Last block authored by collator.
	#[pallet::storage]
	#[pallet::getter(fn relayers)]
	pub type Relayers<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, bool, ValueQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Ask {who: T::AccountId, latest_nonce: u64, prompt: BoundedVec<u8, ConstU32<32>>},
		Reply { nonce: u64, answer: BoundedVec<u8, ConstU32<32>> },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		MustBeRelayer
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::call_index(0)]
		#[pallet::weight(1000000000)]
		pub fn ask(origin: OriginFor<T>,  prompt: BoundedVec<u8, ConstU32<32>>) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/main-docs/build/origins/
			let who = ensure_signed(origin)?;
			// Add the evidence to the on-chain list.
			let latest_nonce = Self::latest_nonce();
			<AskRecords<T>>::insert(latest_nonce, prompt.clone());
			// Emit an event.
			Self::deposit_event(Event::Ask { who, latest_nonce, prompt });
			<LastestNonce<T>>::put(latest_nonce + 1);
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::call_index(1)]
		#[pallet::weight(1000000000)]
		pub fn reply(origin: OriginFor<T>, nonce: u64, answer: BoundedVec<u8, ConstU32<32>>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Self::relayers(who), Error::<T>::MustBeRelayer);
			<ReplyRecords<T>>::insert(nonce, answer.clone());
			Self::deposit_event(Event::Reply {nonce, answer});
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(1000000000)]
		pub fn add_relayer(origin: OriginFor<T>, v: T::AccountId) -> DispatchResult {
			T::AiCommitteeOrigin::ensure_origin(origin)?;
			<Relayers<T>>::insert(&v, true);
			Ok(())
		}
	}
}
