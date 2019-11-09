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

	pub fn can_access(account: T::AccountId, data_point: DataPoint) -> bool {
		for i in data_point.access {
			if i == account {
				return true;
			}
		}
		return false;
	}

	pub fn add_access(appointer: T::AccountId, appointee: T::AccountId, data_point: DataPoint) -> DataPoint<T::AccountId> {
		if Self::can_access(appointer, data_point) {
			data_point.access.push(appointee);
			return data_point;
		}
	}

	pub fn remove_access(remover: T::AccountId, removed: T::AccountId, data_point: DataPoint) -> DataPoint<T::AccountId> {
        if Self::can_access(remover, data_point) {
            data_point.access.remove_item(removed);
            return data_point;
        }
    }

	// TODO: Add this to <balances::Module<T>> and test with u128
	/// Convert u32 to u128 generic type Balance type
	pub fn to_balance(u: u32, digit: &str) -> T::Balance {
		/// Power exponent function
		let pow = |u: u32, p: u32| -> T::Balance {
			let mut base = T::Balance::from(u);
			for _i in 0..p { 
				base *= T::Balance::from(10)
			}
			return base;
		};
		let result = match digit  {
			"femto" | _ => T::Balance::from(u),
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
	trait Store for Module<T: Trait> as IdentityModule {
		/// Total number of domains
		TotalAccounts get(total_accounts): u64;
		/// Interchain accounts
        Accounts get(address): map (T::AccountId, u32) => BYTES;
		/// Private data points for each account (account_address, W23-web2service-index) => {corresponding data points}
		/// TODO: specify W23-index
		Privacy get(data_point): map (T::AccountId, u32) => DataPoint<T::AccountId>;
	}
}

// The module's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		// this is needed only if you are using events in your module
		fn deposit_event() = default;

////////////////////////////////////////////////////////////////////////////////////////////////
/// accounts and data point logics //////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////


	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId, <T as system::Trait>::Hash, <T as balances::Trait>::Balance, <T as system::Trait>::BlockNumber
 {
		SetAccount(AccountId, u32, BYTES),
	}
);