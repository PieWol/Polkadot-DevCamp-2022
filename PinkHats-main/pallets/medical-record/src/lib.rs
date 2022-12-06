#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use pallet_record_sharing::EncryptedKey;
	use scale_info::TypeInfo;
	use serde::{Deserialize, Serialize};

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_record_sharing::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type MaxRecordContentLength: Get<u32>;
		type SignatureLength: Get<u32>;
		type MaxRecordLength: Get<u32>;
	}

	#[derive(
		Decode, Encode, Deserialize, Serialize, MaxEncodedLen, Clone, PartialEq, Eq, Debug, TypeInfo,
	)]
	pub enum UserType {
		Patient,
		Doctor,
	}

	type RecordId = u32;
	type RecordContent<T> = BoundedVec<u8, <T as Config>::MaxRecordContentLength>;
	type Signature<T> = BoundedVec<u8, <T as Config>::SignatureLength>;
	type PatientAccountId<T> = <T as frame_system::Config>::AccountId;
	type DoctorAccountId<T> = <T as frame_system::Config>::AccountId;

	#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq, MaxEncodedLen, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub enum Record<T: Config> {
		VerifiedRecord(
			RecordId,
			PatientAccountId<T>,
			DoctorAccountId<T>,
			RecordContent<T>,
			Signature<T>,
		),
		UnverifiedRecord(RecordId, PatientAccountId<T>, RecordContent<T>),
	}

	impl<T: Config> Record<T> {
		pub fn transform_unverified_record(
			record: Record<T>,
			doctor_id: DoctorAccountId<T>,
			signature: Signature<T>,
		) -> Record<T> {
			match record {
				Record::VerifiedRecord(_, _, _, _, _) => record,
				Record::UnverifiedRecord(record_id, patient_id, record_content) =>
					Record::VerifiedRecord(
						record_id,
						patient_id,
						doctor_id,
						record_content,
						signature,
					),
			}
		}

		pub fn get_id(&self) -> u32 {
			match self {
				Record::UnverifiedRecord(id, _, _) => *id,
				Record::VerifiedRecord(id, _, _, _, _) => *id,
			}
		}

		pub fn is_verified(&self) -> bool {
			match self {
				Record::UnverifiedRecord(_, _, _) => false,
				Record::VerifiedRecord(_, _, _, _, _) => true,
			}
		}
	}

	#[pallet::storage]
	#[pallet::getter(fn records)]
	pub type Records<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		UserType,
		BoundedVec<Record<T>, T::MaxRecordLength>,
	>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		AccountCreated(T::AccountId, UserType),
		PatientAddsRecord(PatientAccountId<T>, RecordId),
		DoctorAddsRecordForPatient(PatientAccountId<T>, DoctorAccountId<T>, RecordId),
		DoctorVerifiesRecordForPatient(PatientAccountId<T>, DoctorAccountId<T>, RecordId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		AccountNotFound,
		AccountAlreadyExist,
		InvalidArgument,
		ExceedsMaxRecordLength,
		RecordAlreadyVerified,
		NonExistentRecord,
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub accounts: Vec<(T::AccountId, UserType)>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { accounts: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			for (account_id, user_type) in self.accounts.iter() {
				<Records<T>>::insert(
					account_id.clone(),
					user_type.clone(),
					BoundedVec::with_bounded_capacity(T::MaxRecordLength::get() as usize),
				);
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// Create an account for a patient or a doctor.
		// A single AccountId can only have one account for each UserType
		#[pallet::weight(10_000)]
		pub fn create_account(origin: OriginFor<T>, user_type: UserType) -> DispatchResult {
			let who = ensure_signed(origin)?;

			match Self::records(&who, &user_type) {
				Some(_) => Err(Error::<T>::AccountAlreadyExist.into()),
				None => {
					<Records<T>>::insert(
						who.clone(),
						user_type.clone(),
						BoundedVec::with_bounded_capacity(T::MaxRecordLength::get() as usize),
					);
					Self::deposit_event(Event::AccountCreated(who, user_type));
					Ok(())
				},
			}
		}

		// Let a patient to add an 'unverified' record which can later be verified by a doctor
		#[pallet::weight(10_000)]
		pub fn patient_adds_record(
			origin: OriginFor<T>,
			record_content: RecordContent<T>,
		) -> DispatchResult {
			let patient_id = ensure_signed(origin)?;
			let add_record = |mb_record: &mut Option<BoundedVec<Record<_>, _>>| match mb_record {
				None => Err(Error::<T>::AccountNotFound),
				Some(patient_records) => {
					let new_record_id = patient_records.len() as u32 + 1;
					patient_records
						.try_push(Record::<T>::UnverifiedRecord(
							new_record_id,
							patient_id.clone(),
							record_content,
						))
						.map_err(|_| Error::<T>::ExceedsMaxRecordLength)
				},
			};

			<Records<T>>::mutate(&patient_id, &UserType::Patient, add_record)?;
			let new_record_id = Self::records(&patient_id, &UserType::Patient)
				.expect("records should exist")
				.len() as u32;
			Self::deposit_event(Event::PatientAddsRecord(patient_id, new_record_id));
			Ok(())
		}

		// Let a doctor to add a verified record for a patient.
		#[pallet::weight(10_000)]
		pub fn doctor_adds_record(
			origin: OriginFor<T>,
			patient_id: PatientAccountId<T>,
			record_content: RecordContent<T>,
			signature: Signature<T>,
		) -> DispatchResult {
			let doctor_id = ensure_signed(origin)?;
			ensure!(
				<Records<T>>::contains_key(&doctor_id, &UserType::Doctor),
				Error::<T>::AccountNotFound
			);
			let add_record = |mb_record: &mut Option<BoundedVec<Record<_>, _>>| match mb_record {
				None => Err(Error::<T>::AccountNotFound),
				Some(patient_records) => {
					let new_record_id = patient_records.len() as u32 + 1;
					patient_records
						.try_push(Record::<T>::VerifiedRecord(
							new_record_id,
							patient_id.clone(),
							doctor_id.clone(),
							record_content,
							signature,
						))
						.map_err(|_| Error::<T>::ExceedsMaxRecordLength)
				},
			};

			<Records<T>>::mutate(&patient_id, &UserType::Patient, add_record)?;
			let new_record_id = Self::records(&patient_id, &UserType::Patient)
				.expect("records should exist")
				.len() as u32;
			Self::deposit_event(Event::DoctorAddsRecordForPatient(
				patient_id,
				doctor_id,
				new_record_id,
			));
			Ok(())
		}

		// Let a doctor to verify a given unverified record for a patient.
		#[pallet::weight(10_000)]
		pub fn doctor_verifies_record(
			origin: OriginFor<T>,
			patient_id: PatientAccountId<T>,
			record_id: u32,
			signature: Signature<T>,
		) -> DispatchResult {
			let doctor_id = ensure_signed(origin.clone())?;
			ensure!(
				<Records<T>>::contains_key(&doctor_id, &UserType::Doctor),
				Error::<T>::AccountNotFound
			);
			let mut patient_records = <Records<T>>::get(&patient_id, &UserType::Patient)
				.ok_or(Error::<T>::AccountNotFound)?;

			let record_index_to_verify = (record_id - 1) as usize;

			ensure!(record_index_to_verify < patient_records.len(), Error::<T>::InvalidArgument);

			let record_to_be_verified =
				patient_records.get_mut(record_index_to_verify).expect("record should exist");

			let verified_record = Record::transform_unverified_record(
				record_to_be_verified.clone(),
				doctor_id.clone(),
				signature,
			);

			ensure!(!record_to_be_verified.is_verified(), Error::<T>::RecordAlreadyVerified);

			*record_to_be_verified = verified_record;
			<Records<T>>::set(patient_id.clone(), UserType::Patient, Some(patient_records));

			Self::deposit_event(Event::DoctorVerifiesRecordForPatient(
				patient_id, doctor_id, record_id,
			));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn share_record_with(
			origin: OriginFor<T>,
			recipient_id: T::AccountId,
			encrypted_key: EncryptedKey<T>,
			record_id: RecordId,
		) -> DispatchResult {
			let sender_id = ensure_signed(origin.clone())?;
			ensure!(
				Self::account_exists(&sender_id) && Self::account_exists(&recipient_id),
				Error::<T>::AccountNotFound
			);

			ensure!(
				Self::get_record_by_id(sender_id, UserType::Patient, record_id).is_some(),
				Error::<T>::NonExistentRecord
			);

			pallet_record_sharing::Pallet::<T>::share_record(
				origin,
				recipient_id,
				encrypted_key,
				record_id,
			)
		}
	}

	// helper to read
	impl<T: Config> Pallet<T> {
		pub fn get_record_by_id(
			account_id: T::AccountId,
			user_type: UserType,
			record_id: u32,
		) -> Option<Record<T>> {
			Self::records(&account_id, user_type).and_then(|records| {
				if (records.len() as u32) < record_id {
					return None
				}
				records.into_iter().find(|r| r.get_id() == record_id)
			})
		}

		fn account_exists(account: &T::AccountId) -> bool {
			Self::records(account, &UserType::Patient).is_some() ||
				Self::records(account, &UserType::Doctor).is_some()
		}
	}
}
