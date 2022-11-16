#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::dispatch::{DispatchErrorWithPostInfo, PostDispatchInfo};
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::Zero;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		#[pallet::constant]
		type MaxAddend: Get<u32>;
		type ClearFrequency: Get<Self::BlockNumber>;
	}

	#[pallet::storage]
	#[pallet::getter(fn single_value)]
	pub(super) type SingleValue<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Added(u32, u32, u32),
		Cleared(u32)
	}

	#[pallet::error]
	pub enum Error<T> {
		Overflow,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		fn on_finalize(n: T::BlockNumber) {
			if (n % T::ClearFrequency::get()).is_zero() {
				let c_val = SingleValue::<T>::get();
				SingleValue::<T>::put(0u32);
				Self::deposit_event(Event::Cleared(c_val));
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1_000)]
		pub fn add_value(origin: OriginFor<T>, val_to_add: u32) -> DispatchResultWithPostInfo {
			let _ = ensure_signed(origin)?;
			ensure!(
				val_to_add <= T::MaxAddend::get(),
				"Value must be <= maximum add amount constant"
			);

			let c_val = SingleValue::<T>::get();
			// let res = c_val.checked_add(val_to_add).ok_or(Error::<T>::Overflow)?;
			let res = match Self::_adder(c_val, val_to_add) {
				Ok(res) => res,
				Err(err) => {
					return Err(DispatchErrorWithPostInfo {
						post_info: PostDispatchInfo::from(()),
						error: DispatchError::Other(err),
					})
				},
			};
			<SingleValue<T>>::put(res);
			Self::deposit_event(Event::Added(c_val, val_to_add, res));
			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn _adder(num1: u32, num2: u32) -> Result<u32, &'static str> {
		num1.checked_add(num2).ok_or("Overflow when adding")
	}
}
