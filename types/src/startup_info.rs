// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block::{BlockHeader, BlockInfo, BlockNumber};
use anyhow::Result;
use bcs_ext::{BCSCodec, Sample};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_accumulator::accumulator_info::AccumulatorInfo;
use starcoin_crypto::HashValue;
use starcoin_uint::U256;
use starcoin_vm_types::genesis_config::ChainId;
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::fmt::Formatter;
use std::hash::Hash;

/// The info of a chain.
#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug)]
pub struct ChainInfo {
    chain_id: ChainId,
    genesis_hash: HashValue,
    status: ChainStatus,
}

impl ChainInfo {
    pub fn new(chain_id: ChainId, genesis_hash: HashValue, status: ChainStatus) -> Self {
        Self {
            chain_id,
            genesis_hash,
            status,
        }
    }

    pub fn chain_id(&self) -> ChainId {
        self.chain_id
    }

    pub fn genesis_hash(&self) -> HashValue {
        self.genesis_hash
    }

    pub fn status(&self) -> &ChainStatus {
        &self.status
    }

    pub fn update_status(&mut self, status: ChainStatus) {
        self.status = status;
    }

    pub fn head(&self) -> &BlockHeader {
        &self.status.head
    }

    pub fn total_difficulty(&self) -> U256 {
        self.status.info.get_total_difficulty()
    }

    pub fn into_inner(self) -> (ChainId, HashValue, ChainStatus) {
        (self.chain_id, self.genesis_hash, self.status)
    }

    pub fn random() -> Self {
        Self {
            chain_id: ChainId::new(rand::random()),
            genesis_hash: HashValue::random(),
            status: ChainStatus::random(),
        }
    }
}

impl std::default::Default for ChainInfo {
    fn default() -> Self {
        Self {
            chain_id: ChainId::test(),
            genesis_hash: HashValue::default(),
            status: ChainStatus::sample(),
        }
    }
}

impl std::fmt::Display for ChainInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).map_err(|_| std::fmt::Error)?
        )
    }
}

/// The latest status of a chain.
#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct ChainStatus {
    /// Chain head block's header.
    pub head: BlockHeader,
    /// Chain block info
    pub info: BlockInfo,
}

impl ChainStatus {
    pub fn new(head: BlockHeader, info: BlockInfo) -> Self {
        Self { head, info }
    }

    pub fn random() -> Self {
        let head = BlockHeader::random();
        let block_info = BlockInfo::new(
            head.id(),
            U256::from(rand::random::<u64>()),
            AccumulatorInfo::new(
                head.txn_accumulator_root(),
                vec![],
                rand::random::<u64>(),
                rand::random::<u64>(),
            ),
            AccumulatorInfo::new(
                head.block_accumulator_root(),
                vec![],
                head.number().saturating_sub(1),
                rand::random::<u64>(),
            ),
        );
        Self {
            head,
            info: block_info,
        }
    }

    pub fn head(&self) -> &BlockHeader {
        &self.head
    }

    pub fn info(&self) -> &BlockInfo {
        &self.info
    }

    pub fn total_difficulty(&self) -> U256 {
        self.info.total_difficulty
    }

    pub fn into_inner(self) -> (BlockHeader, BlockInfo) {
        (self.head, self.info)
    }
}

impl Sample for ChainStatus {
    fn sample() -> Self {
        Self {
            head: BlockHeader::sample(),
            info: BlockInfo::sample(),
        }
    }
}

#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug)]
pub struct StartupInfo {
    /// main chain head block hash
    pub main: HashValue,
}

impl fmt::Display for StartupInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "StartupInfo {{")?;
        write!(f, "main: {:?},", self.main)?;
        write!(f, "}}")?;
        Ok(())
    }
}

impl StartupInfo {
    pub fn new(main: HashValue) -> Self {
        Self { main }
    }

    pub fn update_main(&mut self, new_head: HashValue) {
        self.main = new_head;
    }

    pub fn get_main(&self) -> &HashValue {
        &self.main
    }
}

impl TryFrom<Vec<u8>> for StartupInfo {
    type Error = anyhow::Error;

    fn try_from(value: Vec<u8>) -> Result<Self> {
        Self::decode(value.as_slice())
    }
}

impl TryInto<Vec<u8>> for StartupInfo {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Vec<u8>> {
        self.encode()
    }
}

#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug)]
pub struct SnapshotRange {
    /// snapshot [start, end] block number
    start: BlockNumber,
    end: BlockNumber,
}

impl fmt::Display for SnapshotRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SnapshotHeight [{}, {}],", self.start, self.end)?;
        Ok(())
    }
}

impl SnapshotRange {
    pub fn new(start: BlockNumber, end: BlockNumber) -> Self {
        Self { start, end }
    }

    pub fn update_range(&mut self, start: BlockNumber, end: BlockNumber) {
        self.start = start;
        self.end = end;
    }

    pub fn get_start(&self) -> BlockNumber {
        self.start
    }

    pub fn get_end(&self) -> BlockNumber {
        self.end
    }
}

impl TryFrom<Vec<u8>> for SnapshotRange {
    type Error = anyhow::Error;

    fn try_from(value: Vec<u8>) -> Result<Self> {
        Self::decode(value.as_slice())
    }
}

impl TryInto<Vec<u8>> for SnapshotRange {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Vec<u8>> {
        self.encode()
    }
}

#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug)]
pub struct BarnardHardFork {
    // [number, ...) block will remove
    number: BlockNumber,
    hash: HashValue,
}

impl BarnardHardFork {
    pub fn new(number: BlockNumber, hash: HashValue) -> Self {
        Self { number, hash }
    }

    pub fn get_number(&self) -> BlockNumber {
        self.number
    }

    pub fn get_hash(&self) -> HashValue {
        self.hash
    }
}

impl TryFrom<Vec<u8>> for BarnardHardFork {
    type Error = anyhow::Error;

    fn try_from(value: Vec<u8>) -> Result<Self> {
        Self::decode(value.as_slice())
    }
}

impl TryInto<Vec<u8>> for BarnardHardFork {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Vec<u8>> {
        self.encode()
    }
}

#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug)]
pub struct DragonHardFork {
    // block whose number is greater than `number` will be purged
    number: BlockNumber,
    hash: HashValue,
}

impl DragonHardFork {
    pub fn new(number: BlockNumber, hash: HashValue) -> Self {
        Self { number, hash }
    }

    pub fn get_number(&self) -> BlockNumber {
        self.number
    }

    pub fn get_hash(&self) -> HashValue {
        self.hash
    }
}

impl TryFrom<Vec<u8>> for DragonHardFork {
    type Error = anyhow::Error;

    fn try_from(value: Vec<u8>) -> Result<Self> {
        Self::decode(value.as_slice())
    }
}

impl TryInto<Vec<u8>> for DragonHardFork {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Vec<u8>> {
        self.encode()
    }
}
