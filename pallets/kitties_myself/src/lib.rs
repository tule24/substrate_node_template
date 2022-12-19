#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        pallet_prelude::*,
        traits::{Time, Randomness, Currency},
        sp_runtime::traits::{Hash, Zero},
        BoundedVec
    };
    use frame_system::pallet_prelude::*;
    use scale_info::TypeInfo;
    use sp_runtime::ArithmeticError;

    // Khai báo 1 struct pallet placeholder để có thể sử dụng trong runtime
    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    // Định nghĩa các generic type, constant mà pallet sử dụng. Những type ở đây sẽ được impl bên Runtime
    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type Time: Time;
        type KittyDnaRandom: Randomness<Self::Hash, Self::BlockNumber>;
        type Currency: Currency<Self::AccountId>;
        #[pallet::constant]
        type MaxKittiesOwned: Get<u32>;
    }

    // define moment type
    type MomentOf<T> = <<T as Config>::Time as Time>::Moment;
    type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum Gender {
        Female,
        Male
    }

    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct Kitty<T: Config> {
        pub dna: T::Hash,
        pub owner: T::AccountId,
        pub price: BalanceOf<T>,
        pub gender: Gender,
        pub created_date: MomentOf<T>
    }

    #[pallet::storage]
    #[pallet::getter(fn kitties_total)]
    // Keep track of the number of Kitties existence
    pub type KittiesTotal<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn kitties)]
    // Mapping kitty_dna => Kitty
    pub type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, Kitty<T>>;

    #[pallet::storage]
    #[pallet::getter(fn kitties_owned)]
    // Mapping kitty_owner => Vec<kitty_dna>
    pub type KittiesOwned<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BoundedVec<T::Hash, T::MaxKittiesOwned>, ValueQuery,>;

    // Định nghĩa các event để emit khi các action thành công
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        // Emit when kitty minted
        Minted {owner: T::AccountId, kitty_dna: T::Hash},
        // Emit when transfer kitty
        Transferred {from: T::AccountId, to: T::AccountId, kitty_dna: T::Hash}
    }

    // Định nghĩa các error để emit khi lỗi xảy ra
    #[pallet::error]
    pub enum Error<T>{
        // Minted kitty but kitty_dna is exists
        KittyExisted,
        // Minted kitty but kitty_dna is exists
        KittyNotExisted,
        // Transfer but not owner
        NotOwner,
        // Transfer to myself
        NotTransferToSelf,
        // limit kitty owned
        TooManyOwned
    }

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config>{
        pub kitties_myself: Vec<T::AccountId>
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                kitties_myself: vec![]
            }
        }
    }

    #[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self){
			for (index, account) in self.kitties_myself.iter().enumerate(){
                let price: BalanceOf<T> = Zero::zero();
                let moment: MomentOf<T> = Zero::zero();
                let dna = T::Hashing::hash(&[index as u8]);
                let gender = if index%2 == 0 {Gender::Male} else {Gender::Female};
				assert!(<Pallet<T>>::create_kitty(account, price, dna, gender, moment).is_ok());
			}
		}
	}
    // Định nghĩa các logic cần thực thi trong context nhất định, ex: on_initialize, on_finalize
    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    // Định nghĩa các fn có thể gọi từ bên ngoài vào run time (extrinsic)
    #[pallet::call]
    impl<T: Config> Pallet<T>{
        #[pallet::weight(100)]
        pub fn mint_kitty(origin: OriginFor<T>, price: BalanceOf<T>) -> DispatchResult {
            // Make sure the caller is from a signed origin
            let sender = ensure_signed(origin)?;

            // Determine gender depend on dna.len()
            let (dna, gender) = Self::gen_dna_gender();
            let created_date = T::Time::now();

            Self::create_kitty(&sender, price, dna, gender, created_date)
        }

        #[pallet::weight(100)]
        pub fn transfer(origin: OriginFor<T>, to: T::AccountId, kitty_dna: T::Hash) -> DispatchResult {
            // Make sure the caller is from a signed origin
            let sender = ensure_signed(origin)?;

            let kitty = Self::kitties(&kitty_dna);
            // Make sure kitty exists
            ensure!(kitty != None, <Error<T>>::KittyNotExisted);

            // Make sure sender is owner
            let mut kitty = kitty.unwrap();
            ensure!(kitty.owner == sender, <Error<T>>::NotOwner);
            ensure!(kitty.owner != to, <Error<T>>::NotTransferToSelf);

            // remove kitty_dna from account
            let mut from_owned = Self::kitties_owned(&sender);
            if let Some(index) = from_owned.iter().position(|dna| kitty_dna == *dna) {
                from_owned.swap_remove(index);
            } else {
                return Err(Error::<T>::NotOwner.into());
            };

            <KittiesOwned<T>>::try_mutate(&to, |list_kitty| {
                list_kitty.try_push(kitty_dna.clone())
            }).map_err(|_| <Error<T>>::TooManyOwned)?;

             // change owner of kitty
            kitty.owner = to.clone();
            <Kitties<T>>::insert(kitty_dna.clone(), kitty);
            <KittiesOwned<T>>::insert(sender.clone(), from_owned);

            // emit event
            Self::deposit_event(Event::Transferred{from: sender, to, kitty_dna});
            Ok(())
        }
    }

    // Helper func là các hàm hỗ trợ xử lý các logic cũng như tránh lặp code, giúp bảo mật code
    impl<T: Config> Pallet<T>{
        fn gen_dna_gender() -> (T::Hash, Gender) {
            // Get random kitty_dna make sure it not exists
            let mut random = T::KittyDnaRandom::random(&b"dna"[..]).0;
            while Self::kitties(&random) != None {
                random = T::KittyDnaRandom::random(&b"dna"[..]).0;
            };
            let gender = if random.encode()[0] % 2 == 0 {Gender::Male} else {Gender::Female};
            
            return (random, gender)
        }

        fn create_kitty(account: &T::AccountId, price: BalanceOf<T>, dna: T::Hash, gender: Gender, created_date: MomentOf<T>) -> DispatchResult {
            // Create kitty
            let kitty = Kitty::<T> {
                dna: dna.clone(),
                owner: account.clone(),
                price,
                gender,
                created_date
            };

            // Check kitty total overflow, if not +1
            let kitty_total = Self::kitties_total();
            let new_kitty_total = kitty_total.checked_add(1).ok_or(ArithmeticError::Overflow)?;

            // Update kittiesowned for owner
            <KittiesOwned<T>>::try_mutate(&account, |list_kitty| {
                list_kitty.try_push(dna.clone())
            }).map_err(|_| <Error<T>>::TooManyOwned)?;

            // update kitty total
            <KittiesTotal<T>>::put(new_kitty_total);
            //update kitties
            <Kitties<T>>::insert(dna.clone(), kitty);

            // emit event
            Self::deposit_event(Event::Minted{owner: account.clone(), kitty_dna: dna});

            Ok(())
        }
    }
}