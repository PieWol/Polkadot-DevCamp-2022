use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok, BoundedVec};
pub type AccountId = u64;
#[test]
fn can_share_record() {
	let (patient_account_id, patient) = generate_account(1);
	let (doctor_account_id, _doctor) = generate_account(2);
	new_test_ext().execute_with(|| {
		let max_len = MockMaxKeyLength::get() as usize;
		for i in 0..max_len {
			assert_ok!(RecordSharing::share_record(
				patient.clone(),
				doctor_account_id,
				BoundedVec::with_max_capacity(),
				i as u32,
			));
		}

		let shared_records = RecordSharing::records_shared(patient_account_id, doctor_account_id);
		assert_eq!(shared_records.len(), max_len);

		assert_noop!(
			RecordSharing::share_record(
				patient.clone(),
				doctor_account_id,
				BoundedVec::with_max_capacity(),
				4_u32,
			),
			Error::<Test>::VectorFull
		);
	})
}

fn generate_account(account_id: AccountId) -> (AccountId, RuntimeOrigin) {
	(account_id, RuntimeOrigin::signed(account_id))
}
