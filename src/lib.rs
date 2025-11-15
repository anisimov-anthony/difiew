pub mod manager;
pub mod node;
pub mod protocol;
pub mod store;
pub mod utils;

use libp2p::{
    gossipsub::{self, IdentTopic},
    mdns,
    swarm::{NetworkBehaviour, Swarm},
    PeerId,
};

use bincode::config::Configuration;
use bincode::error::DecodeError;
use bincode::error::EncodeError;
use std::cell::RefCell;

use protocol::ComponentMessage;
use store::error::StoreError;

#[derive(NetworkBehaviour)]
pub struct MyBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
}

pub struct ComponentCore {
    pub swarm: RefCell<Swarm<MyBehaviour>>,
    pub peer_id: PeerId,
    pub topic: IdentTopic,
    pub config: Configuration,
}

#[async_trait::async_trait(?Send)]
pub trait Component {
    fn core(&self) -> &ComponentCore;
    fn core_mut(&mut self) -> &mut ComponentCore;

    async fn start_event_loop(&mut self);

    fn publish_message(&self, msg: ComponentMessage) -> Result<(), ComponentError> {
        let core = self.core();
        let data = bincode::encode_to_vec(&msg, core.config)?;
        let topic = core.topic.clone();
        core.swarm
            .borrow_mut()
            .behaviour_mut()
            .gossipsub
            .publish(topic, data)
            .map_err(|e| ComponentError::Publish(e.to_string()))?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum ComponentError {
    Store(StoreError), // only for node
    Decode(DecodeError),
    Encode(EncodeError),
    Publish(String),
    Timestamp(),
    InvalidInput(), // only for manager
}

impl From<StoreError> for ComponentError {
    fn from(err: StoreError) -> Self {
        ComponentError::Store(err)
    }
}

impl From<DecodeError> for ComponentError {
    fn from(err: DecodeError) -> Self {
        ComponentError::Decode(err)
    }
}

impl From<EncodeError> for ComponentError {
    fn from(err: EncodeError) -> Self {
        ComponentError::Encode(err)
    }
}

impl std::fmt::Display for ComponentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for ComponentError {}
