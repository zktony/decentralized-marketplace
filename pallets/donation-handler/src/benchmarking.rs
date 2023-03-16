use super::*;

#[allow(unused)]
use crate::Pallet as DonationHandler;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::{
	sp_runtime::{traits::Hash, SaturatedConversion},
	traits::{fungibles::Mutate, Currency},
};
use frame_system::RawOrigin;
use participant_handler::{Category, NgoInfo};
const SEED: u32 = 0;

//TODO: Generate weights and replace default weights

benchmarks! {
	donate {
		let b in 1 .. 1000;
		let donor: T::AccountId = account("ngo", b, SEED);
		let ngo: T::AccountId = account("recipient", b, SEED);
		<T as pallet::Config>::Currency::deposit_into_existing(&donor, 1_000_000_000_000_u128.saturated_into());
		let ngo_info = NgoInfo {
		categories: Default::default(),
		cid: T::Hashing::hash_of(&b)
		};
		participant_handler::pallet::Pallet::<T>::add_ngo_to_active_list(&ngo, ngo_info);
	}: _(RawOrigin::Signed(donor), ngo, Category::Pharmaceutical, 1_000_000_000)

	claim_token {
		let b in 1 .. 1000;
		let seller: T::AccountId = account("seller", b, SEED);
		let category = Category::Pharmaceutical;
		<T as pallet::Config>::Currency::deposit_into_existing(&seller, 1_000_000_000_000_u128.saturated_into());
		<T as pallet::Config>::TokenHandler::mint_into(category.get_id() as u128, &seller, 1_000_u128.saturated_into());
	}: _(RawOrigin::Signed(seller), Category::Pharmaceutical, 100)
}
