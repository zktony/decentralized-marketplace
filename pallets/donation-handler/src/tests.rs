use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};
use participant_handler::{Category, NgoInfo, SellerInfo};
use sp_core::H256;

#[test]
fn test_donate_with_valid_arguments_returns_ok() {
	new_test_ext().execute_with(|| {
		create_assets();
		let ngo: u64 = 1;
		let donor: u64 = 2;
		Balances::set_balance(RuntimeOrigin::root(), 2, 1_000_000_000_000_000, 0);
		add_ngo(ngo);
		assert_ok!(DonationHandler::donate(
			RuntimeOrigin::signed(donor),
			ngo,
			Category::Pharmaceutical,
			1_000_000_000_000
		));
	})
}

#[test]
fn test_donate_with_wrong_ngo_returns_error() {
	new_test_ext().execute_with(|| {
		create_assets();
		let ngo: u64 = 1;
		let donor: u64 = 2;
		Balances::set_balance(RuntimeOrigin::root(), 2, 1_000_000_000_000_000, 0);
		assert_noop!(
			DonationHandler::donate(
				RuntimeOrigin::signed(donor),
				ngo,
				Category::Pharmaceutical,
				1_000_000_000_000
			),
			Error::<Test>::RecipientNotValid
		);
	})
}

#[test]
fn test_claim_token_with_valid_args_returns_ok() {
	new_test_ext().execute_with(|| {
		create_assets();
		let seller = 1;
		Balances::set_balance(RuntimeOrigin::root(), seller, 1_000_000_000_000_000, 0);
		add_seller(seller);
		assert_ok!(TokenHandler::mint(
			RuntimeOrigin::signed(1),
			codec::Compact(Category::Pharmaceutical.get_id() as u128),
			seller,
			1_000
		));
		assert_ok!(DonationHandler::claim_token(
			RuntimeOrigin::signed(seller),
			Category::Pharmaceutical,
			100
		));
	});
}

#[test]
fn test_claim_token_wrong_seller_returns_error() {
	new_test_ext().execute_with(|| {
		create_assets();
		let seller = 1;
		Balances::set_balance(RuntimeOrigin::root(), seller, 1_000_000_000_000_000, 0);
		assert_noop!(
			DonationHandler::claim_token(
				RuntimeOrigin::signed(seller),
				Category::Pharmaceutical,
				100
			),
			Error::<Test>::CallerNotValid
		);
	})
}

fn add_ngo(ngo: u64) {
	let ngo_info = NgoInfo { categories: Default::default(), cid: H256::zero() };
	ParticipantHandler::add_ngo_to_active_list(&ngo, ngo_info);
}

fn add_seller(seller: u64) {
	let seller_info = SellerInfo { category: Default::default(), cid: H256::default() };
	ParticipantHandler::add_seller_to_active_list(&seller, seller_info);
}

fn create_assets() {
	Balances::set_balance(RuntimeOrigin::root(), 1, 1_000_000_000_000_000, 0);
	assert_ok!(TokenHandler::create(
		RuntimeOrigin::signed(1),
		codec::Compact(Category::Pharmaceutical.get_id() as u128),
		1u64,
		1u128
	));
	TokenHandler::create(
		RuntimeOrigin::signed(1),
		codec::Compact(Category::Stationery.get_id() as u128),
		1u64,
		1u128,
	);
	TokenHandler::create(
		RuntimeOrigin::signed(1),
		codec::Compact(Category::Clothing.get_id() as u128),
		1u64,
		1u128,
	);
	TokenHandler::create(
		RuntimeOrigin::signed(1),
		codec::Compact(Category::Grocery.get_id() as u128),
		1u64,
		1u128,
	);
}
