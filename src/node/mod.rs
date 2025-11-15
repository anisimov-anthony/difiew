use bincode::config;
use futures::stream::StreamExt;
use libp2p::{
    gossipsub::{self, IdentTopic},
    mdns,
    swarm::SwarmEvent,
    PeerId, Swarm,
};
use std::cell::RefCell;
use std::time::Duration;
use tokio::select;

pub use crate::MyBehaviour;
pub use crate::MyBehaviourEvent;

use crate::{
    node::majority_tracker::{MajorityTracker, Signature},
    protocol::{
        metadata::MetaData, ComponentMessage, ManagerMessage, NodeMessage, RepairRequestParams,
        RepairResponseParams, ShareSignatureParams,
    },
    store::*,
    utils::timestamp::timestamp_millis,
    Component, ComponentCore, ComponentError,
};

pub mod majority_tracker;

#[allow(dead_code)]
pub struct Node {
    core: ComponentCore,
    storage: RefCell<Store>,
    tracker: MajorityTracker,
}

#[allow(dead_code)]
impl Node {
    pub fn new(swarm: Swarm<MyBehaviour>, peer_id: PeerId, topic: IdentTopic) -> Self {
        Self {
            core: ComponentCore {
                swarm: swarm.into(),
                peer_id,
                topic,
                config: config::standard(),
            },
            storage: Store::new().into(),
            tracker: MajorityTracker::new(),
        }
    }

    fn generate_signature(&self) -> Result<Signature, ComponentError> {
        let root = self.storage.borrow().reveal_root();
        let local_timestamp = timestamp_millis().ok_or(ComponentError::Timestamp())?;

        Ok(Signature {
            root,
            local_timestamp,
        })
    }

    fn share_signature(&mut self) -> Result<(), ComponentError> {
        let signature = self.generate_signature()?;
        let timestamp = timestamp_millis().ok_or(ComponentError::Timestamp())?;

        let metadata = MetaData::new(self.core.peer_id, timestamp);
        let params = ShareSignatureParams::new(self.core.peer_id.to_string(), signature);
        let msg = ComponentMessage::NodeMessage(NodeMessage::ShareSignature(params), metadata);

        self.publish_message(msg)
    }

    fn handle_manager_message_and_publish(
        &self,
        msg: ManagerMessage,
    ) -> Result<(), ComponentError> {
        println!("handling ManagerMessage: {:?}", msg);
        let mut binding = self.storage.borrow_mut();
        let message = match msg {
            ManagerMessage::StoreCommand(cmd) => {
                let cmd_result = binding.execute(cmd)?;

                let timestamp = timestamp_millis().ok_or(ComponentError::Timestamp())?;
                let metadata = MetaData::new(self.core.peer_id, timestamp);

                ComponentMessage::NodeMessage(NodeMessage::StoreCommandResult(cmd_result), metadata)
            }
        };

        self.publish_message(message)?;
        Ok(())
    }

    fn handle_node_message(&mut self, msg: NodeMessage) -> Result<(), ComponentError> {
        match msg {
            NodeMessage::ShareSignature(params) => {
                let signature = params.sgn;
                let src_id = params.src_id;
                self.tracker
                    .update_signature(src_id.clone(), signature.clone());

                if self.generate_signature()?.root != signature.root 
                    && let Some(majority) = self.tracker.truthful_majority() {
                        for peer_id in majority {
                            let body = RepairRequestParams::new(
                                self.core.peer_id.to_string(),
                                peer_id.to_string(),
                            );

                            let timestamp = timestamp_millis().ok_or(ComponentError::Timestamp())?;
                            let metadata = MetaData::new(self.core.peer_id, timestamp);

                            let msg = ComponentMessage::NodeMessage(
                                NodeMessage::RepairRequest(body.clone()),
                                metadata,
                            );
                            self.publish_message(msg)?;
                        }
                    }
                Ok(())
            }
            NodeMessage::RepairRequest(params) => {
                let dst = params.dst_id;
                let src = params.src_id;
                if dst == self.core.peer_id.to_string() {
                    let body = RepairResponseParams::new(
                        dst.clone(),
                        src,
                        self.storage.borrow().get_main_store(),
                    );

                    let timestamp = timestamp_millis().ok_or(ComponentError::Timestamp())?;
                    let metadata = MetaData::new(self.core.peer_id, timestamp);

                    let msg = ComponentMessage::NodeMessage(
                        NodeMessage::RepairResponse(body.clone()),
                        metadata,
                    );
                    self.publish_message(msg)?;
                }
                Ok(())
            }
            NodeMessage::RepairResponse(params) => {
                let src = params.src_id;
                let dst = params.dst_id;
                let data = params.repaired_data;
                if dst == self.core.peer_id.to_string() {
                    let _ = self.storage.borrow_mut().update_full_store(data);
                    println!("peer {dst} received a response from peer {src} and replaced the data with new ones");
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl Component for Node {
    fn core(&self) -> &ComponentCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut ComponentCore {
        &mut self.core
    }

    async fn start_event_loop<'a>(&'a mut self) {
        let mut share_signature_stream = tokio::time::interval(Duration::from_secs(1));

        loop {
            let mut swarm_guard = self.core.swarm.borrow_mut();
            select! {
                        event = swarm_guard.select_next_some() => {
                            match event {
                                SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                                    for (peer_id, _multiaddr) in list {
                                        println!("mDNS discovered a new peer: {peer_id}");
                                        swarm_guard.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                                    }
                                }
                                SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                                    for (peer_id, _multiaddr) in list {
                                        println!("mDNS discover peer has expired: {peer_id}");
                                        swarm_guard.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                                    }
                                }
                                SwarmEvent::Behaviour(MyBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                                    propagation_source: _peer_id,
                                    message_id: _id,
                                    message,
                                })) => {
                                    drop(swarm_guard);

                                    let (decoded, _len): (ComponentMessage, usize) = match bincode::decode_from_slice(&message.data[..], self.core.config) {
                                        Ok(v) => v,
                                        Err(e) => {
                                            eprintln!("Failed to decode message: {e}");
                                            continue;
                                        }
                                    };

                                    match decoded {
                                        ComponentMessage::ManagerMessage(mng_msg, _) => {
                                    let _ = self.handle_manager_message_and_publish(mng_msg);

            }
            ComponentMessage::NodeMessage(nd_msg, _) => {
                if let Err(e) = self.handle_node_message(nd_msg) {
                    eprintln!("Failed to handle node message: {e}");
                }
            }
                            }

                                }
                                SwarmEvent::NewListenAddr { address, .. } => {
                                    println!("Local node is listening on {address}");

                                }
                                _ => {

                        }
                            }
                        }
                        _ = share_signature_stream.tick() => {
                            drop(swarm_guard);
                            if let Err(e) = self.share_signature() {
                                eprintln!("Failed to share signature: {e}");
                            }
                        }
                    }
        }
    }
}
