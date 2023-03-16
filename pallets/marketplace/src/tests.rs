use crate::{mock::*, Error, Event, ProductInfo};
use frame_support::{assert_noop, assert_ok};
use participant_handler::{Category, NgoInfo, SellerInfo};
use sp_core::H256;

#[test]
fn test_list_product_with_valid_args_returns_ok() {
	new_test_ext().execute_with(|| {
		let seller = 1u64;
		add_seller(seller);
		assert_ok!(Marketplace::list_product(
			RuntimeOrigin::signed(seller),
			Category::Clothing,
			1_000_000_000_000,
			H256([2; 32])
		));
	})
}

#[test]
fn test_list_product_with_unregistered_seller_returns_err() {
	new_test_ext().execute_with(|| {
		let seller = 1u64;
		assert_noop!(
			Marketplace::list_product(
				RuntimeOrigin::signed(seller),
				Category::Clothing,
				1_000_000_000_000,
				H256([2; 32])
			),
			Error::<Test>::SellerNotValid
		);
	})
}

#[test]
fn test_buy_with_without_valid_product_returns_error() {
	new_test_ext().execute_with(|| {
		let seller = 1u64;
		let ngo = 2u64;
		add_ngo(ngo);
		add_seller(seller);
		assert_noop!(
			Marketplace::buy(RuntimeOrigin::signed(ngo), H256::random()),
			Error::<Test>::ProductNotFound
		);
	})
}

fn add_seller(seller: u64) {
	let seller_info = SellerInfo { category: Default::default(), cid: H256::default() };
	ParticipantHandler::add_seller_to_active_list(&seller, seller_info);
}

fn add_ngo(ngo: u64) {
	let ngo_info = NgoInfo { categories: Default::default(), cid: H256::zero() };
	ParticipantHandler::add_ngo_to_active_list(&ngo, ngo_info);
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
