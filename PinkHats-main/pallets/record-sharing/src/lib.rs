#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

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
	use frame_support::{pallet_prelude::*, BoundedVec};
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type MaxKeyLength: Get<u32>;
	}
	type RecordId = u32;
	pub type EncryptedKey<T> = BoundedVec<u32, <T as Config>::MaxKeyLength>;
	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	#[pallet::getter(fn records_shared)]
	pub type SharedRecords<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<(EncryptedKey<T>, RecordId), T::MaxKeyLength>,
		ValueQuery,
	>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		SharingStored(T::AccountId, RecordId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		// If the User tries to share a record a user he already shared it to.
		AlreadySharedToThisUser,
		// If the user trying to share with doesn't exist.
		UserUnregistered,
		// Given RecordId doesn't exist
		WrongRecordId,
		//Key is too long
		KeyTooLong,
		VectorFull,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn share_record(
			origin: OriginFor<T>,
			recipient: T::AccountId,
			encrypted_key: EncryptedKey<T>,
			record_id: RecordId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			<SharedRecords<T>>::try_mutate(who, recipient.clone(), |x| {
				x.try_push((encrypted_key, record_id)).map_err(|_| Error::<T>::VectorFull)?;
				Ok::<(), Error<T>>(())
			})?;

			Self::deposit_event(Event::<T>::SharingStored(recipient, record_id));
			Ok(())
		}
	}
}
