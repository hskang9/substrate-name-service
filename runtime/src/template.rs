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
pub struct Domain<AccountId, Balance, Moment> {
	owner: AccountId,
	price: Balance,
	ttl: Moment,
	registered_date: Moment
	available: bool,
	highest_bid: Balance,
	auction_closed: Moment,
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
		Domains get(total_domains): Option<u32>;
		Resolver get(domain): map T::Hash => Domain<T::AccountId, T::Balance, T::Moment>;
	}
}

// The module's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		// this is needed only if you are using events in your module
		fn deposit_event() = default;
		
		// Register domain with 1 year ttl(31556926) and 1 DOT base price
		pub fn register_domain(origin, domain_hash: T::Hash) -> Result {
			let sender = ensure_signed(origin)?;
			ensure!(!<Resolver<T>>::exists(domain_hash), "The domain already exists");
			// Convert numbers into generic types which codec supports
			// Generic types can process arithmetics and comparisons just as other rust variables
			let ttl = T::Moment::from(31556926);
			let init_price = T::Balance::from(INIT_BID); 
			let reg_date: T::Moment = <timestamp::Module<T>>::now();
			
			// Try to withdraw price from the user account to register domain 
			let _ = <balances::Module<T> as Currency<_>>::withdraw(&sender, init_price, WithdrawReason::Reserve, ExistenceRequirement::KeepAlive)?;			

			// Register domain
			let new_domain = Domain{
				owner: sender.clone(),
				price: init_price, 
				ttl: ttl, 
				registered_date: reg_date,
				available: false,
				highest_bid: T::Balance::from(0),
				auction_closed: T::Moment::from(0)
				};

			match <Domains<T>>::get() {
				None => <Domains<T>>::set(0);
				_ => ()
			}

			match <Resolver<T>>::insert(domain_hash.clone(), new_domain) {
				Ok(()) => {
					Self::domains
					<Domains<T>>::set(Self::total_domains + 1);
					Self::deposit_event(RawEvent::DomainRegistered(sender.clone(), init_price, ttl, reg_date))
				},
				Err(e) => ()
			}

				
			Ok(())
		}

		pub fn set_sale(origin, domain_hash: T::Hash) -> Result {
			let sender = ensure_signed(origin)?;
			let mut new_domain = Self::item(domain_hash.clone());
			// Ensure the sender is the owner of the domain
			ensure!(sender == Self::domain(domain_hash.clone()).owner, "You are not the owner of the domain");
			// Set sale and put time to finalize the auction
			new_domain.available = true;
			new_domain.
			// Set new domain in the Domain storage
			

			

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

			// transfer the token


			Ok(())
		}
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId, <T as system::Trait>::Hash, <T as balances::Trait>::Balance, <T as timestamp::Trait>::Moment
 {
		DomainRegistered(AccountId, Balance, Moment, Moment),
		NewAuction(AccountId, Hash, Balance, Moment, Moment), 
		NewBid(AccountId, Hash, Balance),
		AuctionFinalized(AccountId, Hash, Balance),
	}
);

impl<T: Trait> Module<T> {
	fn new_domain(hash: T::Hash, domain: Domain<T::AccountId, T::Balance, T::Moment>, item: Item) -> Result {
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

	impl Trait for Test  {
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
