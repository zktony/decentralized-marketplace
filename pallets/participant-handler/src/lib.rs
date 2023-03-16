#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub type BalanceOf<T> = <T as pallet_balances::Config>::Balance;

#[frame_support::pallet]

pub mod pallet {
	use frame_support::{
		pallet_prelude::*,
		sp_runtime::SaturatedConversion,
		traits::{Currency, NamedReservableCurrency, ReservableCurrency},
	};
	use frame_system::pallet_prelude::*;

	#[derive(
		Encode, Decode, Clone, Copy, Debug, MaxEncodedLen, TypeInfo, PartialOrd, PartialEq, Eq,
	)]
	pub enum Category {
		Pharmaceutical = 0,
		Stationery = 1,
		Grocery = 2,
		Clothing = 3,
	}

	impl Default for Category {
		fn default() -> Self {
			Self::Pharmaceutical
		}
	}

	impl Category {
		pub fn get_id(&self) -> u8 {
			*self as u8
		}
	}

	#[derive(
		Encode, Decode, Clone, Debug, MaxEncodedLen, TypeInfo, Default, PartialOrd, PartialEq,
	)]
	pub struct SellerInfo<Hash> {
		pub category: Category,
		pub cid: Hash,
	}

	impl<Hash> SellerInfo<Hash> {
		pub fn is_category_allowed(&self, category: &Category) -> bool {
			*category == self.category
		}
	}

	#[derive(
		Encode, Decode, Clone, Debug, MaxEncodedLen, TypeInfo, Default, PartialOrd, PartialEq,
	)]
	pub struct NgoInfo<Hash> {
		pub categories: BoundedVec<Category, ConstU32<100>>,
		pub cid: Hash,
	}

	impl<Hash> NgoInfo<Hash> {
		pub fn is_category_allowed(&self, category: &Category) -> bool {
			self.categories.contains(&category)
		}
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_identity::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Ngo Staking Amount
		type NgoStakingAmount: Get<u128>;
		/// Seller Staking Amount
		type SellerStakingAmount: Get<u128>;
		/// Balances Pallet
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		/// Governance Origin
		type GovernanceOrigin: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin>;
	}

	#[pallet::storage]
	#[pallet::getter(fn get_ngo_waiting_list)]
	pub type NgoWaitingList<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, NgoInfo<T::Hash>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_ngo_active_list)]
	pub type NgoActiveList<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, NgoInfo<T::Hash>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_seller_waiting_list)]
	pub type SellerWaitingList<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, SellerInfo<T::Hash>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_seller_active_list)]
	pub type SellerActiveList<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, SellerInfo<T::Hash>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New Ngo Applied
		/// parameters. [who]
		NewNgoApplied { who: T::AccountId },
		/// New Ngo Added
		/// parameters. [who]
		NewNgoAdded { applicant: T::AccountId },
		// New Seller Applied
		/// parameters. [who]
		NewSellerApplied { who: T::AccountId },
		/// New Seller Added
		/// parameters. [who]
		NewSellerAdded { applicant: T::AccountId },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Already part Of Waiting List
		AlreadyPartOfWaitingList,
		/// Identity Not Found
		IdentityNotFound,
		/// Not Part Of Waiting List
		NotPartOfWaitingList,
		/// Already Part Of ActiveList
		AlreadyPartOfActiveList,
		/// Applicant Not Found
		ApplicantNotFound,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Apply as NGO
		///
		/// # Parameters
		/// * `ngo_info`: Ngo info.
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn apply_as_ngo(origin: OriginFor<T>, ngo_info: NgoInfo<T::Hash>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(
				pallet_identity::Pallet::<T>::has_identity(&who, 3),
				Error::<T>::IdentityNotFound
			);
			Self::do_apply_as_ngo(&who, ngo_info)?;
			Self::deposit_event(Event::NewNgoApplied { who });
			Ok(())
		}

		/// Approve Ngo
		/// Only Half of General Council or Root can call it.
		///
		/// # Parameters
		/// * `applicant`: Applicant.
		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn approve_ngo(origin: OriginFor<T>, applicant: T::AccountId) -> DispatchResult {
			T::GovernanceOrigin::ensure_origin(origin)?;
			Self::do_approve_ngo(&applicant)?;
			Self::deposit_event(Event::NewNgoAdded { applicant });
			// Emit Event
			Ok(())
		}

		/// Apply as Seller
		///
		/// # Parameters
		/// * `seller_info`: Seller info.
		#[pallet::call_index(2)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn apply_as_seller(
			origin: OriginFor<T>,
			seller_info: SellerInfo<T::Hash>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(
				pallet_identity::Pallet::<T>::has_identity(&who, 3),
				Error::<T>::IdentityNotFound
			);
			Self::do_apply_as_seller(&who, seller_info)?;
			Self::deposit_event(Event::NewSellerApplied { who });
			Ok(())
		}

		/// Approve Seller
		/// Only Half of General Council or Root can call it.
		///
		/// # Parameters
		/// * `applicant`: Applicant.
		#[pallet::call_index(3)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn approve_seller(origin: OriginFor<T>, applicant: T::AccountId) -> DispatchResult {
			T::GovernanceOrigin::ensure_origin(origin)?;
			Self::do_approve_seller(&applicant)?;
			Self::deposit_event(Event::NewSellerAdded { applicant });
			// Emit Event
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn do_apply_as_ngo(applicant: &T::AccountId, ngo_info: NgoInfo<T::Hash>) -> DispatchResult {
			ensure!(
				!<NgoWaitingList<T>>::contains_key(applicant),
				Error::<T>::AlreadyPartOfWaitingList
			);
			ensure!(
				!<NgoActiveList<T>>::contains_key(applicant),
				Error::<T>::AlreadyPartOfActiveList
			);
			<T as Config>::Currency::reserve(
				applicant,
				T::NgoStakingAmount::get().saturated_into(),
			)?;
			<NgoWaitingList<T>>::insert(applicant, ngo_info);
			Ok(())
		}

		fn do_approve_ngo(applicant: &T::AccountId) -> DispatchResult {
			ensure!(<NgoWaitingList<T>>::contains_key(applicant), Error::<T>::NotPartOfWaitingList);
			ensure!(
				!<NgoActiveList<T>>::contains_key(applicant),
				Error::<T>::AlreadyPartOfActiveList
			);
			let ngo_info = <NgoWaitingList<T>>::get(applicant);
			<NgoWaitingList<T>>::remove(applicant);
			<NgoActiveList<T>>::insert(applicant, ngo_info);
			Ok(())
		}

		fn do_apply_as_seller(
			applicant: &T::AccountId,
			seller_info: SellerInfo<T::Hash>,
		) -> DispatchResult {
			ensure!(
				!<SellerWaitingList<T>>::contains_key(applicant),
				Error::<T>::AlreadyPartOfWaitingList
			);
			ensure!(
				!<SellerActiveList<T>>::contains_key(applicant),
				Error::<T>::AlreadyPartOfActiveList
			);
			<T as Config>::Currency::reserve(
				applicant,
				T::SellerStakingAmount::get().saturated_into(),
			)?;
			<SellerWaitingList<T>>::insert(applicant, seller_info);
			Ok(())
		}

		fn do_approve_seller(applicant: &T::AccountId) -> DispatchResult {
			ensure!(
				<SellerWaitingList<T>>::contains_key(applicant),
				Error::<T>::NotPartOfWaitingList
			);
			ensure!(
				!<SellerActiveList<T>>::contains_key(applicant),
				Error::<T>::AlreadyPartOfActiveList
			);
			let seller_info = <SellerWaitingList<T>>::get(applicant);
			<SellerWaitingList<T>>::remove(applicant);
			<SellerActiveList<T>>::insert(applicant, seller_info);
			Ok(())
		}

		pub fn is_active_ngo(recipient: &T::AccountId) -> bool {
			<NgoActiveList<T>>::contains_key(recipient)
		}

		pub fn add_ngo_to_active_list(recipient: &T::AccountId, ngo_info: NgoInfo<T::Hash>) {
			<NgoActiveList<T>>::insert(recipient, ngo_info);
		}

		pub fn add_seller_to_active_list(
			recipient: &T::AccountId,
			seller_info: SellerInfo<T::Hash>,
		) {
			<SellerActiveList<T>>::insert(recipient, seller_info);
		}

		pub fn is_active_seller(seller: &T::AccountId) -> bool {
			<SellerActiveList<T>>::contains_key(seller)
		}
	}
}
