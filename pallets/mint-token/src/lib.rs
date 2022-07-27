#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>

pub use pallet::*;
use codec::{Encode, Decode};
// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;


use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use frame_support::sp_runtime::{
	RuntimeDebug, traits::{
		AtLeast32BitUnsigned, Zero, Saturating, CheckedAdd, CheckedSub
	},
};

#[derive(Clone, Encode, Decode,TypeInfo, Eq, PartialEq, RuntimeDebug, Default)]
pub struct MetaData<AccountId, Balance>{
	issuance: Balance,
	minter: AccountId,
	burner: AccountId
}

#[frame_support::pallet]
pub mod pallet {
	pub use super::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Balance: Member + Parameter + AtLeast32BitUnsigned + Default + Copy;

		type MinBalance: Get<Self::Balance>;

	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	//#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	#[pallet::storage]
	#[pallet::getter(fn get_balance)]
	pub(super) type BalanceToAccount<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, T::Balance, ValueQuery>;



	#[pallet::storage]
	#[pallet::getter(fn meta_data)]
	pub(super) type MetaDataStore<T:Config> = StorageValue<_, MetaData<T::AccountId, T::Balance>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn account)]
	pub(super) type Accounts<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, T::Balance, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config>{
		pub admin: T::AccountId
	}

	#[cfg(feature="std")]
	impl<T:Config> Default for GenesisConfig<T>{
		fn default()-> Self{
			Self{
				admin: Default::default()
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T>{
		fn build(&self){
			MetaDataStore::<T>::put(MetaData{
				issuance: Zero::zero(),
				minter: self.admin,
				burner: self.admin
			});
		}
	}

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),

		MintedNewSupply(T::AccountId),
		Transferred(T::AccountId, T::AccountId, T::Balance),

		Created(T::AccountId),
		Killed(T::AccountId),
		Minted(T::AccountId, T::Balance),
		Burned(T::AccountId, T::Balance),
		Transferd(T::AccountId, T::AccountId, T::Balance)

	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,

		BelowMinBalance,
		NoPermission,
		Overflow,
		Underflow,
		CannotBurnEmpty,
		InsufficientBalance
	}

	#[pallet::hooks]
	impl<T:Config> Hooks<BlockNumberFor<T>> for Pallet<T>{
		fn on_initialize(_n: T::BlockNumber)-> Weight{
			let mut meta = MetaDataStore::<T>::get();
			let value = 50u8.into();
			meta.issuance = meta.issuance.saturating_add(value);
			Accounts::<T>::mutate(meta.minter, |bal|{
				bal = bal.saturating_add(value);
			});
		}
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;

			// Update storage.
			<Something<T>>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored(something, who));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
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
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn mint(origin: OriginFor<T>,amount: T::Balance) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			//ensure!(Self::kitties(kitt));
			// update storage
			<BalanceToAccount<T>>::insert(&sender, amount);
			Self::deposit_event(Event::MintedNewSupply(sender));

			Ok(())

		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn transfer(origin: OriginFor<T>, to: T::AccountId, #[pallet::compact] amount: T::Balance) -> DispatchResult{
			let sender = ensure_signed(origin)?;
			let sender_balance = Self::get_balance(&sender);
			let receiver_balance = Self::get_balance(&to);

			// calculate new balance
			let update_sender = sender_balance.saturating_sub(amount);
			let update_to = receiver_balance.saturating_add(amount);

			// update both account storage
			<BalanceToAccount<T>>::insert(&sender, update_sender);
			<BalanceToAccount<T>>::insert(&to, update_to);

			// emit event
			Self::deposit_event(Event::Transferred(sender, to, amount));

			Ok(())
		}

		#[pallet::weight(1_000)]
		pub(super) fn mint2(origin: OriginFor<T>, beneficiary: T::AccountId, #[pallet::compact] amount: T::Balance)-> DispatchResult{
			let sender  = ensure_signed(origin)?;
			ensure!(amount >= T::MinBalance::get(), Error::<T>::BelowMinBalance );
			let mut meta = Self::meta_data();
			ensure!(sender == meta.minter, Error::<T>::NoPermission);

			meta.issuance = meta.issuance.checked_add(amount).ok_or(Error::<T>::Overflow);

			MetaDataStore::<T>::put(meta);

			if Self::increase_balance(beneficiary, amount){
				Self::deposit_event(Event::<T>::Created(beneficiary.clone()));
			}

			Self::deposit_event(Event::<T>::Minted(beneficiary, amount));

			Ok(())
		}

		#[pallet::weight(1_000)]
		pub(super) fn burn(origin: OriginFor<T>, burned: T::AccountId, #[pallet::compact] amount: T::Balance, allow_killing: bool)-> DispatchResult{
			let sender = ensure_signed(origin)?;
			let mut meta = Self::meta_data();
			ensure!(sender == meta.burner, Error::<T>::NoPermission );

			let balance = Accounts::<T>::get(burned);
			ensure!(balance> Zero::zero(), Error::<T>::CannotBurnEmpty);
			let new_balance = balance.saturating_sub(amount);
			let burn_amount = if new_balance < T::MinBalance::get(){
				ensure!(allow_killing, Error::<T>::BelowMinBalance);
				let burn_amount = balance;
				ensure!(meta.issuance.checked_sub(burn_amount).is_some(), Error::<T>::Underflow);
				Accounts::<T>::remove(burned);
				Self::deposit_event(Event::<T>::Killed(burned.clone()));
				burn_amount
			}else{
				let burn_amount = amount;
				ensure!(meta.issuance.checked_sub(burn_amount).is_some(), Error::<T>::Underflow);
				Accounts::<T>::insert(burned, new_balance);
				burn_amount
			};

			meta.issuance = meta.issuance.saturating_sub(burn_amount);
			MetaDataStore::<T>::put(meta);

			Self::deposit_event(Event::<T>::Burned(burned, burn_amount));

			Ok(())
		}

		#[pallet::weight(1_000)]
		#[transactional]
		pub(super) fn transfer2(origin: OriginFor<T>, to: T::AccountId, #[pallet::compact] amount: T::Balance)-> DispatchResult{
			let sender = ensure_signed(origin)?;
			Accounts::<T>::try_mutate(sender, |bal|{
				let new_bal = bal.checked_sub(amount).ok_or(Error::<T>::InsufficientBalance);
				ensure!(new_bal>= T::MinBalance::get(), Error::<T>::BelowMinBalance);
				bal = new_bal;
			});

			Accounts::<T>::try_mutate(to, |rec_bal|{
				let new_bal = rec_bal.saturating_add(amount);
				ensure!(new_bal>= T::MinBalance.get(), Error::<T>::BelowMinBalance);
				rec_bal = new_bal;
			});

			Self::deposit_event(Event::<T>::Transferred(sender, to, amount));

			Ok(())
		}

	}
}

impl<T: Config> Pallet<T>{
	fn increase_balance(acc: &T::AccountId, amount: T::Balance)-> bool{
		Accounts::<T>::mutate( acc, |bal|{
			let created = bal == &Zero::zero();
			bal = bal.saturating_add(amount);
			created
		})
	}
}
