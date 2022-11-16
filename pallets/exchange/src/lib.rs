#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet{
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config{
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
    }

    #[pallet::error]
    pub enum Error<T> {
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
       
    }
}
