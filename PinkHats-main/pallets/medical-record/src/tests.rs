use crate::{mock::*, Error, UserType};
use frame_support::{assert_noop, assert_ok, bounded_vec, BoundedVec};

use sp_core::Get;

#[test]
fn user_can_create_account() {
	let (patient_account_id, patient) = generate_account(1);
	ExternalitiesBuilder::default().build().execute_with(|| {
		assert_ok!(MedicalRecord::create_account(patient.clone(), UserType::Patient));

		let account_created =
			MedicalRecord::records(patient_account_id, UserType::Patient).is_some();
		assert!(account_created, "failed to create an account");

		assert_noop!(
			MedicalRecord::create_account(patient, UserType::Patient),
			Error::<Test>::AccountAlreadyExist
		);
	});
}

#[test]
fn patient_can_add_record() {
	let (patient_account_id, patient) = generate_account(1);
	ExternalitiesBuilder::default()
		.with_accounts(vec![(patient_account_id, UserType::Patient)])
		.build()
		.execute_with(|| {
			let max_record_len = <MockMaxRecordLength as Get<u32>>::get() as usize;
			for _ in 0..max_record_len {
				assert_ok!(MedicalRecord::patient_adds_record(
					patient.clone(),
					BoundedVec::with_max_capacity()
				));
			}

			let records = MedicalRecord::records(patient_account_id, UserType::Patient)
				.expect("The record should exist");
			assert_eq!(records.len(), max_record_len);
			assert_eq!(records.into_iter().filter(|r| !r.is_verified()).count(), max_record_len);

			assert_noop!(
				MedicalRecord::patient_adds_record(
					patient.clone(),
					BoundedVec::with_max_capacity()
				),
				Error::<Test>::ExceedsMaxRecordLength
			);
		});
}

#[test]
fn doctor_can_add_record_for_patient() {
	let (patient_account_id, patient) = generate_account(1);
	let (doctor_account_id, doctor) = generate_account(2);
	ExternalitiesBuilder::default()
		.with_accounts(vec![
			(patient_account_id, UserType::Patient),
			(doctor_account_id, UserType::Doctor),
		])
		.build()
		.execute_with(|| {
			let max_record_len = <MockMaxRecordLength as Get<u32>>::get() as usize;
			for _ in 0..max_record_len {
				assert_ok!(MedicalRecord::doctor_adds_record(
					doctor.clone(),
					patient_account_id,
					BoundedVec::with_max_capacity(),
					BoundedVec::with_max_capacity(),
				));
			}

			let patient_records = MedicalRecord::records(patient_account_id, UserType::Patient)
				.expect("the record should exist");
			assert_eq!(patient_records.len(), max_record_len);
			assert_eq!(
				patient_records.into_iter().filter(|r| r.is_verified()).count(),
				max_record_len
			);

			assert_noop!(
				MedicalRecord::patient_adds_record(
					patient.clone(),
					BoundedVec::with_max_capacity()
				),
				Error::<Test>::ExceedsMaxRecordLength
			);
		});
}

#[test]
fn doctor_can_transform_unverified_record() {
	let (patient_account_id, patient) = generate_account(1);
	let (doctor_account_id, doctor) = generate_account(2);
	ExternalitiesBuilder::default()
		.with_accounts(vec![
			(patient_account_id, UserType::Patient),
			(doctor_account_id, UserType::Doctor),
		])
		.build()
		.execute_with(|| {
			// Patient submits a record
			let max_record_len = <MockMaxRecordLength as Get<u32>>::get() as usize;
			for _ in 0..max_record_len {
				assert_ok!(MedicalRecord::patient_adds_record(
					patient.clone(),
					BoundedVec::with_max_capacity()
				));
			}
			let record_id_to_verify = 1.min(max_record_len as u32 / 2);
			// Doctor verifies the second record
			let signature: BoundedVec<u8, MockSignatureLength> = bounded_vec![];
			assert_ok!(MedicalRecord::doctor_verifies_record(
				doctor.clone(),
				patient_account_id,
				record_id_to_verify,
				signature.clone()
			));

			let verified_record = MedicalRecord::get_record_by_id(
				patient_account_id,
				UserType::Patient,
				record_id_to_verify,
			)
			.expect("Record should exist");

			assert!(verified_record.is_verified());

			let patient_records = MedicalRecord::records(patient_account_id, UserType::Patient)
				.expect("records should exists");
			assert_eq!(
				patient_records.into_iter().filter(|r| !r.is_verified()).count(),
				max_record_len - 1
			);

			// A record verified once cannot be verified again
			assert_noop!(
				MedicalRecord::doctor_verifies_record(
					doctor.clone(),
					patient_account_id,
					record_id_to_verify,
					signature
				),
				Error::<Test>::RecordAlreadyVerified
			);
		});
}

fn generate_account(account_id: AccountId) -> (AccountId, RuntimeOrigin) {
	(account_id, RuntimeOrigin::signed(account_id))
}
