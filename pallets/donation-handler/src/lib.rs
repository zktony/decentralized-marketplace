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
		sp_runtime::{traits::AccountIdConversion, SaturatedConversion},
		traits::{
			fungibles::{Create, Inspect, Mutate, Transfer},
			Currency, ExistenceRequirement,
		},
		transactional, PalletId,
	};
	use frame_system::pallet_prelude::*;
	use participant_handler::Category;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + participant_handler::Config {
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

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Token Donated
		/// parameters. [donor, recipient, amount, category]
		TokenDonated {
			donor: T::AccountId,
			recipient: T::AccountId,
			amount: u128,
			category: Category,
		},
		/// Tokens Claimed
		/// parameters. [seller, category, amount]
		TokensClaimed { seller: T::AccountId, category: Category, amount: u128 },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Recipient not valid
		RecipientNotValid,
		/// Caller Not Valid
		CallerNotValid,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Donor can donate SFT (Semi Fungible Token) to Approved NGO.
		///
		/// # Parameters
		/// * `recipient`: Recipient who will get SFT.
		/// * `category`: Category.
		/// * `amount`: Donation Amount.
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn donate(
			origin: OriginFor<T>,
			recipient: T::AccountId,
			category: Category,
			amount: u128,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_donate(&who, &recipient, amount, &category)?;
			// Emit an event.
			Self::deposit_event(Event::TokenDonated { donor: who, recipient, amount, category });
			Ok(())
		}

		/// Seller can claim Native Token by burning equal amount of SFT.
		///
		/// # Parameters
		/// * `category`: variant of SFT
		/// * `amount`: Expected Amount
		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn claim_token(
			origin: OriginFor<T>,
			category: Category,
			amount: u128,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_claim(&who, &category, amount)?;
			Self::deposit_event(Event::TokensClaimed { seller: who, category, amount });
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		#[transactional]
		pub fn do_donate(
			donor: &T::AccountId,
			recipient: &T::AccountId,
			amount: u128,
			category: &Category,
		) -> DispatchResult {
			ensure!(
				participant_handler::pallet::Pallet::<T>::is_active_ngo(recipient),
				Error::<T>::RecipientNotValid
			);
			// TODO: Also check if NGO can accept in given Category
			<T as Config>::Currency::transfer(
				donor,
				&Self::get_pallet_account(),
				amount.saturated_into(),
				ExistenceRequirement::KeepAlive,
			)?;
			T::TokenHandler::mint_into(
				category.get_id() as u128,
				recipient,
				amount.saturated_into(),
			)?;
			Ok(())
		}

		#[transactional]
		pub fn do_claim(
			recipient: &T::AccountId,
			category: &Category,
			amount: u128,
		) -> DispatchResult {
			ensure!(
				participant_handler::pallet::Pallet::<T>::is_active_seller(recipient),
				Error::<T>::CallerNotValid
			);
			T::TokenHandler::burn_from(
				category.get_id() as u128,
				recipient,
				amount.saturated_into(),
			)?;
			<T as Config>::Currency::transfer(
				&Self::get_pallet_account(),
				recipient,
				amount.saturated_into(),
				ExistenceRequirement::KeepAlive,
			)?;
			Ok(())
		}

		#[transactional]
		pub fn do_transfer_token(
			source: &T::AccountId,
			recipient: &T::AccountId,
			category: &Category,
			amount: u128,
		) -> DispatchResult {
			ensure!(
				participant_handler::pallet::Pallet::<T>::is_active_ngo(source),
				Error::<T>::RecipientNotValid
			);
			ensure!(
				participant_handler::pallet::Pallet::<T>::is_active_seller(recipient),
				Error::<T>::CallerNotValid
			);
			//TODO: Check if Seller can accept Category Token?
			T::TokenHandler::transfer(
				category.get_id() as u128,
				source,
				recipient,
				amount.saturated_into(),
				true,
			)?;
			Ok(())
		}

		fn get_pallet_account() -> T::AccountId {
			T::DonationPalletId::get().into_account_truncating()
		}
	}
}
// TODO
// Remove hard coupling between donation-handler and participant-handler
