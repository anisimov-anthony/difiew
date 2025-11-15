use crate::node::MyBehaviour;
use crate::utils::bin_args::BinArgs;
use libp2p::{gossipsub, identity, mdns, noise, tcp, yamux, PeerId, SwarmBuilder};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    time::Duration,
};

pub fn build_swarm(
    key: identity::Keypair,
    args: &BinArgs,
) -> Result<libp2p::Swarm<MyBehaviour>, Box<dyn std::error::Error>> {
    let peer_id = PeerId::from(key.public());

    let message_id_fn = |msg: &gossipsub::Message| {
        let mut h = DefaultHasher::new();
        msg.data.hash(&mut h);
        gossipsub::MessageId::from(h.finish().to_string())
    };

    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(args.heartbeat_interval))
        .validation_mode(gossipsub::ValidationMode::Strict)
        .message_id_fn(message_id_fn)
        .build()
        .map_err(std::io::Error::other)?;

    let gossipsub = gossipsub::Behaviour::new(
        gossipsub::MessageAuthenticity::Signed(key.clone()),
        gossipsub_config,
    )?;

    let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id)?;

    Ok(SwarmBuilder::with_existing_identity(key)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_quic()
        .with_behaviour(|_| MyBehaviour { gossipsub, mdns })?
        .build())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::bin_args::BinArgs;
    use libp2p::{identity, PeerId};

    #[tokio::test]
    async fn build_swarm_successfully_creates_swarm() {
        let key = identity::Keypair::generate_ed25519();
        let args = BinArgs {
            heartbeat_interval: 10,
            ..Default::default()
        };

        let result = build_swarm(key.clone(), &args);
        assert!(result.is_ok());

        let swarm = result.unwrap();
        assert_eq!(*swarm.local_peer_id(), PeerId::from(key.public()));
    }
}
