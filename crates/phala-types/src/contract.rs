use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use codec::{Decode, Encode};
use scale_info::TypeInfo;

pub use phala_mq::ContractId;

pub type ContractId32 = u32;
pub const SYSTEM: ContractId32 = 0;
pub const DATA_PLAZA: ContractId32 = 1;
pub const BALANCES: ContractId32 = 2;
pub const ASSETS: ContractId32 = 3;
pub const WEB3_ANALYTICS: ContractId32 = 4;
pub const DIEM: ContractId32 = 5;
pub const SUBSTRATE_KITTIES: ContractId32 = 6;
pub const BTC_LOTTERY: ContractId32 = 7;
pub const GEOLOCATION: ContractId32 = 8;
pub const GUESS_NUMBER: ContractId32 = 100;
pub const BTC_PRICE_BOT: ContractId32 = 101;

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub enum CodeIndex<CodeHash> {
    NativeCode(ContractId32),
    WasmCode(CodeHash),
}

pub mod messaging {
    use alloc::vec::Vec;
    use codec::{Decode, Encode};

    use super::ContractInfo;
    use crate::{messaging::ContractClusterId, EcdhPublicKey, WorkerIdentity, WorkerPublicKey};
    use phala_mq::bind_topic;

    bind_topic!(ContractEvent<CodeHash, AccountId>, b"phala/contract/event");
    #[derive(Encode, Decode, Debug)]
    pub enum ContractEvent<CodeHash, AccountId> {
        // TODO.shelven: enable add and remove workers
        InstantiateCode {
            contract_info: ContractInfo<CodeHash, AccountId>,
            deploy_workers: Vec<WorkerIdentity>,
        },
    }

    impl<CodeHash, AccountId> ContractEvent<CodeHash, AccountId> {
        pub fn instantiate_code(
            contract_info: ContractInfo<CodeHash, AccountId>,
            deploy_workers: Vec<WorkerIdentity>,
        ) -> Self {
            ContractEvent::InstantiateCode {
                contract_info,
                deploy_workers,
            }
        }
    }

    bind_topic!(ContractOperation<AccountId>, b"phala/contract/op");
    #[derive(Encode, Decode, Debug)]
    pub enum ContractOperation<AccountId> {
        UploadCodeToCluster {
            origin: AccountId,
            code: Vec<u8>,
            cluster_id: ContractClusterId,
        },
    }
}

/// On-chain contract registration info
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub struct ContractInfo<CodeHash, AccountId> {
    pub deployer: AccountId,
    pub code_index: CodeIndex<CodeHash>,
    pub salt: Vec<u8>,
    /// Contract cluster counter of the contract
    pub cluster_id: u64,
    pub instantiate_data: Vec<u8>,
}

impl<CodeHash, AccountId> ContractInfo<CodeHash, AccountId> {
    pub fn contract_id(&self) -> ContractId {
        // TODO.shelven: calculate the real contract id
        id256(0)
    }
}

/// Contract query request parameters, to be encrypted.
#[derive(Encode, Decode, Debug)]
pub struct ContractQuery<Data> {
    pub head: ContractQueryHead,
    /// The request data.
    pub data: Data,
}

/// Contract query head
#[derive(Encode, Decode, Debug)]
pub struct ContractQueryHead {
    /// The contract id.
    pub id: ContractId,
    /// A random byte array generated by the client.
    pub nonce: [u8; 32],
}

/// Contract query response, to be encrypted.
#[derive(Encode, Decode, Debug)]
pub struct ContractQueryResponse<Data> {
    /// The nonce from the client.
    pub nonce: [u8; 32],
    /// The query result.
    pub result: Data,
}

pub struct Data(pub Vec<u8>);

impl Encode for Data {
    fn size_hint(&self) -> usize {
        self.0.len()
    }
    fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
        dest.write(&self.0)
    }
}

/// Contract query error define
#[derive(Encode, Decode, Debug)]
pub enum ContractQueryError {
    /// Signature is invalid.
    InvalidSignature,
    /// No such contract.
    ContractNotFound,
    /// Unable to decode the request data.
    DecodeError,
    /// Other errors reported during the contract query execution.
    OtherError(String),
}

impl From<ContractQueryError> for prpc::server::Error {
    fn from(err: ContractQueryError) -> Self {
        Self::ContractQueryError(alloc::format!("{:?}", err))
    }
}

pub fn command_topic(id: ContractId) -> Vec<u8> {
    format!("phala/contract/{}/command", hex::encode(&id))
        .as_bytes()
        .to_vec()
}
