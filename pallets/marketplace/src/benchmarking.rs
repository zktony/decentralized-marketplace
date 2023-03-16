use super::*;

#[allow(unused)]
use crate::Pallet as DonationHandler;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::{
	sp_runtime::{traits::Hash, SaturatedConversion},
	traits::{fungibles::Mutate, Currency},
};
use frame_system::RawOrigin;
use participant_handler::{Category, NgoInfo, SellerInfo};
const SEED: u32 = 0;

//TODO: Generate weights and replace default weights

benchmarks! {
	list_product {
		let b in 1 .. 1000;
		let seller: T::AccountId = account("seller", b, SEED);
		// Add seller to Active List
		let seller_info = SellerInfo {
		category: Default::default(),
		cid: T::Hashing::hash_of(&b)
		};
		participant_handler::pallet::Pallet::<T>::add_seller_to_active_list(&seller, seller_info);
		let category = Category::Pharmaceutical;
		let price = 1_000_000_000_u128;
		let cid: T::Hash = T::Hashing::hash_of(&b);
	}: _(RawOrigin::Signed(seller), category, price, cid)

	buy {
		let b in 1 .. 1000;
		let ngo: T::AccountId = account("recipient", b, SEED);
		let ngo_info = NgoInfo {
		categories: Default::default(),
		cid: T::Hashing::hash_of(&b)
		};
		participant_handler::pallet::Pallet::<T>::add_ngo_to_active_list(&ngo, ngo_info);
		let pid: T::Hash = T::Hashing::hash_of(&b);
	}: _(RawOrigin::Signed(ngo), pid)
}
