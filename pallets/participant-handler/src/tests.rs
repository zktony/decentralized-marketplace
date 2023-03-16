use crate::{
	mock::*, Category, Error, Event, NgoActiveList, NgoInfo, NgoWaitingList, SellerActiveList,
	SellerInfo, SellerWaitingList,
};
use frame_support::{assert_noop, assert_ok};
use pallet_identity::{Data, IdentityInfo};
use sp_core::bounded::BoundedVec;
use sp_runtime::testing::H256;

#[test]
fn test_apply_as_ngo_with_valid_inputs_returns_ok() {
	new_test_ext().execute_with(|| {
		let (ngo, ngo_info) = get_ngo_info();
		add_identity(ngo);
		assert_ok!(ParticipantHandler::apply_as_ngo(RuntimeOrigin::signed(ngo), ngo_info));
		assert_eq!(Balances::reserved_balance(ngo), 1_000_000_000_001);
		assert!(<NgoWaitingList<Test>>::contains_key(ngo));
	})
}

#[test]
fn test_apply_as_ngo_with_anonymous_identity_returns_identity_error() {
	new_test_ext().execute_with(|| {
		let (ngo, ngo_info) = get_ngo_info();
		assert_noop!(
			ParticipantHandler::apply_as_ngo(RuntimeOrigin::signed(ngo), ngo_info),
			Error::<Test>::IdentityNotFound
		);
	})
}

#[test]
fn test_apply_as_ngo_with_already_registered_ngo_returns_error() {
	new_test_ext().execute_with(|| {
		let (ngo, ngo_info) = get_ngo_info();
		<NgoWaitingList<Test>>::insert(ngo, ngo_info.clone());
		add_identity(ngo);
		assert_noop!(
			ParticipantHandler::apply_as_ngo(RuntimeOrigin::signed(ngo), ngo_info),
			Error::<Test>::AlreadyPartOfWaitingList
		);
	})
}

#[test]
fn test_approve_ngo_with_valid_arg_returns_ok() {
	new_test_ext().execute_with(|| {
		let (ngo, ngo_info) = get_ngo_info();
		<NgoWaitingList<Test>>::insert(ngo, ngo_info);
		assert_ok!(ParticipantHandler::approve_ngo(RuntimeOrigin::signed(0), ngo));
	})
}

#[test]
fn test_approve_ngo_with_un_registered_ngo_returns_error() {
	new_test_ext().execute_with(|| {
		let (ngo, _) = get_ngo_info();
		assert_noop!(
			ParticipantHandler::approve_ngo(RuntimeOrigin::signed(0), ngo),
			Error::<Test>::NotPartOfWaitingList
		);
	})
}

#[test]
fn test_approve_ngo_with_already_part_of_active_list_returns_error() {
	new_test_ext().execute_with(|| {
		let (ngo, ngo_info) = get_ngo_info();
		<NgoWaitingList<Test>>::insert(ngo, ngo_info.clone());
		<NgoActiveList<Test>>::insert(ngo, ngo_info);
		assert_noop!(
			ParticipantHandler::approve_ngo(RuntimeOrigin::signed(0), ngo),
			Error::<Test>::AlreadyPartOfActiveList
		);
	})
}

#[test]
fn test_apply_as_seller_with_valid_inputs_returns_ok() {
	new_test_ext().execute_with(|| {
		let (seller, seller_info) = get_seller_info();
		add_identity(seller);
		assert_ok!(ParticipantHandler::apply_as_seller(RuntimeOrigin::signed(seller), seller_info));
		assert_eq!(Balances::reserved_balance(seller), 1_000_000_000_001);
		assert!(<SellerWaitingList<Test>>::contains_key(seller));
	})
}

#[test]
fn test_apply_as_seller_with_anonymous_identity_returns_identity_error() {
	new_test_ext().execute_with(|| {
		let (seller, seller_info) = get_seller_info();
		assert_noop!(
			ParticipantHandler::apply_as_seller(RuntimeOrigin::signed(seller), seller_info),
			Error::<Test>::IdentityNotFound
		);
	})
}

#[test]
fn test_apply_as_seller_with_already_registered_seller_returns_error() {
	new_test_ext().execute_with(|| {
		let (seller, seller_info) = get_seller_info();
		<SellerWaitingList<Test>>::insert(seller, seller_info.clone());
		add_identity(seller);
		assert_noop!(
			ParticipantHandler::apply_as_seller(RuntimeOrigin::signed(seller), seller_info),
			Error::<Test>::AlreadyPartOfWaitingList
		);
	})
}

#[test]
fn test_approve_seller_with_valid_arg_returns_ok() {
	new_test_ext().execute_with(|| {
		let (seller, seller_info) = get_seller_info();
		<SellerWaitingList<Test>>::insert(seller, seller_info);
		assert_ok!(ParticipantHandler::approve_seller(RuntimeOrigin::signed(0), seller));
	})
}

#[test]
fn test_approve_seller_with_un_registered_seller_returns_error() {
	new_test_ext().execute_with(|| {
		let (seller, _) = get_seller_info();
		assert_noop!(
			ParticipantHandler::approve_seller(RuntimeOrigin::signed(0), seller),
			Error::<Test>::NotPartOfWaitingList
		);
	})
}

#[test]
fn test_approve_seller_with_already_part_of_active_list_returns_error() {
	new_test_ext().execute_with(|| {
		let (seller, seller_info) = get_seller_info();
		<SellerWaitingList<Test>>::insert(seller, seller_info.clone());
		<SellerActiveList<Test>>::insert(seller, seller_info);
		assert_noop!(
			ParticipantHandler::approve_seller(RuntimeOrigin::signed(0), seller),
			Error::<Test>::AlreadyPartOfActiveList
		);
	})
}

fn get_ngo_info() -> (u64, NgoInfo<H256>) {
	let ngo: u64 = 1;
	let categories_supported = vec![Category::Pharmaceutical, Category::Clothing];
	let ngo_info = NgoInfo {
		categories: BoundedVec::try_from(categories_supported).unwrap(),
		cid: H256::from([1; 32]),
	};
	(ngo, ngo_info)
}

fn get_seller_info() -> (u64, SellerInfo<H256>) {
	let seller: u64 = 1;
	let seller_info = SellerInfo { category: Category::Pharmaceutical, cid: H256::from([1; 32]) };
	(seller, seller_info)
}

fn add_identity(id: u64) {
	assert_ok!(Balances::set_balance(RuntimeOrigin::root(), id, 1 * 10_000_000_000_000_000u128, 0));
	assert_ok!(Identity::set_identity(RuntimeOrigin::signed(id), Box::new(ten())));
}

fn ten() -> IdentityInfo<MaxAdditionalFields> {
	IdentityInfo {
		additional: BoundedVec::default(),
		display: Data::Raw(b"ten".to_vec().try_into().unwrap()),
		legal: Data::Raw(b"The Right Ordinal Ten, Esq.".to_vec().try_into().unwrap()),
		web: Default::default(),
		riot: Default::default(),
		email: Default::default(),
		pgp_fingerprint: None,
		image: Default::default(),
		twitter: Default::default(),
	}
}
