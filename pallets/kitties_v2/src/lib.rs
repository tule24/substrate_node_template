#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::*,
		traits::{tokens::ExistenceRequirement, Currency, Randomness}, Twox64Concat, BoundedVec,
	};
	use frame_system::pallet_prelude::*;
	use scale_info::TypeInfo;
	use sp_io::hashing::blake2_128;
	use sp_runtime::ArithmeticError;

	#[cfg(feature = "std")]
	use frame_support::serde::{Deserialize, Serialize};

	// Handles our pallet's currency abstraction
	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	// Struct for holding kitty information
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct Kitty<T: Config> {
		pub dna: [u8; 16],               // using 16 bytes to represent a kitty DNA == Hash
		pub price: Option<BalanceOf<T>>, // None assume not for sale
		pub gender: Gender,
		pub owner: T::AccountId,
	}

	// Set Gender type in kitty struct
	#[derive(Clone, Encode, Decode, PartialEq, Copy, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub enum Gender {
		Female,
		Male,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	//  Configure the pallet by specifying the parameters and types on which it depends
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		// The Currency handler for the kitties pallet
		type Currency: Currency<Self::AccountId>;

		// The maximum amount of kitties a single account can own
		#[pallet::constant]
		type MaxKittiesOwned: Get<u32>;

		// The type of Randomness we want to specify for this pallet
		type KittyRandomness: Randomness<Self::Hash, Self::BlockNumber>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new kitty was successfully created.
		Created { kitty: [u8; 16], owner: T::AccountId },
		/// The price of a kitty was successfully set.
		PriceSet { kitty: [u8; 16], price: Option<BalanceOf<T>> },
		/// A kitty was successfully transferred.
		Transferred { from: T::AccountId, to: T::AccountId, kitty: [u8; 16] },
		/// A kitty was successfully sold.
		Sold { seller: T::AccountId, buyer: T::AccountId, kitty: [u8; 16], price: BalanceOf<T> },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// An account may only own `MaxKittiesOwned` kitties.
		TooManyOwned,
		/// Trying to transfer or buy a kitty from oneself.
		TransferToSelf,
		/// This kitty already exists!
		DuplicateKitty,
		/// This kitty does not exist!
		NoKitty,
		/// You are not the owner of this kitty.
		NotOwner,
		/// This kitty is not for sale.
		NotForSale,
		/// Ensures that the buying price is greater than the asking price.
		BidPriceTooLow,
		/// You need to have two cats with different gender to breed.
		CantBreed,
	}

	// Keeps track of the numer of kitties in existence
	#[pallet::storage]
	pub type CountForKitties<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	// Maps the kitty struct to the kitty DNA
	pub type Kitties<T: Config> = StorageMap<_, Twox64Concat, [u8; 16], Kitty<T>>;

	// Track the kitties owned by each account
	#[pallet::storage]
	pub type KittiesOwned<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, BoundedVec<[u8; 16], T::MaxKittiesOwned>, ValueQuery>;

	// Our pallet's genesis configuration
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub kitties_v2: Vec<(T::AccountId, [u8; 16], Gender)>
	}

	// Required to implement default for GenesisConfig
	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> GenesisConfig<T> {
			GenesisConfig {kitties_v2: vec![]}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self){
			// When building a kitty from genesis config, we require the DNA and Gender to be
			// supplied
			for (account, dna, gender) in &self.kitties_v2{
				assert!(Pallet::<T>::mint(account, *dna, *gender).is_ok());
			}
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// Create a new unique kitty
		#[pallet::weight(100)]
		pub fn create_kitty(origin: OriginFor<T>) -> DispatchResult {
			// Make sure the caller is from a signed origin
			let sender = ensure_signed(origin)?;

			// Generate unique DNA and gener using a helper function
			let (dna, gender) = Self::gen_dna_gender();

			// Mint new kitty to storage by calling helper function
			Self::mint(&sender, dna, gender)?;
			Ok(())
		}

		// Set price for kitty
		#[pallet::weight(100)]
		pub fn set_price(origin: OriginFor<T>, kitty_id: [u8; 16], new_price: Option<BalanceOf<T>>) -> DispatchResult{
			// Make sure the caller is from a signed origin
			let sender = ensure_signed(origin)?;

			// Ensure the kitty exists and it called by the kitty owner
			let mut kitty = Kitties::<T>::get(&kitty_id).ok_or(Error::<T>::NoKitty)?;
			ensure!(kitty.owner == sender, Error::<T>::NotOwner);

			//set price for kitty
			kitty.price = new_price;
			Kitties::<T>::insert(&kitty_id, kitty);

			// emit event
			Self::deposit_event(Event::PriceSet { kitty: kitty_id, price: new_price });
			Ok(())
		}

		// Directly transfer a kitty to another recipient.
		// Any account that holds a kitty can send it to another Account. This will reset the asking price of the kitty, marking it not for sale.
		#[pallet::weight(100)]
		pub fn transfer(origin: OriginFor<T>, to: T::AccountId, kitty_id: [u8; 16]) -> DispatchResult {
			// Make sure the caller is from a signed origin
			let from = ensure_signed(origin)?;
			let kitty = Kitties::<T>::get(&kitty_id).ok_or(Error::<T>::NoKitty)?;
			ensure!(kitty.owner == from, Error::<T>::NotOwner);
			Self::do_transfer(kitty_id, to, None)?;
			Ok(())
		}

		/// Buy a kitty for sale. The `limit_price` parameter is set as a safeguard against the 
		/// possibility that the seller front-runs the transaction by setting a high price. A front-end
		/// should assume that this value is always equal to the actual price of the kitty. The buyer 
		/// will always be charged the actual price of the kitty.
		///
		/// If successful, this dispatchable will reset the price of the kitty to `None`, making 
		/// it no longer for sale and handle the balance and kitty transfer between the buyer and seller.
		#[pallet::weight(100)]
		pub fn buy_kitty(origin: OriginFor<T>, kitty_id: [u8; 16], limit_price: BalanceOf<T>) -> DispatchResult {
			// Make sure the caller is from a signed origin
			let buyer = ensure_signed(origin)?;
			//Transfer the kitty from seller to buyer as a sale
			Self::do_transfer(kitty_id, buyer, Some(limit_price))?;
			Ok(())
		}

		/// Breed a kitty.
		///
		/// Breed two kitties to give birth to a new kitty.
		#[pallet::weight(100)]
		pub fn breed_kitty(origin: OriginFor<T>, parent_1: [u8;16], parent_2: [u8;16]) -> DispatchResult {
			// Make sure the caller is from a signed origin
			let sender = ensure_signed(origin)?;

			// Get the kitties
			let maybe_mom = Kitties::<T>::get(&parent_1).ok_or(Error::<T>::NoKitty)?;
			let maybe_dad = Kitties::<T>::get(&parent_2).ok_or(Error::<T>::NoKitty)?;

			// Check both parents are owned by the caller of this function
			ensure!(maybe_mom.owner == sender, Error::<T>::NotOwner);
			ensure!(maybe_dad.owner == sender, Error::<T>::NotOwner);

			// Parents must be of opposite genders
			ensure!(maybe_mom.gender != maybe_dad.gender, Error::<T>::CantBreed);

			// Create new DNA from these parents
			let (new_dna, new_gender) = Self::breed_dna(&parent_1, &parent_2);

			// Mint new kitty
			Self::mint(&sender, new_dna, new_gender)?;
			Ok(())

		}
	}

	// helper function
	impl<T: Config> Pallet<T> {
		// gen and returns DNA & gender
		fn gen_dna_gender() -> ([u8;16], Gender) {
			let random = T::KittyRandomness::random(&b"dna&gender"[..]).0;

			// Create randomness payload. Multiple kitties can be generated in the same block,
			// retaining uniqueness
			let unique_payload = (
				random, 
				frame_system::Pallet::<T>::extrinsic_index().unwrap_or_default(),
				frame_system::Pallet::<T>::block_number()
			);

			// Turns into a byte array
			let encoded_payload = unique_payload.encode();
			let hash = blake2_128(&encoded_payload);

			// Generate Gender
			if hash[0] % 2 == 0 {
				// Males are identified by having an even leading byte
				(hash, Gender::Male)
			} else {
				// Females are identified by having an odd leading byte
				(hash, Gender::Female)
			}
		}

		// Picks from existing DNA
		fn mutate_dna_fragment(dna_fragment1: u8, dna_fragment2: u8, random_value: u8) -> u8 {
			// Given some random u8
			if random_value % 2 == 0 {
				// either return `dna_fragment1` if its an even value
				dna_fragment1
			} else {
				// or return `dna_fragment2` if its an odd value
				dna_fragment2
			}
		}

		// Generates a new kitty using existing kitties
		fn breed_dna(parent1: &[u8;16], parent2: &[u8;16]) -> ([u8;16], Gender) {
			// Call `gen_dna` to generate random kitty DNA
			// We don't know what Gender this kitty is yet
			let (mut new_dna, new_gender) = Self::gen_dna_gender();

			// randomly combine DNA using `mutate_dna_fragment`
			for i in 0..new_dna.len(){
				// At this point, `new_dna` is a randomly generated set of bytes, so we can
				// extract each of its bytes to act as a random value for `mutate_dna_fragment`
				new_dna[i] = Self::mutate_dna_fragment(parent1[i], parent2[i], new_dna[i])
			}

			// return new DNA and gender
			(new_dna, new_gender)
		}

		// Mint a kitty
		fn mint(owner: &T::AccountId, dna: [u8;16], gender: Gender) -> Result<(), DispatchError>{

			// Check if the kitty_dna does not already exist in our storage map
			ensure!(!Kitties::<T>::contains_key(&dna), Error::<T>::DuplicateKitty);

			// create a new object
			let kitty = Kitty::<T> {
				dna,
				price: None,
				gender,
				owner: owner.clone()
			};

			// Performs this operation first as it may fail
			let count = CountForKitties::<T>::get();
			let new_count = count.checked_add(1).ok_or(ArithmeticError::Overflow)?;

			// Append kitty to KittiesOwned
			KittiesOwned::<T>::try_mutate(&owner, |list_kitty| {
				list_kitty.try_push(kitty.dna)
			}).map_err(|_| Error::<T>::TooManyOwned)?;

			// Write new kitty to storage
			Kitties::<T>::insert(kitty.dna, kitty);
			CountForKitties::<T>::put(new_count);

			// Emit event
			Self::deposit_event(Event::Created{kitty: dna, owner: owner.clone()});

			Ok(())

		}

		// upgrade storage to transfer kitty
		fn do_transfer(kitty_id: [u8; 16], to: T::AccountId, maybe_limit_price: Option<BalanceOf<T>>) -> DispatchResult {
			// get the kitty
			let mut kitty = Kitties::<T>::get(&kitty_id).ok_or(Error::<T>::NoKitty)?;
			let from = kitty.owner;

			ensure!(from != to, Error::<T>::TransferToSelf);
			let mut from_owned = KittiesOwned::<T>::get(&from);

			// Remove kitty from list of owned kitties
			if let Some(index) = from_owned.iter().position(|&id| id == kitty_id) {
				from_owned.swap_remove(index);
			} else { 
				return Err(Error::<T>::NoKitty.into());
			}

			// Add kitty to the list owned kitties
			let mut to_owned = KittiesOwned::<T>::get(&to);
			to_owned.try_push(kitty_id).map_err(|_| Error::<T>::TooManyOwned)?;

			// Mutating state here via a balance transfer, so nothing is allowed to fail after this.
			// The buyer will always be charged the actual price. The limit_price parameter is just a 
			// protection so the seller isn't able to front-run the transaction.
			if let Some(limit_price) = maybe_limit_price {
				// Current kitty price if for sale
				if let Some(price) = kitty.price {
					ensure!(limit_price >= price, Error::<T>::BidPriceTooLow);
					// Transfer the amount from buyer to seller
					T::Currency::transfer(&to, &from, price, ExistenceRequirement::KeepAlive)?;
					// deposit sold event
					Self::deposit_event(Event::Sold { seller: from.clone(), buyer: to.clone(), kitty: kitty_id, price });
				} else {
					// Kitty price is set to `None` and is not for sale
					return Err(Error::<T>::NotForSale.into());
				}
			}

			// Transfer succeeded, update the kitty owner and reset the price to `None`.
			kitty.owner = to.clone();
			kitty.price = None;

			// Write updates to storage
			Kitties::<T>::insert(&kitty_id, kitty);
			KittiesOwned::<T>::insert(&to, to_owned);
			KittiesOwned::<T>::insert(&from, from_owned);

			Self::deposit_event(Event::Transferred { from, to, kitty: kitty_id });
			Ok(())
		}
	}
 }
