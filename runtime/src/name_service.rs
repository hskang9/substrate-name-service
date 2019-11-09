use support::{decl_module, decl_storage, decl_event, dispatch::Result, ensure};
use support::traits::{Currency, WithdrawReason, ExistenceRequirement};
use system::{ensure_signed};
use codec::{Encode, Decode};
use rstd::prelude::*;

// The timestamp inherent type is u64 and Substrate calculates as milliseconds, but `From` for all generic types supports u8, u16, u32 in SimpleArithmetic trait, saying that those are not fallible.
// Therefore, use TryFrom for big integers
// FIXME: TryFrom does not support unwrap() in its result so make function for conversion
// use core::convert::TryFrom;
// FIXME: TryFrom causes a bug for inconsistency in Storage hash, actually type bigger than u32 causes an error

// 1 year in blockseconds
// each block is assumed to be generated in 6 seconds. divide that with 31556952(1 year) seconds and you get 5259492 blocks. 
const YEAR: u32 =  5259492;
pub type IPV4 = [u8; 4];
pub type IPV6 = [u16; 6];
pub type BYTES = Vec<u8>;

#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct Domain<AccountId, Balance, BlockNumber> {
	/// domain name in bytestring e.g. b'hyungsukkang.eth'
	name: BYTES,
	/// source of this domain a.k.a. the address of the blockchain
	source: AccountId,
	/// the current domain price
	price: Balance,
	/// Time to claim the ownership
	ttl: BlockNumber,
	/// Registered date in block height
	registered_date: BlockNumber,
	/// whether it is available for purchase or sale
	available: bool,
	/// highest bid in the auction stage
	highest_bid: Balance,
	/// bidder who bidded highest
	bidder: AccountId,
	/// Auction closing date
	auction_closed: BlockNumber,

	/// TODO: Try to make browser engine which asks for this with Servo fork
	/// IPV4 in case where the owner wants to put IP address
	ipv4: IPV4,
	/// IPV6 in case where the owner wants to put IP address for his or her IoT device 
	ipv6: IPV6,
}

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

	pub fn new_domain(domain_name: BYTES, source: T::AccountId) -> Domain<T::AccountId, T::Balance, T::BlockNumber> {
		// Convert numbers into generic types which is mapped to native type in lib.rs
		// Generic types can process arithmetics and comparisons just as other rust variables
		let ttl = T::BlockNumber::from(YEAR);
		let init_price = Self::to_balance(1, "milli");
		let reg_date: T::BlockNumber = <system::Module<T>>::block_number(); 
		
		Domain {
			name: domain_name,
			source: source.clone(),
			price: init_price,
			ttl: ttl,
			registered_date: reg_date,
			available: false,
			highest_bid: T::Balance::from(0),
			bidder: source,
			auction_closed: T::BlockNumber::from(0),
			ipv4: [0,0,0,0],
			ipv6: [0,0,0,0,0,0],
		}
	}

	// TODO: Add this to <balances::Module<T>> and test with u128
	/// Convert u32 to u128 generic type Balance type
	pub fn to_balance(u: u32, digit: &str) -> T::Balance {
		let power = |u: u32, p: u32| -> T::Balance {
			let mut base = T::Balance::from(u);
			for _i in 0..p { 
				base *= T::Balance::from(10)
			}
			return base;
		};
		let result = match digit  {
			"femto" => T::Balance::from(u),
			"nano" =>  power(u, 3),
			"micro" => power(u, 6),
			"milli" => power(u, 9),
			"one" => power(u,12),
			"kilo" => power(u, 15),
			"mega" => power(u, 18),
			"giga" => power(u, 21),
			"tera" => power(u, 24),
			"peta" => power(u, 27),
			"exa" => power(u, 30),
			"zetta" => power(u, 33),
			"yotta" => power(u, 36),
			_ => T::Balance::from(u)
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
	trait Store for Module<T: Trait> as NamingServiceModule {
		/// Total number of domains
		Domains get(total_domains): u64;
		/// Hash is the blake2b hash of the domain name
		/// In Javascript, use @polkadot/util-crypto's blake2AsHex("<domain name you want>" 256) and put the hexstring in the polkadot.js apps param.
		/// Or use blakejs with this example.
		/// > var blake = require('blakejs');
		/// > console.log(blake.blake2s('hyungsukkang.dot'))
		/// fecf3628563657233c1d29fd6589bcb792d1ce7611892490c3dd5857647006d7
		Resolver get(domain): map T::Hash => Domain<T::AccountId, T::Balance, T::BlockNumber>;
		/// Reverse resolver for account => domain
		Reverse get(account): map T::AccountId => Vec<T::Hash>;
		/// Account for multichain (account_address, slip-44-chain-index) => {corresponding chain address}(e.g. b"0x6EaD823cfB6d45996b8E413C7bE43282f042A78e")
		Accounts get(address): map (T::AccountId, u32) => BYTES;
		/// Private data points for each account (account_address, W23-web2service-index) => {corresponding data points}
		/// TODO: specify W23-index
		Privacy get(data_point): map (T::AccountId, u32) => DataPoint<T::AccountId> ;
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
/// domain and reverse logics //////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////	
			
		// Register domain with estimated 1 year ttl blocktime(31556926000 milliseconds) and 1 milli DEV(0.001 DEV) base price
		pub fn register_domain(origin, domain_hash: T::Hash, domain_name: BYTES) -> Result {
			let sender = ensure_signed(origin)?;
			ensure!(!<Resolver<T>>::exists(domain_hash), "The domain already exists");
			// Make new Domain struct
			let new_domain = Self::new_domain(domain_name, sender.clone());

			// Try to withdraw registration fee from the user without killing the account
			let _ = <balances::Module<T> as Currency<_>>::withdraw(&sender.clone(), new_domain.price, WithdrawReason::Reserve, ExistenceRequirement::KeepAlive)?;			

			// TODO: Make new profile
			
			// Insert new domain to the Resolver state
			<Resolver<T>>::insert(domain_hash, new_domain.clone());

			// Increment domain number	
			let mut domains = Self::total_domains();
			domains = domains.wrapping_add(1);

			// Store domain number to Domains state
			Domains::put(domains);			

			// Deposit event
			Self::deposit_event(RawEvent::DomainRegistered(sender.clone(), new_domain.price, new_domain.ttl, new_domain.registered_date));
			
			Ok(())
		}

		pub fn set_ipv4(origin, domain_hash: T::Hash, ipv4: IPV4) -> Result {
			// Ensure that 
			// domain exists
			ensure!(<Resolver<T>>::exists(domain_hash), "The domain does not exist");
			// the sender is the source of the domain
			let sender = ensure_signed(origin)?;
			let mut new_domain = Self::domain(domain_hash);
			ensure!(sender == new_domain.source, "you are not the source of the domain");
			
			// Set ipv4 for new domain
			let old_ipv4 = new_domain.ipv4;
			new_domain.ipv4 = ipv4;

			// Change domain data with the new one and emit event
			<Resolver<T>>::mutate(domain_hash.clone(), |domain| *domain = new_domain.clone());
			Self::deposit_event(RawEvent::SetIPV4(domain_hash, old_ipv4, new_domain.ipv4));

			Ok(())
		}

		pub fn resolve(_origin, domain_hash: T::Hash) -> Result {
			ensure!(<Resolver<T>>::exists(domain_hash), "The domain does not exist");
			let domain = Self::domain(domain_hash);
			Self::deposit_event(RawEvent::DomainResolved(domain_hash, domain.source, domain.price, domain.available, domain.highest_bid, domain.bidder, domain.auction_closed));

			Ok(())
		}

		pub fn renew(origin, domain_hash: T::Hash) -> Result {
			let sender = ensure_signed(origin)?;

			let mut new_domain = Self::domain(domain_hash.clone());
			let now = <system::Module<T>>::block_number();
			// Ensure the sender is the source of the domain and its ttl is not expired
			ensure!(new_domain.source == sender && now < new_domain.registered_date + new_domain.ttl, "You are either not the source of the domain or the domain is expired");
			
			// Extend domain TTL by a year
			let ttl = T::BlockNumber::from(YEAR);
			new_domain.ttl += ttl;		

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
			let now = <system::Module<T>>::block_number();
			// Ensure the sender is the source of the domain or its ttl is expired
			ensure!(sender == new_domain.source || new_domain.registered_date + new_domain.ttl < now, "You are neither the source of the domain or the claimer after the domain's TTL");

			
			// Set domain available for selling
			new_domain.available = true;

			// Set auction to be closed after 1 hour(60* 60 seconds) * 1000(milliseconds conversion) using timestamp 
			let converted = T::BlockNumber::from(600);
			new_domain.auction_closed = now + converted;

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
			let now = <system::Module<T>>::block_number();
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
			let _sender = ensure_signed(origin)?; 
			// Ensure that
			// Domain does already exist
			ensure!(<Resolver<T>>::exists(domain_hash), "The domain is not registered yet");
			// But wait, get domain data and time
			let mut new_domain = Self::domain(domain_hash);
			let now = <system::Module<T>>::block_number();
			// The auction is available
			ensure!(new_domain.available, "The auction for the domain is currently not available");
			// The auction is finalized or the source wants to finalize the auction(test)
			// TEST: If you want to test auction finalization without waiting for 1 hour, just add '|| sender == new_domain.source in ensure! macro
			ensure!(now > new_domain.auction_closed, "The auction has not been finalized yet");

			let _ = <balances::Module<T> as Currency<_>>::transfer(&new_domain.bidder, &new_domain.source, new_domain.highest_bid);


			let ttl = T::BlockNumber::from(YEAR);

			// Set new domain data to bidder as source, highest_bid as price, and reinitialize rest of them 
			new_domain.source = new_domain.bidder.clone();
			new_domain.price = new_domain.highest_bid;
			new_domain.available = false;
			new_domain.ttl = ttl;
			new_domain.registered_date = now;
			new_domain.available = false;
			new_domain.highest_bid = T::Balance::from(0);
			new_domain.auction_closed = T::BlockNumber::from(0);

			// mutate domain with new_domain struct in the Domain state
			<Resolver<T>>::mutate(domain_hash.clone(), |domain| *domain = new_domain.clone());
			Self::deposit_event(RawEvent::AuctionFinalized(new_domain.bidder, domain_hash, new_domain.highest_bid));

			Ok(())
		}


////////////////////////////////////////////////////////////////////////////////////////////////
/// accounts and data point logics //////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId, <T as system::Trait>::Hash, <T as balances::Trait>::Balance, <T as system::Trait>::BlockNumber
 {
		DomainRegistered(AccountId, Balance, BlockNumber, BlockNumber),
		SetIPV4(Hash, IPV4, IPV4),
		NewAuction(AccountId, Hash, BlockNumber, BlockNumber), 
		NewBid(AccountId, Hash, Balance),
		AuctionFinalized(AccountId, Hash, Balance),
		DomainResolved(Hash, AccountId, Balance, bool, Balance, AccountId, BlockNumber),
		DomainRenewal(Hash, AccountId, BlockNumber),
	}
);