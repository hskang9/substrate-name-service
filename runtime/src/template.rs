/// A runtime module template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references


/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/srml/example/src/lib.rs

use support::{decl_module, decl_storage, decl_event, dispatch::Result, ensure};
use support::traits::{Currency, WithdrawReason, ExistenceRequirement};
use system::{ensure_signed};
use codec::{Encode, Decode};

const INIT_BID: u32 = 1000;

#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct Domain<AccountId, Moment> {
	source: AccountId,
	price: u32,
	ttl: u32,
	reg_date: Moment
}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct Item {
	highest_bid: u32,
	finalized_date: u32,
	reg_date: u32,
	available: bool
}

/// The module's configuration trait.
pub trait Trait: system::Trait + balances::Trait + timestamp::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}


// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as NameStorage {

		
		// Just a dummy storage item.
		// Here we are declaring a StorageValue, `Something` as a Option<u32>
		// `get(something)` is the default getter which returns either the stored `u32` or `None` if nothing stored
		Something get(something): Option<u32>;
		Resolver get(domain): map T::Hash => Domain<T::AccountId, T::Moment>;
		Auction get(item): map T::Hash => Item;
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
		
		// Register domain with 1 year ttl(31556926) and 1 DOT base price
		pub fn register(origin, domain_hash: T::Hash) -> Result {
			let sender = ensure_signed(origin)?;
			ensure!(!<Resolver<T>>::exists(domain_hash), "The domain already exists");
			
			let ttl = 31556926;
			// TODO: Get off-chain worker for getting the time 
			let reg_date = <timestamp::Module<T>>::now();
			let to_balance: T::Balance = T::Balance::from(INIT_BID);
			
			// Try to withdraw price from the user account to register domain 
			let _ = <balances::Module<T> as Currency<_>>::withdraw(&sender, to_balance, WithdrawReason::Reserve, ExistenceRequirement::KeepAlive)?;			

			// Register domain
			let new_domain = Domain{source: sender.clone(), price: INIT_BID, ttl: ttl, reg_date: reg_date};
			let new_item = Item{highest_bid: INIT_BID, finalized_date: 0, reg_date: 0, available: false};

			match Self::new_domain(domain_hash, new_domain, new_item) {
				Ok(()) => (),
				Err(e) => ()
			}
			
			Self::deposit_event(RawEvent::DomainRegistered(sender.clone(), INIT_BID, ttl, reg_date));
			
			Ok(())
		}

		pub fn set_sale(origin, domain_hash: T::Hash) -> Result {


			Ok(())
		}


		// Auction functions
		pub fn new_bid(origin, domain_hash: T::Hash, bid: u32, current_time: u32) -> Result {
			let sender = ensure_signed(origin)?;
			// Ensure that
			// Domain does already exist
			ensure!(!<Resolver<T>>::exists(domain_hash), "The domain does not exist");
			// The auction for the domain exists
			ensure!(<Auction<T>>::exists(domain_hash), "The Auction for the domain does not exist");
			let item = Self::item(domain_hash);
			// The Auction is not finalized yet
			ensure!(item.available && item.finalized_date > current_time, "The auction is not currently available");
			// The bid price is higher than the current highest bid
			ensure!(item.highest_bid < bid, "Bid higher");

			// transfer the money to 


			Ok(())
		}
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId, Hash = <T as system::Trait>::Hash, Price = u32, Ttl=u32
, RegDate=<T as timestamp::Trait>::Moment
, EndDate=<T as timestamp::Trait>::Moment
 {
		// Just a dummy event.
		// Event `Something` is declared with a parameter of the type `u32` and `AccountId`
		// To emit this event, we call the deposit funtion, from our runtime funtions
		SomethingStored(u32, AccountId),
		DomainRegistered(AccountId, Price, Ttl, RegDate),
		NewAuction(AccountId, Hash, Price, Ttl, EndDate), 
		NewBid(AccountId, Hash, Price),
		AuctionFinalized(AccountId, Hash, Price),
	}
);

impl<T: Trait> Module<T> {
	fn new_domain(hash: T::Hash, domain: Domain<T::AccountId, T::Moment>, item: Item) -> Result {
		<Resolver<T>>::insert(hash, domain);
		<Auction<T>>::insert(hash, item);
		Ok(())
	}
}



/// tests for this module
#[cfg(test)]
mod tests {
	use super::*;

	use runtime_io::with_externalities;
	use primitives::{H256, Blake2Hasher};
	use support::{impl_outer_origin, assert_ok, parameter_types};
	use sr_primitives::{traits::{BlakeTwo256, IdentityLookup}, testing::Header};
	use sr_primitives::weights::Weight;
	use sr_primitives::Perbill;

	impl_outer_origin! {
		pub enum Origin for Test {}
	}

	// For testing the module, we construct most of a mock runtime. This means
	// first constructing a configuration type (`Test`) which `impl`s each of the
	// configuration traits of modules we want to use.
	#[derive(Clone, Eq, PartialEq)]
	pub struct Test;
	parameter_types! {
		pub const BlockHashCount: u64 = 250;
		pub const MaximumBlockWeight: Weight = 1024;
		pub const MaximumBlockLength: u32 = 2 * 1024;
		pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
	}
	impl system::Trait for Test {
		type Origin = Origin;
		type Call = ();
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type WeightMultiplierUpdate = ();
		type Event = ();
		type BlockHashCount = BlockHashCount;
		type MaximumBlockWeight = MaximumBlockWeight;
		type MaximumBlockLength = MaximumBlockLength;
		type AvailableBlockRatio = AvailableBlockRatio;
		type Version = ();
	}
	impl Trait for Test {
		type Event = ();
	}
	type TemplateModule = Module<Test>;

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
		system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
	}

	#[test]
	fn it_works_for_default_value() {
		with_externalities(&mut new_test_ext(), || {
			// Just a dummy test for the dummy funtion `do_something`
			// calling the `do_something` function with a value 42
			assert_ok!(TemplateModule::do_something(Origin::signed(1), 42));
			// asserting that the stored value is equal to what we stored
			assert_eq!(TemplateModule::something(), Some(42));
		});
	}
}
