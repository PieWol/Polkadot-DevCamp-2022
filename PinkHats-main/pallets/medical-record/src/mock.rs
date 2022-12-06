use crate::{self as pallet_medical_record, UserType};
use frame_support::{
	parameter_types,
	traits::{ConstU16, ConstU64, GenesisBuild},
};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

pub type AccountId = u64;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system,
		MedicalRecord: pallet_medical_record,
		RecordSharing: pallet_record_sharing,
	}
);

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_medical_record::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type MaxRecordContentLength = MockMaxRecordContentLength;
	type SignatureLength = MockSignatureLength;
	type MaxRecordLength = MockMaxRecordLength;
}

parameter_types! {
	pub const MockMaxRecordContentLength: u32 = 1;
	pub const MockSignatureLength: u32 = 3;
	pub const MockMaxRecordLength: u32 = 3;
	pub const MockMaxKeyLength: u32 = 3;
}

impl pallet_record_sharing::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type MaxKeyLength = MockMaxKeyLength;
}

#[derive(Default)]
pub struct ExternalitiesBuilder {
	accounts: Vec<(AccountId, UserType)>,
}

impl ExternalitiesBuilder {
	pub fn with_accounts(mut self, accounts: Vec<(AccountId, UserType)>) -> Self {
		self.accounts = accounts;
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.expect("Frame system builds valid default genesis config");

		pallet_medical_record::GenesisConfig::<Test> { accounts: self.accounts }
			.assimilate_storage(&mut t)
			.expect("Can build genesis for medical_record pallet");

		sp_io::TestExternalities::from(t)
	}
}
