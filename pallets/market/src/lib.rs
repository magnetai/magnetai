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

pub mod weights;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use frame_system::Config as SysConfig;
	use sp_std::{prelude::*, vec, convert::TryInto};
	pub use weights::WeightInfo;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Origin used to administer the pallet
		type MarketCommitteeOrigin: EnsureOrigin<<Self as SysConfig>::RuntimeOrigin>;

		type MarketWeightInfo: WeightInfo;
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
	#[pallet::getter(fn order_records)]
	pub(super) type OrderRecords<T: Config> = StorageMap<
		_,
		Twox64Concat,
		u64,
		BoundedVec<u8, ConstU32<8192>>,
		ValueQuery,
	>;

	/// Last block authored by collator.
	#[pallet::storage]
	#[pallet::getter(fn api_records)]
	pub type ApiRecords<T: Config> =
		StorageMap<_, Twox64Concat, u64, BoundedVec<u8, ConstU32<8192>>, ValueQuery>;

	
	/// Last block authored by collator.
	#[pallet::storage]
	#[pallet::getter(fn metrics_records)]
	pub type MetricsRecords<T: Config> =
		StorageMap<_, Twox64Concat, u64, BoundedVec<u8, ConstU32<8192>>, ValueQuery>;

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
		Order {who: T::AccountId, latest_nonce: u64, model: BoundedVec<u8, ConstU32<8192>>},
		ApiReady { nonce: u64, api: BoundedVec<u8, ConstU32<8192>> },
		MetricsUpdate { nonce: u64, metrics: BoundedVec<u8, ConstU32<8192>> },
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
		#[pallet::weight(T::MarketWeightInfo::new_order(model.len() as u32))]
		pub fn new_order(origin: OriginFor<T>,  model: Vec<u8>) -> DispatchResultWithPostInfo {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/main-docs/build/origins/
			let who = ensure_signed(origin)?;
			// Add the evidence to the on-chain list.
			let latest_nonce = Self::latest_nonce();
			let truncate_model = BoundedVec::<u8, ConstU32<8192>>::truncate_from(model.clone());
			<OrderRecords<T>>::insert(latest_nonce, truncate_model.clone());
			// Emit an event.
			Self::deposit_event(Event::Order { who, latest_nonce, model: truncate_model });
			<LastestNonce<T>>::put(latest_nonce + 1);
			// Return a successful DispatchResultWithPostInfo
			Ok(().into())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::call_index(1)]
		#[pallet::weight(T::MarketWeightInfo::api_ready(api.len() as u32))]
		pub fn api_ready(origin: OriginFor<T>, nonce: u64, api: Vec<u8>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			ensure!(Self::relayers(who), Error::<T>::MustBeRelayer);
			let truncate_api = BoundedVec::<u8, ConstU32<8192>>::truncate_from(api.clone());
			<ApiRecords<T>>::insert(nonce, truncate_api.clone());
			Self::deposit_event(Event::ApiReady {nonce, api: truncate_api});
			Ok(().into())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::call_index(2)]
		#[pallet::weight(T::MarketWeightInfo::upload_metrics(metrics.len() as u32))]
		pub fn upload_metrics(origin: OriginFor<T>, nonce: u64, metrics: Vec<u8>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			ensure!(Self::relayers(who), Error::<T>::MustBeRelayer);
			let truncate_metrics = BoundedVec::<u8, ConstU32<8192>>::truncate_from(metrics.clone());
			<MetricsRecords<T>>::insert(nonce, truncate_metrics.clone());
			Self::deposit_event(Event::MetricsUpdate {nonce, metrics: truncate_metrics});
			Ok(().into())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(T::MarketWeightInfo::add_relayer())]
		pub fn add_relayer(origin: OriginFor<T>, v: T::AccountId) -> DispatchResult {
			T::MarketCommitteeOrigin::ensure_origin(origin)?;
			<Relayers<T>>::insert(&v, true);
			Ok(())
		}
	}
}
