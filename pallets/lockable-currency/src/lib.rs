#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet{
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_support::traits::{Currency, LockIdentifier, LockableCurrency, WithdrawReasons};
	use frame_system::ensure_signed;
    use frame_system::pallet_prelude::OriginFor;

    const EXAMPLE_ID: LockIdentifier = *b"example ";
    type BalanceOf<T> = <<T as Config>::StakeCurrency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config{
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type StakeCurrency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Balance was locked successfully.
		Locked(T::AccountId, BalanceOf<T>),
		/// Lock was extended successfully.
		Unlocked(T::AccountId),
		/// Balance was unlocked successfully.
		ExtendedLock(T::AccountId, BalanceOf<T>),
    }

    #[pallet::error]
    pub enum Error<T> {}

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn lock_capital(
            origin: OriginFor<T>,
            #[pallet::compact] amount: BalanceOf<T>
        ) -> DispatchResultWithPostInfo {
            let user = ensure_signed(origin)?;

            T::StakeCurrency::set_lock(EXAMPLE_ID, &user, amount, WithdrawReasons::all());
            Self::deposit_event(Event::Locked(user, amount));
            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn extend_lock(
            origin: OriginFor<T>,
            #[pallet::compact] amount: BalanceOf<T>
        ) -> DispatchResultWithPostInfo {
            let user = ensure_signed(origin)?;

            T::StakeCurrency::extend_lock(EXAMPLE_ID, &user, amount, WithdrawReasons::all());
            Self::deposit_event(Event::ExtendedLock(user, amount));
            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn unlock_all(
            origin: OriginFor<T>
        ) -> DispatchResultWithPostInfo {
            let user = ensure_signed(origin)?;

            T::StakeCurrency::remove_lock(EXAMPLE_ID, &user);
            Self::deposit_event(Event::Unlocked(user));
            Ok(().into())
        }
    }
}