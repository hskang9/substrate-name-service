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
const YEAR: u32 = 31556926;

#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct Domain<AccountId, Balance, Moment> {
	source: AccountId,
	price: Balance,
	ttl: Moment,
	registered_date: Moment,
	available: bool,
	highest_bid: Balance,
	bidder: AccountId,
	auction_closed: Moment,
}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct Item {
	
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
		Domains get(total_domains): u64;
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
			let ttl = T::Moment::from(YEAR);
			let init_price = T::Balance::from(INIT_BID); 
			let reg_date: T::Moment = <timestamp::Module<T>>::now();
			
			// Try to withdraw registration fee from the user without killing the account
			// TODO: make validator pot for them to decide what to do with registration fees
			let _ = <balances::Module<T> as Currency<_>>::withdraw(&sender, init_price, WithdrawReason::Reserve, ExistenceRequirement::KeepAlive)?;			

			// make new Domain struct
			let new_domain = Domain{
				source: sender.clone(),
				price: init_price,
				ttl: ttl,
				registered_date: reg_date,
				available: false,
				highest_bid: T::Balance::from(0),
				bidder: sender.clone(),
				auction_closed: T::Moment::from(0)
				};

			// Insert new domain to the Resolver state
			<Resolver<T>>::insert(domain_hash, new_domain);

			// Increment domain number	
			let mut domains = Self::total_domains();
			domains = domains.wrapping_add(1);

			// Store domain number to Domains state
			Domains::put(domains);			

			// Deposit event
			Self::deposit_event(RawEvent::DomainRegistered(sender.clone(), init_price, ttl, reg_date));
			
			Ok(())
		}

		pub fn resolve(origin, domain_hash: T::Hash) -> Result {
			ensure!(<Resolver<T>>::exists(domain_hash), "The domain does not exist");
			let domain = Self::domain(domain_hash);
			Self::deposit_event(RawEvent::DomainResolved(domain_hash, domain.source));

			Ok(())
		}

		pub fn renew(origin, domain_hash: T::Hash) -> Result {
			let sender = ensure_signed(origin)?;

			let mut new_domain = Self::domain(domain_hash.clone());
			let now = <timestamp::Module<T>>::now();
			// Ensure the sender is the source of the domain and its ttl is not expired
			ensure!(new_domain.source == sender && now < new_domain.registered_date + new_domain.ttl, "You are either not the source of the domain or the domain is expired");
			
			// Extend domain TTL by a year 
			new_domain.ttl += T::Moment::from(YEAR);		

			// Try to withdraw price from the user account to renew the domain 
			let _ = <balances::Module<T> as Currency<_>>::withdraw(&sender, new_domain.price, WithdrawReason::Reserve, ExistenceRequirement::KeepAlive)?;			


			// mutate domain with new_domain struct in the Domain state
			<Resolver<T>>::mutate(domain_hash.clone(), |domain| *domain = new_domain.clone());
			Self::deposit_event(RawEvent::DomainRenewal(domain_hash, sender, new_domain.registered_date + new_domain.ttl));


			Ok(())
		}

		pub fn claim_auction(origin, domain_hash: T::Hash) -> Result {
			let sender = ensure_signed(origin)?;
			// Ensure that
			// Domain does already exist
			ensure!(<Resolver<T>>::exists(domain_hash), "The domain does not exist");
			// But wait, get domain data and time
 			let mut new_domain = Self::domain(domain_hash.clone());
			let now = <timestamp::Module<T>>::now();
			// Ensure the sender is the source of the domain or its ttl is expired
			ensure!(sender == new_domain.source || new_domain.registered_date + new_domain.ttl < now, "You are neither the source of the domain or the claimer after the domain's TTL");

			
			// Set domain available for selling
			new_domain.available = true;

			// Set auction to be closed after 1 year using timestamp 
			new_domain.auction_closed = now + T::Moment::from(YEAR);

			// mutate domain with new_domain struct in the Domain state
			<Resolver<T>>::mutate(domain_hash.clone(), |domain| *domain = new_domain.clone());
			Self::deposit_event(RawEvent::NewAuction(sender, domain_hash, now, new_domain.auction_closed));


			Ok(())
		}

		
		pub fn new_bid(origin, domain_hash: T::Hash, bid: T::Balance) -> Result {
			let sender = ensure_signed(origin)?;
			// Ensure that
			// Domain does already exist
			ensure!(<Resolver<T>>::exists(domain_hash), "The domain does not exist");
			// But wait, get domain data
			let mut new_domain = Self::domain(domain_hash.clone());
			// The auction is available
			ensure!(new_domain.available, "The auction for the domain is currently not available");
			// The auction is not finalized
			let now = <timestamp::Module<T>>::now();
			ensure!(new_domain.auction_closed > now, "The bid for the auction is already finalized");
			// The bid price is higher than the current highest bid
			ensure!(new_domain.highest_bid < bid.clone(), "Bid higher");
			

			// Set new domain data
			new_domain.bidder = sender.clone();
			new_domain.highest_bid = bid.clone();
			
			// mutate domain with new_domain struct in the Domain state
			<Resolver<T>>::mutate(domain_hash.clone(), |domain| *domain = new_domain.clone());
			Self::deposit_event(RawEvent::NewBid(sender, domain_hash, bid));

			Ok(())
		}

		pub fn finalize_auction(origin, domain_hash: T::Hash) -> Result {
			let sender = ensure_signed(origin)?; 
			// Ensure that
			// Domain does already exist
			ensure!(<Resolver<T>>::exists(domain_hash), "The domain is not registered yet");
			// But wait, get domain data and time
			let mut new_domain = Self::domain(domain_hash);
			let now = <timestamp::Module<T>>::now();
			// The auction is available
			ensure!(new_domain.available, "The auction for the domain is currently not available");
			// The auction is finalized or the source wants to finalize(test)
			ensure!(now > new_domain.auction_closed || sender == new_domain.source, "The auction has not been finalized yet");

			let _ = <balances::Module<T> as Currency<_>>::transfer(&new_domain.bidder, &new_domain.source, new_domain.highest_bid);

			// Set new domain data to bidder as source, highest_bid as price, and reinitialize rest of them 
			new_domain.source = new_domain.bidder.clone();
			new_domain.price = new_domain.highest_bid;
			new_domain.available = false;
			new_domain.ttl = T::Moment::from(YEAR);
			new_domain.registered_date = now;
			new_domain.available = false;
			new_domain.highest_bid = T::Balance::from(0);
			new_domain.auction_closed = T::Moment::from(0);

			// mutate domain with new_domain struct in the Domain state
			<Resolver<T>>::mutate(domain_hash.clone(), |domain| *domain = new_domain.clone());
			Self::deposit_event(RawEvent::AuctionFinalized(new_domain.bidder, domain_hash, new_domain.highest_bid));

			Ok(())
		}
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId, <T as system::Trait>::Hash, <T as balances::Trait>::Balance, <T as timestamp::Trait>::Moment
 {
		// Just a dummy event.
		// Event `Something` is declared with a parameter of the type `u32` and `AccountId`
		// To emit this event, we call the deposit funtion, from our runtime funtions
		SomethingStored(u32, AccountId),
		DomainRegistered(AccountId, Balance, Moment, Moment),
		NewAuction(AccountId, Hash, Moment, Moment), 
		NewBid(AccountId, Hash, Balance),
		AuctionFinalized(AccountId, Hash, Balance),
		DomainResolved(Hash, AccountId),
		DomainRenewal(Hash, AccountId, Moment),
	}
);

impl<T: Trait> Module<T> {
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

	impl timestamp::Trait for Test {
		type Moment = u64;
		type OnTimestampSet = ();
        type MinimumPeriod = ();
	}

	impl balances::Trait for Test {
		type Balance = u128;
		type OnFreeBalanceZero = ();
		type OnNewAccount = ();
		type TransactionPayment = ();
		type TransferPayment = ();
		type DustRemoval = ();
		type Event = ();
		type ExistentialDeposit = ();
		type TransferFee = ();
		type CreationFee = ();
		type TransactionBaseFee = ();
		type TransactionByteFee = ();
		type WeightToFee = ();
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
			//assert_ok!(TemplateModule::register_domain(""));
			// asserting that the stored value is equal to what we stored
			assert_eq!(TemplateModule::total_domains(), 0);
		});
	}

	#[test]
	fn test_register_domain() {
		with_externalities(&mut new_test_ext(), || {
			let alice = 1u64;
			let dummy_hash = H256([2; 32]);
			assert_ok!(TemplateModule::register_domain(Origin::signed(alice), dummy_hash));
			assert_eq!(TemplateModule::domain(dummy_hash).source, alice);
		});
	}

	// TODO: Test other functions with features
	// - Catching events after the event
	// - Set balance of the test account

	#[test]
	fn test_claim_auction() {
		with_externalities(&mut new_test_ext(), || {
			let alice = 1u64;
			let dummy_hash = H256([2; 32]);
			assert_ok!(TemplateModule::register_domain(Origin::signed(alice), dummy_hash));
			assert_eq!(TemplateModule::domain(dummy_hash).source, alice);
		});
	}
}
