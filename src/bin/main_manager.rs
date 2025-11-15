use clap::Parser;
use difiew::{
    manager::Manager,
    utils::{bin_args::BinArgs, swarm_builder::build_swarm},
    Component,
};
use libp2p::{identity, Multiaddr, PeerId};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = BinArgs::parse();
    let key = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(key.public());
    println!("Manager peer id: {}", peer_id);

    let mut swarm = build_swarm(key, &args)?;
    let topic = libp2p::gossipsub::IdentTopic::new(&args.topic);
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

    let tcp_addr: Multiaddr = args.tcp_listen.parse()?;
    swarm.listen_on(tcp_addr)?;

    let mut manager = Manager::new(swarm, peer_id, topic);
    manager.start_event_loop().await;
    Ok(())
}
