#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_template::Config{
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		SomethingValue(u32),
		UpdateSomething(u32)
	}

	#[pallet::error]
	pub enum Error<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(100)]
		pub fn update_something(origin: OriginFor<T>, num: u32) -> DispatchResult {
			let something = pallet_template::Pallet::<T>::something()?;
			Self::deposit_event(Event::SomethingValue(something));

			pallet_template::Pallet::<T>::do_something(origin, num)?;
			let something_new = pallet_template::Pallet::<T>::something()?;
			Self::deposit_event(Event::UpdateSomething(something_new));
			Ok(())
		}
	}
}
