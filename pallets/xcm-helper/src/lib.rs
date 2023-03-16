#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
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
		dispatch::DispatchResultWithPostInfo,
		pallet_prelude::*,
		sp_runtime::SaturatedConversion,
		sp_std,
		traits::{
			fungibles::{Create, Inspect, Mutate, Transfer},
			Currency, ExistenceRequirement, ReservableCurrency, WithdrawReasons,
		},
	};
	use frame_system::pallet_prelude::*;
	use xcm::{
		latest::{AssetId, Fungibility, Junction, Junctions, MultiAsset, MultiLocation},
		v2::prelude::XcmError,
	};
	use xcm_executor::{
		traits::{Convert as MoreConvert, TransactAsset, WeightTrader},
		Assets,
	};

	#[derive(Encode, Decode, Clone, TypeInfo, PartialEq, Debug)]
	pub enum AssetType {
		Fungible,
		NonFungible,
	}

	#[derive(Encode, Decode, Clone, TypeInfo, PartialEq, Debug)]
	pub struct ParachainAsset {
		pub location: MultiLocation,
		pub asset_type: AssetType,
	}

	impl ParachainAsset {
		pub fn convert_asset_id(asset_id: AssetId) -> Option<Self> {
			match asset_id {
				AssetId::Concrete(location) =>
					Some(Self { location, asset_type: AssetType::Fungible }),
				AssetId::Abstract(_) => None,
			}
		}
	}

	pub trait AssetIdConverter {
		/// Converts AssetId to MultiLocation
		fn convert_asset_id_to_location(asset_id: u128) -> Option<MultiLocation>;
		/// Converts Location to AssetId
		fn convert_location_to_asset_id(location: MultiLocation) -> Option<u128>;
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Multilocation to AccountId Convetor
		type AccountIdConvert: MoreConvert<MultiLocation, Self::AccountId>;
		/// Integrate Balances Pallet
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		/// Asset Manager
		type AssetManager: Create<<Self as frame_system::Config>::AccountId>
			+ Mutate<<Self as frame_system::Config>::AccountId, Balance = u128, AssetId = u128>
			+ Inspect<<Self as frame_system::Config>::AccountId>
			+ Transfer<<Self as frame_system::Config>::AccountId>;
		/// Parachain Id
		#[pallet::constant]
		type ParachainId: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	//Asset Mapping
	#[pallet::storage]
	#[pallet::getter(fn get_asset)]
	pub type AssetMapping<T: Config> =
		StorageMap<_, Blake2_128Concat, u128, ParachainAsset, OptionQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_ref_time(10_000) + T::DbWeight::get().writes(1))]
		pub fn create_asset(
			origin: OriginFor<T>,
			asset_info: AssetId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?; //FIXME: Ensure half council or root
			let asset_id = Self::convert_asset_id_into_local(asset_info.clone())
				.ok_or(Error::<T>::StorageOverflow)?; //FIXME: Change Error
			let parachain_asset =
				ParachainAsset::convert_asset_id(asset_info).ok_or(Error::<T>::StorageOverflow)?; //FIXME: Change Error
			<AssetMapping<T>>::insert(asset_id, parachain_asset);
			Self::deposit_event(Event::SomethingStored(1, who)); //TODO: Update the error
			Ok(().into())
		}
	}

	impl<T: Config> TransactAsset for Pallet<T> {
		fn deposit_asset(
			what: &MultiAsset,
			who: &MultiLocation,
		) -> sp_std::result::Result<(), XcmError> {
			let MultiAsset { id, fun } = what;
			let who =
				T::AccountIdConvert::convert_ref(who).map_err(|_| XcmError::FailedToDecode)?;
			let amount: u128 = Self::get_amount(fun).ok_or(XcmError::Trap(101))?;
			if Self::is_native_asset(id) {
				T::Currency::deposit_creating(&who, amount.saturated_into());
			} else {
				let asset_id =
					Self::convert_asset_id_into_local(what.id.clone()).ok_or(XcmError::Trap(22))?; //TODO: Verify error
				T::AssetManager::mint_into(asset_id, &who, amount.saturated_into())
					.map_err(|_| XcmError::Trap(24))?;
			}
			Ok(())
		}

		fn withdraw_asset(
			what: &MultiAsset,
			who: &MultiLocation,
		) -> sp_std::result::Result<Assets, XcmError> {
			let MultiAsset { id, fun } = what;
			let who =
				T::AccountIdConvert::convert_ref(who).map_err(|_| XcmError::FailedToDecode)?;
			let amount: u128 = Self::get_amount(fun).ok_or(XcmError::Trap(101))?;
			if Self::is_native_asset(id) {
				T::Currency::withdraw(
					&who,
					amount.saturated_into(),
					WithdrawReasons::all(),
					ExistenceRequirement::KeepAlive,
				)
				.map_err(|_| XcmError::Trap(21))?; //TODO: Check for withdraw reason and error
			} else {
				let asset_id =
					Self::convert_asset_id_into_local(what.id.clone()).ok_or(XcmError::Trap(22))?; //TODO: Verify error
				T::AssetManager::burn_from(asset_id, &who, amount.saturated_into())
					.map_err(|_| XcmError::Trap(24))?;
			}
			Ok(what.clone().into())
		}

		fn transfer_asset(
			asset: &MultiAsset,
			from: &MultiLocation,
			to: &MultiLocation,
		) -> sp_std::result::Result<Assets, XcmError> {
			let MultiAsset { id, fun } = asset;
			let from =
				T::AccountIdConvert::convert_ref(from).map_err(|_| XcmError::FailedToDecode)?;
			let to = T::AccountIdConvert::convert_ref(to).map_err(|_| XcmError::FailedToDecode)?;
			let amount: u128 = Self::get_amount(fun).ok_or(XcmError::Trap(101))?;
			if Self::is_native_asset(id) {
				T::Currency::transfer(
					&from,
					&to,
					amount.saturated_into(),
					ExistenceRequirement::KeepAlive,
				)
				.map_err(|_| XcmError::Trap(21))?;
			} else {
				let asset_id =
					Self::convert_asset_id_into_local(id.clone()).ok_or(XcmError::Trap(22))?;
				T::AssetManager::transfer(asset_id, &from, &to, amount, true)
					.map_err(|_| XcmError::Trap(23))?;
			}
			Ok(asset.clone().into())
		}
	}

	impl<T: Config> Pallet<T> {
		fn convert_asset_id_into_local(asset_id: AssetId) -> Option<u128> {
			match asset_id {
				AssetId::Concrete(location) => {
					let parachain_asset =
						ParachainAsset { location, asset_type: AssetType::Fungible };
					let derived_asset_id = parachain_asset.encode();
					let derived_asset_id_hash =
						&sp_io::hashing::keccak_256(derived_asset_id.as_ref())[0..16];
					let mut temp = [0u8; 16];
					temp.copy_from_slice(derived_asset_id_hash);
					Some(u128::from_le_bytes(temp))
				},
				AssetId::Abstract(_) => None,
			}
		}

		/// Checks if asset is native or not
		pub fn is_native_asset(asset: &AssetId) -> bool {
			let native_asset = MultiLocation {
				parents: 1,
				interior: Junctions::X1(Junction::Parachain(T::ParachainId::get().into())),
			};
			match asset {
				AssetId::Concrete(location) if location == &native_asset => true,
				_ => false,
			}
		}

		/// Converts XCM::Fungibility into u128
		pub fn get_amount(fun: &Fungibility) -> Option<u128> {
			if let Fungibility::Fungible(amount) = fun {
				return Some(*amount)
			} else {
				None
			}
		}
	}

	impl<T: Config> AssetIdConverter for Pallet<T> {
		fn convert_asset_id_to_location(asset_id: u128) -> Option<MultiLocation> {
			if let Some(parachain_asset) = <AssetMapping<T>>::get(asset_id) {
				Some(parachain_asset.location)
			} else {
				None
			}
		}

		fn convert_location_to_asset_id(location: MultiLocation) -> Option<u128> {
			let parachain_asset = ParachainAsset { location, asset_type: AssetType::Fungible };
			let derived_asset_id = parachain_asset.encode();
			let derived_asset_id_hash =
				&sp_io::hashing::keccak_256(derived_asset_id.as_ref())[0..16];
			let mut temp = [0u8; 16];
			temp.copy_from_slice(derived_asset_id_hash);
			Some(u128::from_le_bytes(temp))
		}
	}
}
