#![cfg_attr(not(feature = "std"), no_std)]

use frame_system::{
	self as system,
	offchain::{
		AppCrypto, CreateSignedTransaction, SendSignedTransaction, Signer,
	}
};
use frame_support::traits::{Currency};
use sp_core::crypto::KeyTypeId;
use sp_runtime::{
	offchain::{http, Duration},
};
use sp_std::vec::Vec;
use lite_json::json::JsonValue;
#[cfg(test)]
mod tests;

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"A002");
pub mod crypto {
	use super::KEY_TYPE;
	use sp_runtime::{
		app_crypto::{app_crypto, sr25519},
		traits::Verify,
	};
	use sp_core::sr25519::Signature as Sr25519Signature;
	use frame_support::sp_runtime::{MultiSigner, MultiSignature};
	app_crypto!(sr25519, KEY_TYPE);

	pub struct TestAuthId;
	impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature> for TestAuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}
	impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for TestAuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}
}

pub use pallet::*;
use sp_runtime::app_crypto::sp_core::crypto::UncheckedFrom;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use super::*;
	use sp_runtime::app_crypto::sp_core::crypto::UncheckedFrom;
	use frame_support::traits::Currency;


	#[pallet::config]
	pub trait Config: frame_system::Config  + pallet_contracts::Config + CreateSignedTransaction<Call<Self>> where
		<Self as frame_system::Config>::AccountId: AsRef<[u8]> + UncheckedFrom<Self::Hash>,
		<<Self as pallet_contracts::Config>::Currency as Currency<<Self as frame_system::Config>::AccountId>>::Balance: From<u128>,
	{
		type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Call: From<Call<Self>>;
		type Currency: Currency<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T>
		where
			T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
			<<T as pallet_contracts::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance: From<u128>,
	{
		fn offchain_worker(block_number: T::BlockNumber) {

			let parent_hash = <system::Pallet<T>>::block_hash(block_number - 1u32.into());
			log::info!("Current block: {:?} (parent hash: {:?})", block_number, parent_hash);

			let transaction_type = block_number % 5u32.into();
			if transaction_type == T::BlockNumber::from(1u32) {
				let res = Self::my_fetch_price_and_send_signed();
				if let Err(e) = res {
					log::error!("Error: {}", e);
				} else {
					let res = Self::my_call_contract_update();
					if let Err(e) = res {
						log::error!("Error: {}", e);
					}
				}
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> where
		T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
		<<T as pallet_contracts::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance: From<u128>
	{
		#[pallet::weight(0)]
		fn my_submit_price(origin: OriginFor<T>, price: (u128,u128)) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			<ElpPrice<T>>::put(price.0);
			<ElcPrice<T>>::put(price.1);
			Self::deposit_event(Event::UpdatePrice(price, who));
			Ok(().into())
		}

		#[pallet::weight(0)]
		pub fn my_set_update_fn_para(origin: OriginFor<T>, address:T::AccountId, selector:Vec<u8>) -> DispatchResultWithPostInfo {
			let _who = ensure_signed(origin)?;
			<ContractAddress<T>>::put(address);
			<Selector<T>>::put(selector);
			Ok(().into())
		}

		#[pallet::weight(10_000)]
		fn call_contract_update(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let _who = ensure_signed(origin)?;

			let elc_price = <ElcPrice<T>>::get();
			let elp_price = <ElpPrice<T>>::get();
			let address = <ContractAddress<T>>::get();
			let selector = <Selector<T>>::get();
			log::info!("=============================================================================================");
			log::info!("address:{:?},selector:{:?},elc_price:{},elp_price{}", address, selector, elc_price,elp_price);
			let encoded_elp = u128::encode(&elp_price);
			let encoded_elc = u128::encode(&elc_price);
			let input_data = [&selector[..], &encoded_elp[..], &encoded_elc[..]].concat();
			let exec_result = <pallet_contracts::Pallet<T>>::bare_call(_who, address.clone(), 0.into(), 600000000000000, input_data);
			match exec_result.exec_result {
				Ok(v) => {
					let result_val = bool::decode(&mut &v.data[..]);
					match result_val {
						Ok(b) => {
							log::info!("================update ok============{}",b);
						},
						Err(e) => { log::error!("{:?}",e)},
					}
				},
				Err(e) => {
					log::error!("================error============{:?}",e);
					log::error!("============gas================={}",exec_result.gas_consumed);
				},
			}

			Ok(().into())
		}
	}

	/// Events for the pallet.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> where
		T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
		<<T as pallet_contracts::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance: From<u128>
	{
		UpdatePrice((u128,u128), T::AccountId),
	}

	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T> where
		T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
		<<T as pallet_contracts::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance: From<u128>
	{
		type Call = Call<T>;

		fn validate_unsigned(
			_source: TransactionSource,
			_call: &Self::Call,
		) -> TransactionValidity {
			InvalidTransaction::Call.into()
		}
	}

	//oracle price fetch contract address
	#[pallet::storage]
	#[pallet::getter(fn contract_address)]
	pub(super) type ContractAddress<T: Config> = StorageValue<_, T::AccountId, ValueQuery>;

	//oracle price fetch selector
	#[pallet::storage]
	#[pallet::getter(fn selector)]
	pub(super) type Selector<T: Config> = StorageValue<_, Vec<u8>, ValueQuery>;

	//elp price
	#[pallet::storage]
	#[pallet::getter(fn elp_price)]
	pub(super) type ElpPrice<T: Config> = StorageValue<_, u128, ValueQuery>;

	//elc price
	#[pallet::storage]
	#[pallet::getter(fn elc_price)]
	pub(super) type ElcPrice<T: Config> = StorageValue<_, u128, ValueQuery>;
}

impl<T: Config> Pallet<T>
	where
		T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
		<<T as pallet_contracts::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance: From<u128>
{

	pub fn my_call_contract_update()  -> Result<(), &'static str>  {
		let signer = Signer::<T, T::AuthorityId>::all_accounts();
		if !signer.can_sign() {
			return Err(
				"No local accounts available. Consider adding one via `author_insertKey` RPC."
			)?
		}
		let results = signer.send_signed_transaction(
			|_account| {
				Call::call_contract_update()
			}
		);

		for (acc, res) in &results {
			match res {
				Ok(()) => log::info!("[{:?}] update price ok!", acc.id),
				Err(e) => log::error!("[{:?}] Failed to submit transaction: {:?}", acc.id, e),
			}
		}

		Ok(())
	}

	fn my_fetch_price_and_send_signed() -> Result<(), &'static str> {
		let signer = Signer::<T, T::AuthorityId>::all_accounts();
		if !signer.can_sign() {
			return Err(
				"No local accounts available. Consider adding one via `author_insertKey` RPC."
			)?
		}
		let price = Self::my_fetch_price().map_err(|_| "Failed to fetch price")?;
		let results = signer.send_signed_transaction(
			|_account| {
				Call::my_submit_price(price)
			}
		);

		for (acc, res) in &results {
			match res {
				Ok(()) => log::info!("[{:?}] Submitted price of {} cents,{} cents", acc.id, price.0,price.1),
				Err(e) => log::error!("[{:?}] Failed to submit transaction: {:?}", acc.id, e),
			}
		}

		Ok(())
	}

	fn my_fetch_price() -> Result<(u128,u128), http::Error> {
		let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(5_000));
		let request = http::Request::get(
			"https://min-api.cryptocompare.com/data/price?fsym=DOT&tsyms=USD"
		);
		let pending = request
			.deadline(deadline)
			.send()
			.map_err(|_| http::Error::IoError)?;
		let response = pending.try_wait(deadline)
			.map_err(|_| http::Error::DeadlineReached)??;
		if response.code != 200 {
			log::warn!("Unexpected status code: {}", response.code);
			return Err(http::Error::Unknown);
		}
		let body = response.body().collect::<Vec<u8>>();
		let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
			log::warn!("No UTF8 body");
			http::Error::Unknown
		})?;
		let price = match Self::my_parse_price(body_str) {
			Some(price) => Ok(price),
			None => {
				log::warn!("Unable to extract price from the response: {:?}", body_str);
				Err(http::Error::Unknown)
			}
		}?;

		log::warn!("Got price: {} cents,{} cents", price.0,price.1);

		Ok(price)
	}

	fn my_parse_price(price_str: &str) -> Option<(u128,u128)> {
		let val = lite_json::parse_json(price_str);
		let price = val.ok().and_then(|v| match v {
			JsonValue::Object(obj) => {
				let mut elp_chars = "USD".chars();
				obj.into_iter()
					.find(|(k, _)| k.iter().all(|k| Some(*k) == elp_chars.next()))
					.and_then(|v| match v.1 {
						JsonValue::Number(number) => Some(number),
						_ => None,
					})
			},
			_ => None
		})?;
		let exp = price.fraction_length.checked_sub(2).unwrap_or(0);
		let elp_price = price.integer as u32 * 100 + (price.fraction / 10_u64.pow(exp)) as u32;
		let val2 = lite_json::parse_json(price_str);
		let price = val2.ok().and_then(|v| match v {
			JsonValue::Object(obj) => {
				let mut elc_chars = "USD".chars();
				obj.into_iter()
					.find(|(k, _)| k.iter().all(|k| Some(*k) == elc_chars.next()))
					.and_then(|v| match v.1 {
						JsonValue::Number(number) => Some(number),
						_ => None,
					})
			},
			_ => None
		})?;
		let exp = price.fraction_length.checked_sub(2).unwrap_or(0);
		let elc_price = price.integer as u32 * 100 + (price.fraction / 10_u64.pow(exp)) as u32;
		Some((elp_price as u128, elc_price as u128))
	}
}
