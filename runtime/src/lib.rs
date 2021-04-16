#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit="256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata,U256,H256,H160};
use sp_std::{prelude::*, marker::PhantomData};
use codec::{Encode, Decode};

use sp_runtime::{
	ApplyExtrinsicResult, generic, create_runtime_str, impl_opaque_keys, MultiSignature,PerThing,
	transaction_validity::{TransactionValidity, TransactionSource},
        traits::{BlakeTwo256, Block as BlockT, AccountIdLookup, Verify, IdentifyAccount},
};

use sp_api::impl_runtime_apis;
use sp_runtime::SaturatedConversion;
use fp_rpc::TransactionStatus;

// XCM imports
use xcm::v0::{Junction, MultiLocation, NetworkId};

use frame_system::limits::{BlockLength, BlockWeights};
use frame_system::{EnsureNever, EnsureRoot, EnsureSigned};

// A few exports that help ease life for downstream crates.
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use pallet_timestamp::Call as TimestampCall;
pub use pallet_balances::Call as BalancesCall;
pub use sp_runtime::{Permill, Perbill};
pub use frame_support::{
	construct_runtime, parameter_types, StorageValue, ConsensusEngineId,
	traits::{KeyOwnerProofSystem, Randomness,FindAuthor,LockIdentifier,U128CurrencyToVote,IsInVec,All},
	weights::{
		Weight, IdentityFee, DispatchClass,
		constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
	},
};

/// Import the template pallet.
pub use template;
pub use gateway;

use pallet_evm::{
        FeeCalculator, HashedAddressMapping, EnsureAddressTruncated, Runner,Account as EVMAccount,
};

pub type BlockNumber = u32;
pub type Signature = MultiSignature;
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
pub type AccountIndex = u32;
pub type Balance = u128;
pub type Index = u32;

pub type Hash = sp_core::H256;
pub type DigestItem = generic::DigestItem<Hash>;

pub mod opaque {
	use super::*;
	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	pub type SessionHandlers = ();
	impl_opaque_keys! {
		pub struct SessionKeys {}
	}
}

pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("phoenix-parachain"),
	impl_name: create_runtime_str!("phoenix-parachain"),
	authoring_version: 1,
	spec_version: 100,
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
};

// add by WangYi
pub const PNX: Balance = 1_000_000_000_000;
pub const DOLLARS: Balance = PNX;
pub const CENTS: Balance = DOLLARS / 100;
pub const MILLICENTS: Balance = DOLLARS / 1000;

pub const MILLISECS_PER_BLOCK: u64 = 6000;
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;
pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
const MAXIMUM_BLOCK_WEIGHT: Weight = 2 * WEIGHT_PER_SECOND;

parameter_types! {
	pub const BlockHashCount: BlockNumber = 250;
	pub const Version: RuntimeVersion = VERSION;
	pub RuntimeBlockLength: BlockLength =
		BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
		.base_block(BlockExecutionWeight::get())
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = ExtrinsicBaseWeight::get();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();
	pub const SS58Prefix: u8 = 42;
}

// Configure FRAME pallets to include in runtime.

impl frame_system::Config for Runtime {
	type BaseCallFilter = ();
	type BlockWeights = RuntimeBlockWeights;
	type BlockLength = RuntimeBlockLength;
	type AccountId = AccountId;
	type Call = Call;
	type Lookup = AccountIdLookup<AccountId, ()>;
	type Index = Index;
	type BlockNumber = BlockNumber;
	type Hash = Hash;
	type Hashing = BlakeTwo256;
	type Header = generic::Header<BlockNumber, BlakeTwo256>;
	type Event = Event;
	type Origin = Origin;
	type BlockHashCount = BlockHashCount;
	type DbWeight = RocksDbWeight;
	type Version = Version;
	type PalletInfo = PalletInfo;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type AccountData = pallet_balances::AccountData<Balance>;
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
        type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
}

parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 500;
	pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Runtime {
	type MaxLocks = MaxLocks;
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const TransactionByteFee: Balance = 1;
}

impl pallet_transaction_payment::Config for Runtime {
	type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, ()>;
	type TransactionByteFee = TransactionByteFee;
	type WeightToFee = IdentityFee<Balance>;
	type FeeMultiplierUpdate = ();
}

impl pallet_sudo::Config for Runtime {
	type Event = Event;
	type Call = Call;
}

impl cumulus_pallet_parachain_system::Config for Runtime {
	type SelfParaId = parachain_info::Pallet<Runtime>;
        type Event = Event;
        type OnValidationData = ();
        type DownwardMessageHandlers = ();
        type OutboundXcmpMessageSource = ();
        type XcmpMessageHandler = ();
        type ReservedXcmpWeight = ();
}


impl parachain_info::Config for Runtime {}

parameter_types! {
	pub const SignedClaimHandicap: u32 = 2;
	pub const TombstoneDeposit: u64 = 16;
	pub const DepositPerContract: u64 = 8 * DepositPerStorageByte::get();
	pub const DepositPerStorageByte: u64 = 10_000;
	pub const DepositPerStorageItem: u64 = 10_000;
        pub RentFraction: Perbill = PerThing::from_rational(4u32, 10_000u32);
	pub const SurchargeReward: u64 = 500_000;
	pub const MaxDepth: u32 = 100;
	pub const MaxValueSize: u32 = 16_384;
	pub const DeletionQueueDepth: u32 = 1024;
	pub const DeletionWeightLimit: Weight = 500_000_000_000;
	pub const MaxCodeSize: u32 = 128 * 1024;
}

impl pallet_contracts::Config for Runtime {
	type Time = Timestamp;
	type Randomness = RandomnessCollectiveFlip;
	type Currency = Balances;
	type Event = Event;
	type RentPayment = ();
	type SignedClaimHandicap = SignedClaimHandicap;
	type TombstoneDeposit = TombstoneDeposit;
	type DepositPerContract = DepositPerContract;
	type DepositPerStorageByte = DepositPerStorageByte;
	type DepositPerStorageItem = DepositPerStorageItem;
	type RentFraction = RentFraction;
	type SurchargeReward = SurchargeReward;
	type MaxDepth = MaxDepth;
	type MaxValueSize = MaxValueSize;
	type WeightPrice = ();
	type WeightInfo = ();
	type ChainExtension = ();
	type DeletionQueueDepth = DeletionQueueDepth;
	type DeletionWeightLimit = DeletionWeightLimit;
	type MaxCodeSize = MaxCodeSize;
}

parameter_types! {
        pub const CandidacyBond: Balance = 10 * DOLLARS;
        pub const VotingBondBase: Balance = 1 * DOLLARS;
        pub const VotingBondFactor: Balance = 10 * CENTS;
        pub const TermDuration: BlockNumber = 7 * DAYS;
        pub const DesiredMembers: u32 = 7;
        pub const DesiredRunnersUp: u32 = 7;
        pub const ElectionsPhragmenModuleId: LockIdentifier = *b"phrelect";
}

impl pallet_elections_phragmen::Config for Runtime {
        type Event = Event;
        type PalletId = ElectionsPhragmenModuleId;
        type Currency = Balances;
        type ChangeMembers = ();
        type InitializeMembers = ();
        type CurrencyToVote = U128CurrencyToVote;
        type CandidacyBond = CandidacyBond;
        type VotingBondBase = VotingBondBase;
        type VotingBondFactor = VotingBondFactor;
        type LoserCandidate = ();
        type KickedMember = ();
        type DesiredMembers = DesiredMembers;
        type DesiredRunnersUp = DesiredRunnersUp;
        type TermDuration = TermDuration;
        type WeightInfo = pallet_elections_phragmen::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
        pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) * RuntimeBlockWeights::get().max_block;
        pub const MaxScheduledPerBlock: u32 = 50;
}

impl pallet_scheduler::Config for Runtime {
        type Event = Event;
        type Origin = Origin;
        type PalletsOrigin = OriginCaller;
        type Call = Call;
        type MaximumWeight = MaximumSchedulerWeight;
        type ScheduleOrigin = EnsureRoot<AccountId>;
        type MaxScheduledPerBlock = MaxScheduledPerBlock;
        type WeightInfo = ();
}

parameter_types! {
        pub const LaunchPeriod: BlockNumber = 10 * MINUTES;
        pub const VotingPeriod: BlockNumber = 10 * MINUTES;
        pub const FastTrackVotingPeriod: BlockNumber = 5 * MINUTES;
        pub const InstantAllowed: bool = true;
        pub const MinimumDeposit: Balance = 100 * DOLLARS;
        pub const EnactmentPeriod: BlockNumber = 10 * MINUTES;
        pub const CooloffPeriod: BlockNumber = 10 * MINUTES;
        pub const PreimageByteDeposit: Balance = 1 * CENTS;
        pub const MaxVotes: u32 = 100;
        pub const MaxProposals: u32 = 100;
}

impl pallet_democracy::Config for Runtime {
        type Proposal = Call;
        type Event = Event;
        type Currency = Balances;
        type EnactmentPeriod = EnactmentPeriod;
        type LaunchPeriod = LaunchPeriod;
        type VotingPeriod = VotingPeriod;
        type FastTrackVotingPeriod = FastTrackVotingPeriod;
        type MinimumDeposit = MinimumDeposit;
        type ExternalOrigin = EnsureRoot<AccountId>;
        type ExternalMajorityOrigin = EnsureRoot<AccountId>;
        type ExternalDefaultOrigin = EnsureRoot<AccountId>;
        type FastTrackOrigin = EnsureRoot<AccountId>;
        type CancellationOrigin = EnsureRoot<AccountId>;
        type BlacklistOrigin = EnsureRoot<AccountId>;
        type CancelProposalOrigin = EnsureRoot<AccountId>;
        type VetoOrigin = EnsureNever<AccountId>; // (root not possible)
        type CooloffPeriod = CooloffPeriod;
        type PreimageByteDeposit = PreimageByteDeposit;
        type Slash = ();
        type InstantOrigin = EnsureRoot<AccountId>;
        type InstantAllowed = InstantAllowed;
        type Scheduler = Scheduler;
        type MaxVotes = MaxVotes;
        type OperationalPreimageOrigin = EnsureSigned<AccountId>;
        type PalletsOrigin = OriginCaller;
        type WeightInfo = ();
        type MaxProposals = MaxProposals;
}

pub struct FixedGasPrice;
impl FeeCalculator for FixedGasPrice {
        fn min_gas_price() -> U256 {
                // Gas price is always one token per gas.
                0.into()
        }
}

parameter_types! {
        pub const ChainId: u64 = 42;
}

impl pallet_evm::Config for Runtime {
        type FeeCalculator = FixedGasPrice;
        type GasWeightMapping = ();
        type CallOrigin = EnsureAddressTruncated;
        type WithdrawOrigin = EnsureAddressTruncated;
        type AddressMapping = HashedAddressMapping<BlakeTwo256>;
        type Currency = Balances;
        type Event = Event;
        type Runner = pallet_evm::runner::stack::Runner<Self>;
        type Precompiles = (
                pallet_evm_precompile_simple::ECRecover,
                pallet_evm_precompile_simple::Sha256,
                pallet_evm_precompile_simple::Ripemd160,
                pallet_evm_precompile_simple::Identity,
        );
        type ChainId = ChainId;
        type OnChargeTransaction = ();
}

pub struct TransactionConverter;
impl fp_rpc::ConvertTransaction<UncheckedExtrinsic> for TransactionConverter {
        fn convert_transaction(&self, transaction: pallet_ethereum::Transaction) -> UncheckedExtrinsic {
                UncheckedExtrinsic::new_unsigned(
                        pallet_ethereum::Call::<Runtime>::transact(transaction).into()
                )
        }
}

impl fp_rpc::ConvertTransaction<opaque::UncheckedExtrinsic>
        for TransactionConverter
{
        fn convert_transaction(
                &self,
                transaction: pallet_ethereum::Transaction,
        ) -> opaque::UncheckedExtrinsic {
                let extrinsic = UncheckedExtrinsic::new_unsigned(
                        pallet_ethereum::Call::<Runtime>::transact(transaction).into(),
                );
                let encoded = extrinsic.encode();
                opaque::UncheckedExtrinsic::decode(&mut &encoded[..])
                        .expect("Encoded extrinsic is always valid")
        }
}

// Consensus not supported in parachain
pub struct EthereumFindAuthor<F>(PhantomData<F>);
impl<F: FindAuthor<u32>> FindAuthor<H160> for EthereumFindAuthor<F>
{
        fn find_author<'a, I>(_digests: I) -> Option<H160>
        where
                I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
        {
                None
        }
}

pub struct PhantomAura;
impl FindAuthor<u32> for PhantomAura {
        fn find_author<'a, I>(_digests: I) -> Option<u32>
        where
                I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
        {
                Some(0 as u32)
        }
}

parameter_types! {
        pub BlockGasLimit: U256 = U256::from(u32::max_value());
}

impl pallet_ethereum::Config for Runtime {
        type Event = Event;
        type FindAuthor = EthereumFindAuthor<PhantomAura>;
        type StateRoot = pallet_ethereum::IntermediateStateRoot;
        type BlockGasLimit = BlockGasLimit;
}

parameter_types! {
	pub const RococoLocation: MultiLocation = MultiLocation::X1(Junction::Parent);
	pub const RococoNetwork: NetworkId = NetworkId::Polkadot;
	pub Ancestry: MultiLocation = Junction::Parachain {
		id: ParachainInfo::parachain_id().into()
	}.into();
}

impl cumulus_pallet_xcm::Config for Runtime {}

impl template::Config for Runtime {
	type Event = Event;
}

pub type SignedPayload = generic::SignedPayload<Call, SignedExtra>;

parameter_types! {
        pub const GracePeriod: u32 = 5;
        pub const UnsignedInterval: u32 = 2;
        pub const UnsignedPriority: u32 = 1 << 20;
}

impl gateway::Config for Runtime {
        type Event = Event;
        type AuthorityId = gateway::crypto::TestAuthId;
        type Call = Call;
        type GracePeriod = GracePeriod;
        type UnsignedInterval = UnsignedInterval;
        type UnsignedPriority = UnsignedPriority;
}

impl frame_system::offchain::SigningTypes for Runtime {
        type Public = <Signature as Verify>::Signer;
        type Signature = Signature;
}

impl<LocalCall> frame_system::offchain::SendTransactionTypes<LocalCall> for Runtime 
where
        Call: From<LocalCall>,
{
        type OverarchingCall = Call;
        type Extrinsic = UncheckedExtrinsic;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Runtime
where
        Call: From<LocalCall>,
{
        fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
                call: Call,
                public: <Signature as sp_runtime::traits::Verify>::Signer,
                account: AccountId,
                index: Index,
        ) -> Option<(Call, <UncheckedExtrinsic as sp_runtime::traits::Extrinsic>::SignaturePayload)> 
        {
                let period = BlockHashCount::get() as u64;
                let current_block = System::block_number()
                        .saturated_into::<u64>()
                        .saturating_sub(1);
                let tip = 0;
                let extra: SignedExtra = (
                        frame_system::CheckSpecVersion::<Runtime>::new(),
                        frame_system::CheckTxVersion::<Runtime>::new(),
                        frame_system::CheckGenesis::<Runtime>::new(),
                        frame_system::CheckEra::<Runtime>::from(generic::Era::mortal(period, current_block)),
                        frame_system::CheckNonce::<Runtime>::from(index),
                        frame_system::CheckWeight::<Runtime>::new(),
                        pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
                );

                let raw_payload = SignedPayload::new(call, extra).ok()?;
                let signature = raw_payload.using_encoded(|payload| C::sign(payload, public))?;

                let address = sp_runtime::MultiAddress::Id(account);
                let (call, extra, _) = raw_payload.deconstruct();
                Some((call, (address, signature, extra)))
        }
}

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = opaque::Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Sudo: pallet_sudo::{Pallet, Call, Storage, Config<T>, Event<T>},
		RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Call, Storage},
		ParachainSystem: cumulus_pallet_parachain_system::{Pallet, Call, Storage, Inherent, Event<T>},
		TransactionPayment: pallet_transaction_payment::{Pallet, Storage},
		ParachainInfo: parachain_info::{Pallet, Storage, Config},
		TemplateModule: template::{Pallet, Call, Storage, Event<T>},

		Gateway: gateway::{Pallet, Call, Storage, Event<T>, ValidateUnsigned},
                Scheduler: pallet_scheduler::{Pallet, Storage, Config, Event<T>, Call},
                Democracy: pallet_democracy::{Pallet, Storage, Config, Event<T>, Call},
                EVM: pallet_evm::{Pallet, Config, Call, Storage, Event<T>},
                Ethereum: pallet_ethereum::{Pallet, Call, Storage, Event, Config, ValidateUnsigned},
                Contracts: pallet_contracts::{Pallet, Call, Config<T>, Storage, Event<T>},
                Elections: pallet_elections_phragmen::{Pallet, Call, Storage, Event<T>, Config<T>},

	}
);

pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
pub type SignedBlock = generic::SignedBlock<Block>;
pub type BlockId = generic::BlockId<Block>;
pub type SignedExtra = (
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>
);
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPallets,
>;

impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block)
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			Runtime::metadata().into()
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}
		fn random_seed() -> <Block as BlockT>::Hash {
			RandomnessCollectiveFlip::random_seed().0
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx)
		}
	}
	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			opaque::SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
		fn account_nonce(account: AccountId) -> Index {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
	}

        impl fp_rpc::EthereumRuntimeRPCApi<Block> for Runtime {
                fn chain_id() -> u64 {
                        <Runtime as pallet_evm::Config>::ChainId::get()
                }

                fn account_basic(address: H160) -> EVMAccount {
                        EVM::account_basic(&address)
                }

                fn gas_price() -> U256 {
                        <Runtime as pallet_evm::Config>::FeeCalculator::min_gas_price()
                }

                fn account_code_at(address: H160) -> Vec<u8> {
                        EVM::account_codes(address)
                }

                fn author() -> H160 {
                        <pallet_ethereum::Pallet<Runtime>>::find_author()
                }

                fn storage_at(address: H160, index: U256) -> H256 {
                        let mut tmp = [0u8; 32];
                        index.to_big_endian(&mut tmp);
                        EVM::account_storages(address, H256::from_slice(&tmp[..]))
                }

                fn call(
                        from: H160,
                        to: H160,
                        data: Vec<u8>,
                        value: U256,
                        gas_limit: U256,
                        gas_price: Option<U256>,
                        nonce: Option<U256>,
                        estimate: bool,
                ) -> Result<pallet_evm::CallInfo, sp_runtime::DispatchError> {
                        let config = if estimate {
                                let mut config = <Runtime as pallet_evm::Config>::config().clone();
                                config.estimate = true;
                                Some(config)
                        } else {
                                None
                        };

                        <Runtime as pallet_evm::Config>::Runner::call(
                                from,
                                to,
                                data,
                                value,
                                gas_limit.low_u64(),
                                gas_price,
                                nonce,
                                config.as_ref().unwrap_or(<Runtime as pallet_evm::Config>::config()),
                        ).map_err(|err| err.into())
                }

                fn create(
                        from: H160,
                        data: Vec<u8>,
                        value: U256,
                        gas_limit: U256,
                        gas_price: Option<U256>,
                        nonce: Option<U256>,
                        estimate: bool,
                ) -> Result<pallet_evm::CreateInfo, sp_runtime::DispatchError> {
                        let config = if estimate {
                                let mut config = <Runtime as pallet_evm::Config>::config().clone();
                                config.estimate = true;
                                Some(config)
                        } else {
                                None
                        };

                        <Runtime as pallet_evm::Config>::Runner::create(
                                from,
                                data,
                                value,
                                gas_limit.low_u64(),
                                gas_price,
                                nonce,
                                config.as_ref().unwrap_or(<Runtime as pallet_evm::Config>::config()),
                        ).map_err(|err| err.into())
                }

                fn current_transaction_statuses() -> Option<Vec<TransactionStatus>> {
                        Ethereum::current_transaction_statuses()
                }

                fn current_block() -> Option<pallet_ethereum::Block> {
                        Ethereum::current_block()
                }

                fn current_receipts() -> Option<Vec<pallet_ethereum::Receipt>> {
                        Ethereum::current_receipts()
                }

                fn current_all() -> (
                        Option<pallet_ethereum::Block>,
                        Option<Vec<pallet_ethereum::Receipt>>,
                        Option<Vec<TransactionStatus>>
                ) {
                        (
                                Ethereum::current_block(),
                                Ethereum::current_receipts(),
                                Ethereum::current_transaction_statuses()
                        )
                }
        }

        impl pallet_contracts_rpc_runtime_api::ContractsApi<Block, AccountId, Balance, BlockNumber>
                for Runtime
        {
                fn call(
                        origin: AccountId,
                        dest: AccountId,
                        value: Balance,
                        gas_limit: u64,
                        input_data: Vec<u8>,
                ) -> pallet_contracts_primitives::ContractExecResult {
                        Contracts::bare_call(origin, dest, value, gas_limit, input_data)
                }

                fn get_storage(
                        address: AccountId,
                        key: [u8; 32],
                ) -> pallet_contracts_primitives::GetStorageResult {
                        Contracts::get_storage(address, key)
                }

                fn rent_projection(
                        address: AccountId,
                ) -> pallet_contracts_primitives::RentProjectionResult<BlockNumber> {
                        Contracts::rent_projection(address)
                }
        }

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{Benchmarking, BenchmarkBatch, add_benchmark, TrackedStorageKey};

			use frame_system_benchmarking::Pallet as SystemBench;
			impl frame_system_benchmarking::Config for Runtime {}

			let whitelist: Vec<TrackedStorageKey> = vec![
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac").to_vec().into(),
				hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").to_vec().into(),
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").to_vec().into(),
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec().into(),
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
			];

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);

			add_benchmark!(params, batches, frame_system, SystemBench::<Runtime>);
			add_benchmark!(params, batches, pallet_balances, Balances);
			add_benchmark!(params, batches, pallet_timestamp, Timestamp);

			if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
			Ok(batches)
		}
	}
}

cumulus_pallet_parachain_system::register_validate_block!(Runtime, Executive);
