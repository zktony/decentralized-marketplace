#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::*,
		sp_runtime::{
			traits::Hash
		},
		traits::{
			fungibles::{Create, Inspect, Mutate, Transfer},
			Currency, ExistenceRequirement,
		},
		transactional, PalletId,
	};
	use frame_system::pallet_prelude::*;
	use participant_handler::Category;

	#[derive(Encode, Decode, Clone, Copy, Debug, MaxEncodedLen, TypeInfo)]
	pub struct ProductInfo<AccountId, Hash> {
		pub category: Category,
		pub price: u128,
		pub status: Status,
		pub owner: AccountId,
		pub cid: Hash,
	}

	impl<AccountId, Hash> ProductInfo<AccountId, Hash> {
		pub fn new(category: Category, price: u128, owner: AccountId, cid: Hash) -> Self {
			Self { category, price, status: Status::OpenForSell, owner, cid }
		}

		pub fn update_info(&mut self, buyer: AccountId) {
			self.status = Status::Sold;
			self.owner = buyer;
		}
	}

	#[derive(Encode, Decode, Clone, Copy, Debug, MaxEncodedLen, TypeInfo)]
	pub enum Status {
		Sold,
		OpenForSell,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config:
		frame_system::Config + participant_handler::Config + donation_handler::Config
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Token Handler
		type TokenHandler: Create<<Self as frame_system::Config>::AccountId>
			+ Mutate<<Self as frame_system::Config>::AccountId, Balance = u128, AssetId = u128>
			+ Inspect<<Self as frame_system::Config>::AccountId>
			+ Transfer<<Self as frame_system::Config>::AccountId>;
		/// Native Currency Handler
		type Currency: Currency<Self::AccountId>;
		/// Donation Handler Pallet Id
		#[pallet::constant]
		type DonationPalletId: Get<PalletId>;
	}

	/// Product List
	#[pallet::storage]
	#[pallet::getter(fn get_product_info)]
	pub(super) type ProductMap<T: Config> = StorageMap<
		_,
		frame_support::Blake2_128Concat,
		T::Hash,
		ProductInfo<T::AccountId, T::Hash>,
		OptionQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Product Listed
		/// parameters. [donor, recipient, amount, category]
		ProductListed { seller: T::AccountId, category: Category },
		/// Product Bought
		/// parameters. [buyer, pid]
		ProductBought { buyer: T::AccountId, pid: T::Hash },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Recipient not valid
		RecipientNotValid,
		/// Caller Not Valid
		CallerNotValid,
		/// Seller Not Valid
		SellerNotValid,
		/// Buyer Not Valid
		BuyerNotValid,
		/// Product Not Found
		ProductNotFound,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// List Product
		/// Only Approved Seller can call it.
		///
		/// # Parameters
		/// * `category`: Category to which product belongs.
		/// * `price`: Price of the product.
		/// * `cid`: Content Id of Product on IPFS.
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn list_product(
			origin: OriginFor<T>,
			category: Category,
			price: u128,
			cid: T::Hash,
		) -> DispatchResult {
			let seller = ensure_signed(origin)?;
			Self::do_list_product(&seller, category, price, cid)?;
			Self::deposit_event(Event::ProductListed { seller, category });
			Ok(())
		}

		/// Buy Product
		/// Only Ngo can buy it using Semi Fungible Token.
		///
		/// # Parameters
		/// * `pid`: Product Id.
		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn buy(origin: OriginFor<T>, pid: T::Hash) -> DispatchResult {
			let buyer = ensure_signed(origin)?;
			Self::do_buy(&buyer, pid)?;
			Self::deposit_event(Event::ProductBought { buyer, pid });
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		#[transactional]
		pub fn do_list_product(
			seller: &T::AccountId,
			category: Category,
			price: u128,
			cid: T::Hash,
		) -> DispatchResult {
			ensure!(
				participant_handler::pallet::Pallet::<T>::is_active_seller(seller),
				Error::<T>::SellerNotValid
			);
			let product = ProductInfo::new(category, price, seller.clone(), cid);
			//TODO: Check if seller can list product in given category
			let pid: T::Hash = T::Hashing::hash_of(&product);
			<ProductMap<T>>::insert(pid, product);
			Ok(())
		}

		#[transactional]
		pub fn do_buy(buyer: &T::AccountId, pid: T::Hash) -> DispatchResult {
			ensure!(
				participant_handler::pallet::Pallet::<T>::is_active_ngo(buyer),
				Error::<T>::SellerNotValid
			);
			<ProductMap<T>>::try_mutate(pid, |product| {
				if let Some(product) = product {
					donation_handler::Pallet::<T>::do_transfer_token(
						buyer,
						&product.owner,
						&product.category,
						product.price,
					)?;
					product.update_info(buyer.clone());
					Ok(())
				} else {
					Err(Error::<T>::ProductNotFound.into())
				}
			})
		}
	}
}
// TODO
// Remove hard coupling between donation-handler and participant-handler
