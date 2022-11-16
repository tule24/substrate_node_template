#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet{
    use frame_support::Blake2_128;
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::ensure_signed;
    use frame_system::pallet_prelude::OriginFor;

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
        MintedNewSupply(T::AccountId),
        Transferred(T::AccountId, T::AccountId, u64)
    }

    #[pallet::storage]
    #[pallet::getter(fn get_balance)]
    pub(super) type BalanceToAccount<T:Config> = StorageMap<_, Blake2_128, T::AccountId, u64, ValueQuery>;

    #[pallet::error]
    pub enum Error<T> {
        InsufficientFunds,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn mint(
            origin: OriginFor<T>,
            amount: u64
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            
            // Update storage
            <BalanceToAccount<T>>::insert(&sender, amount);

            //Emit an event
            Self::deposit_event(Event::MintedNewSupply(sender));

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn transfer(
            origin: OriginFor<T>,
            to: T::AccountId,
            amount: u64
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            let sender_balance = Self::get_balance(&sender);
            let receiver_balance = Self::get_balance(&to);
           
            // Calculate new balance
            let update_from_balance = sender_balance.checked_sub(amount).ok_or(Error::<T>::InsufficientFunds)?;
            let update_to_balance = receiver_balance.checked_add(amount).expect("Entire supply fits in u64, qed");

            <BalanceToAccount<T>>::insert(&sender, update_from_balance);
            <BalanceToAccount<T>>::insert(&to, update_to_balance);

            Self::deposit_event(Event::Transferred(sender, to, amount));

            Ok(().into())
        }
    }
}
