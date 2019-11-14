use support::{decl_module, decl_storage, decl_event, dispatch::Result, ensure};
use support::traits::{Currency, WithdrawReason, ExistenceRequirement};
use system::{ensure_signed};
use codec::{Encode, Decode};
use rstd::prelude::*;

pub type BYTES = Vec<u8>;


#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct DataPoint<AccountId> {
	/// Array of accounts that are able to get access to the data point
	access: Vec<AccountId>,
	/// Whether the data is public and can be shown to anyone
	public: bool,
	/// Encrypted/Decrypted data in byte array(e.g. b"twt://@{twitter username}", b"ipfs://{some IPFS hash}", b"jpg://{some blob of image}" )
	data: BYTES,
}

// Module's function and Methods of custom struct to be placed here
impl<T: Trait> Module<T> {

	pub fn new_datapoint(account: T::AccountId, data: BYTES) -> DataPoint<T::AccountId> {
		DataPoint {
			access: vec![account],
			public: false,
			data: data,
		}
	}

	pub fn can_access(account: T::AccountId, data_point: DataPoint<T::AccountId>) -> bool {
		for i in data_point.access {
			if i == account {
				return true;
			}
		}
		return false;
	}

	// TODO: Add this to <balances::Module<T>> and test with u128
	/// Convert u32 to u128 generic type Balance type
	pub fn to_balance(u: u32, digit: &str) -> T::Balance {
		// Power exponent function
		let pow = |u: u32, p: u32| -> T::Balance {
			let mut base = T::Balance::from(u);
			for _i in 0..p { 
				base *= T::Balance::from(10)
			}
			return base;
		};
		let result = match digit  {
			"femto" => T::Balance::from(u),
			"nano" =>  pow(u, 3),
			"micro" => pow(u, 6),
			"milli" => pow(u, 9),
			"one" => pow(u,12),
			"kilo" => pow(u, 15),
			"mega" => pow(u, 18),
			"giga" => pow(u, 21),
			"tera" => pow(u, 24),
			"peta" => pow(u, 27),
			"exa" => pow(u, 30),
			"zetta" => pow(u, 33),
			"yotta" => pow(u, 36),
			_ => T::Balance::from(0),
		}; 
		result 
	}
}

/// The module's configuration trait.
pub trait Trait: system::Trait + balances::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	
}

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as TemplateModule {
		// Just a dummy storage item.
		// Here we are declaring a StorageValue, `Something` as a Option<u32>
		// `get(something)` is the default getter which returns either the stored `u32` or `None` if nothing stored
		Something get(something): Option<u32>;
	}
}

// The module's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		// this is needed only if you are using events in your module
		fn deposit_event() = default;

		// Just a dummy entry point.
		// function that can be called by the external world as an extrinsics call
		// takes a parameter of the type `AccountId`, stores it and emits an event
		pub fn do_something(origin, something: u32) -> Result {
			// TODO: You only need this if you want to check it was signed.
			let who = ensure_signed(origin)?;

			// TODO: Code to execute when something calls this.
			// For example: the following line stores the passed in u32 in the storage
			Something::put(something);

			// here we are raising the Something event
			Self::deposit_event(RawEvent::SomethingStored(something, who));
			Ok(())
		}
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
		// Just a dummy event.
		// Event `Something` is declared with a parameter of the type `u32` and `AccountId`
		// To emit this event, we call the deposit funtion, from our runtime funtions
		SomethingStored(u32, AccountId),
	}
);