use crate::node::majority_tracker::*;
use crate::store::command::*;
use crate::store::result::*;
use bincode::{Decode, Encode};
use std::collections::HashMap;

pub mod metadata;
use metadata::*;

#[derive(Decode, Encode, Debug)]
pub enum ComponentMessage<'m, 'n> {
    ManagerMessage(ManagerMessage<'m>, MetaData),
    NodeMessage(NodeMessage<'n>, MetaData),
}

#[derive(Decode, Encode, Debug)]
pub enum ManagerMessage<'a> {
    StoreCommand(StoreCommand<'a>),
}

#[derive(Decode, Encode, Debug)]
pub enum NodeMessage<'a> {
    StoreCommandResult(StoreCommandResult<'a>),

    ShareSignature(ShareSignatureParams),

    RepairRequest(RepairRequestParams),

    // WARN: for the first time all data will be sent, without batching and etc
    RepairResponse(RepairResponseParams),
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct ShareSignatureParams {
    pub src_id: String,
    pub sgn: Signature,
}

impl ShareSignatureParams {
    pub fn new(src_id: String, sgn: Signature) -> Self {
        Self { src_id, sgn }
    }
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct RepairRequestParams {
    pub src_id: String,
    pub dst_id: String,
}

impl RepairRequestParams {
    pub fn new(src_id: String, dst_id: String) -> Self {
        Self { src_id, dst_id }
    }
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct RepairResponseParams {
    pub src_id: String,
    pub dst_id: String,
    pub repaired_data: HashMap<String, String>,
}

impl RepairResponseParams {
    pub fn new(src_id: String, dst_id: String, repaired_data: HashMap<String, String>) -> Self {
        Self {
            src_id,
            dst_id,
            repaired_data,
        }
    }
}
