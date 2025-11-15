use bincode::config;
use futures::stream::StreamExt;
use libp2p::{
    gossipsub::{self, IdentTopic},
    mdns,
    swarm::SwarmEvent,
    PeerId, Swarm,
};
use tokio::{io, io::AsyncBufReadExt, select};

use crate::{utils::timestamp::timestamp_millis, Component, ComponentCore, ComponentError};

use crate::node::{MyBehaviour, MyBehaviourEvent};
use crate::protocol::{metadata::MetaData, ComponentMessage, ManagerMessage, NodeMessage};
use crate::store::command::{handle_cmd_input, CmdArgs};

pub struct Manager {
    core: ComponentCore,
}

#[allow(dead_code)]
impl Manager {
    pub fn new(swarm: Swarm<MyBehaviour>, peer_id: PeerId, topic: IdentTopic) -> Self {
        Self {
            core: ComponentCore {
                swarm: swarm.into(),
                peer_id,
                topic,
                config: config::standard(),
            },
        }
    }

    pub fn execute_user_input(&mut self, args: CmdArgs) -> Result<(), ComponentError> {
        let store_cmd = handle_cmd_input(&args).ok_or(ComponentError::InvalidInput())?;

        let timestamp = timestamp_millis().ok_or(ComponentError::Timestamp())?;

        let metadata = MetaData::new(self.core.peer_id, timestamp);

        let msg =
            ComponentMessage::ManagerMessage(ManagerMessage::StoreCommand(store_cmd), metadata);

        println!("Compmsg: {:?}", msg);

        self.publish_message(msg)
            .map_err(|e| ComponentError::Publish(e.to_string()))
    }
}

#[async_trait::async_trait(?Send)]
impl Component for Manager {
    fn core(&self) -> &ComponentCore {
        &self.core
    }
    fn core_mut(&mut self) -> &mut ComponentCore {
        &mut self.core
    }

    async fn start_event_loop<'a>(&'a mut self) {
        let (tx, mut rx) = tokio::sync::mpsc::channel(32);

        let input_handle = tokio::spawn({
            let tx = tx.clone();
            async move {
                let stdin = io::stdin();
                let mut reader = io::BufReader::new(stdin).lines();

                loop {
                    match reader.next_line().await {
                        Ok(Some(line)) => {
                            let line = line.trim();
                            if line.is_empty() {
                                continue;
                            }

                            let parts: Vec<&str> = line.splitn(2, ' ').collect();
                            match parts.as_slice() {
                                [cmd_type, cmd_arg] => {
                                    let args = CmdArgs {
                                        cmd_type: cmd_type.to_string(),
                                        cmd_arg: cmd_arg.to_string(),
                                    };
                                    if tx.send(args).await.is_err() {
                                        break;
                                    }
                                }
                                [cmd_type] => {
                                    println!(
                                        "Invalid command: missing argument for '{}'",
                                        cmd_type
                                    );
                                }
                                [] => {
                                    continue;
                                }
                                &[_, _, _, ..] => println!("unhandled case"),
                            }
                        }
                        Ok(None) => break, // EOF
                        Err(e) => {
                            eprintln!("Input error: {e}");
                            break;
                        }
                    }
                }
            }
        });

        loop {
            let mut swarm_guard = self.core.swarm.borrow_mut();
            select! {
                event = swarm_guard.select_next_some() => match event {
                    SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                                                drop(swarm_guard);

                        for (peer_id, _multiaddr) in list {
                            println!("mDNS discovered a new peer: {peer_id}");
                            self.core.swarm.borrow_mut().behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                        }
                    },
                    SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                                                drop(swarm_guard);

                        for (peer_id, _multiaddr) in list {
                            println!("mDNS discover peer has expired: {peer_id}");
                            self.core.swarm.borrow_mut().behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                        }
                    },
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Local node is listening on {address}");
                    }

                    SwarmEvent::Behaviour(MyBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                        propagation_source: _peer_id,
                        message_id: _id,
                        message,
                    })) => {
                        if let Ok((decoded, _len)) = bincode::decode_from_slice(&message.data[..], self.core.config) {
                            drop(swarm_guard);


                                if let ComponentMessage::NodeMessage(NodeMessage::StoreCommandResult(result), _) = decoded {
                                println!("manager got {:?}", result)}


                            }
                    }

                    _ => {}
                },

                args = rx.recv() => {
                    if let Some(args) = args {
                        drop(swarm_guard);
                        _ = self.execute_user_input(args);
                    } else {
                        break;
                    }
                }
            }
        }

        let _ = input_handle.await;
    }
}
