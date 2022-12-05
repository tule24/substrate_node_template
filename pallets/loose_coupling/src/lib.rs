#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use pallet_template::DoSome;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config{
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type IncreaseValue: DoSome;
	}

	#[pallet::storage]
	#[pallet::getter(fn something)]
	pub type Something<T> = StorageValue<_, u32>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Increase(u32)
	}

	#[pallet::error]
	pub enum Error<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(100)]
		pub fn increase_value(origin: OriginFor<T>, something: u32) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let res = T::IncreaseValue::increase_value(something);
			Self::deposit_event(Event::Increase(res));

			Ok(())
		}
	}
}
