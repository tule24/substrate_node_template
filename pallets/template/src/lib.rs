#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use frame_support::sp_runtime::traits::Printable;
	use frame_support::sp_std::if_std;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	#[pallet::storage]
	pub type Number<T:Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery, >;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
		SomethingDeleted(T::AccountId),
		Value(u32)
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig{
		pub genesis_value: u32,
	}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self{
			Self{
				genesis_value: 0u32
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			<Something<T>>::put(self.genesis_value)
		}
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			// OriginFor l?? 1 alias type c???a RuntimeOrigin
			// M?? RuntimeOrigin l?? 1 type trong Config ???????c s??? d???ng ????? dispatch call

			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// N???u sign r???i th?? tr??? v??? accountId c???a ng?????i call
			// https://docs.substrate.io/main-docs/build/origins/
			let who = ensure_signed(origin)?; 

			// Update storage.
			<Something<T>>::put(something);
			// Something::<T>::put(something);
			// Something::<T>::get() == <Something<T>>::get() == Self::something();\
			"My something is stored".print();
			// Emit an event.
			Self::deposit_event(Event::SomethingStored(something, who));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn put_number(origin: OriginFor<T>, number: u32) -> DispatchResult{
			let who = ensure_signed(origin)?;
			<Number<T>>::insert(who.clone(), number);
			if_std! {
				println!("Hello number");
				println!("Number put is: {:#?}", number);
				println!("The caller is: {:#?}", who);
			}
			Self::deposit_event(Event::SomethingStored(number, who));
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn get_number(origin: OriginFor<T>) -> DispatchResult{
			let who = ensure_signed(origin)?;
			let num = <Number<T>>::get(who.clone());
			Self::deposit_event(Event::Value(num));
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn delete_number(origin: OriginFor<T>) -> DispatchResult{
			let who = ensure_signed(origin)?;
			<Number<T>>::remove(who.clone());
			Self::deposit_event(Event::SomethingDeleted(who));
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match <Something<T>>::get() {
				// Return an error if the value has not been set.
				None => return Err(Error::<T>::NoneValue.into()),
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					<Something<T>>::put(new);
					Ok(())
				},
			}
			// let x = <Something<T>>::get()?;
			// // let x = x.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
			// <Something<T>>::put(x);
			// Ok(())
		}
	}
}

pub trait DoSome{
	fn increase_value(value: u32) -> u32;
}

impl<T: Config> DoSome for Pallet<T>{
	fn increase_value(value: u32) -> u32 {
		let something = <Something<T>>::get().unwrap();
		something * value
	}
}
