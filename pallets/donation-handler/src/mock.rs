use crate as donation_handler;
use frame_support::traits::{AsEnsureOriginWithArg, ConstU16, ConstU64};
use frame_system as system;
use frame_system::{EnsureRoot, EnsureSigned};
use sp_core::{parameter_types, ConstU32, H256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

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
		Balances: pallet_balances,
		Identity: pallet_identity,
		ParticipantHandler: participant_handler,
		TokenHandler: pallet_assets,
		DonationHandler: donation_handler,
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
	type AccountData = pallet_balances::AccountData<u128>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

pub const TOKEN: u128 = 1_000_000_000_000;

parameter_types! {
	pub const ExistentialDeposit: u128 = 1 * TOKEN;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Test {
	type Balance = u128;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = frame_system::Pallet<Test>;
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
	type WeightInfo = ();
	type RuntimeEvent = RuntimeEvent;
}

parameter_types! {
	pub const AssetDeposit: u128 = 100;
	pub const ApprovalDeposit: u128 = 1;
	pub const StringLimit: u32 = 50;
	pub const MetadataDepositBase: u128 = 10;
	pub const MetadataDepositPerByte: u128 = 1;
}

impl pallet_assets::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Balance = u128;
	type RemoveItemsLimit = ConstU32<1000>;
	type AssetId = u128;
	type AssetIdParameter = codec::Compact<u128>;
	type Currency = Balances;
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<Self::AccountId>>;
	type ForceOrigin = EnsureSigned<Self::AccountId>;
	type AssetDeposit = AssetDeposit;
	type AssetAccountDeposit = AssetDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = StringLimit;
	type Freezer = ();
	type Extra = ();
	type CallbackHandle = ();
	type WeightInfo = ();
}

parameter_types! {
	pub const BasicDeposit: u128 = 1;       // 258 bytes on-chain
	pub const FieldDeposit: u128 = 1;        // 66 bytes on-chain
	pub const SubAccountDeposit: u128 = 1;   // 53 bytes on-chain
	pub const MaxSubAccounts: u32 = 100;
	pub const MaxAdditionalFields: u32 = 100;
	pub const MaxRegistrars: u32 = 20;
}

impl pallet_identity::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type BasicDeposit = BasicDeposit;
	type FieldDeposit = FieldDeposit;
	type SubAccountDeposit = SubAccountDeposit;
	type MaxSubAccounts = MaxSubAccounts;
	type MaxAdditionalFields = MaxAdditionalFields;
	type MaxRegistrars = MaxRegistrars;
	type Slashed = ();
	type ForceOrigin = frame_system::EnsureSigned<Self::AccountId>;
	type RegistrarOrigin = frame_system::EnsureSigned<Self::AccountId>;
	type WeightInfo = pallet_identity::weights::SubstrateWeight<Test>;
}

use frame_support::PalletId;

parameter_types! {
	pub const DonationPalletId: PalletId = PalletId(*b"py/nopls");
}

impl donation_handler::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type TokenHandler = TokenHandler;
	type Currency = ();
	type DonationPalletId = DonationPalletId;
}

parameter_types! {
	pub const NgoStakingAmount: u128 = 1_000_000_000_000;
	pub const SellerStakingAmount: u128 = 1_000_000_000_000;
}

impl participant_handler::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type NgoStakingAmount = NgoStakingAmount;
	type SellerStakingAmount = SellerStakingAmount;
	type Currency = Balances;
	type GovernanceOrigin = EnsureSigned<Self::AccountId>;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}
