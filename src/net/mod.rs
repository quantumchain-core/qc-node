use chrono::Utc;
use libp2p::{
    futures::StreamExt,
    gossipsub, identity, mdns, noise, tcp, yamux,
    swarm::{NetworkBehaviour, SwarmEvent},
    PeerId, SwarmBuilder,
};
use std::time::Duration;
use tokio::time;

use crate::crypto::dilithium::RAMPURA_TESTNET_0_CHAIN_ID;

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "QtcBehaviourEvent")]
pub struct QtcBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
}

pub enum QtcBehaviourEvent {
    Gossipsub(gossipsub::Event),
    Mdns(mdns::Event),
}

impl From<gossipsub::Event> for QtcBehaviourEvent {
    fn from(event: gossipsub::Event) -> Self {
        QtcBehaviourEvent::Gossipsub(event)
    }
}

impl From<mdns::Event> for QtcBehaviourEvent {
    fn from(event: mdns::Event) -> Self {
        QtcBehaviourEvent::Mdns(event)
    }
}

pub async fn run_node() -> Result<(), Box<dyn std::error::Error>> {
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    log::info!("Local peer id: {local_peer_id}");

    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(10))
        .validation_mode(gossipsub::ValidationMode::Strict)
        .build()
        .expect("Valid config");

    let mut gossipsub = gossipsub::Behaviour::new(
        gossipsub::MessageAuthenticity::Signed(local_key.clone()),
        gossipsub_config,
    )
    .expect("Correct Gossipsub instantiation");

    let topic = gossipsub::IdentTopic::new(RAMPURA_TESTNET_0_CHAIN_ID);
    gossipsub.subscribe(&topic)?;

    let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), local_peer_id)?;
    let behaviour = QtcBehaviour { gossipsub, mdns };

    let mut swarm = SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|_| behaviour)?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    let mut tick = time::interval(Duration::from_secs(5));

    loop {
        tokio::select! {
            _ = tick.tick() => {
                let block_data = format!("block-{}-{}", RAMPURA_TESTNET_0_CHAIN_ID, Utc::now().timestamp());
                if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic.clone(), block_data.as_bytes()) {
                    log::error!("Publish error: {e:?}");
                }
            }
            event = swarm.select_next_some() => {
                match event {
                    SwarmEvent::Behaviour(QtcBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                        propagation_source: peer_id,
                        message_id: id,
                        message,
                    })) => {
                        log::info!(
                            "Got message: '{}' with id: {id} from peer: {peer_id}",
                            String::from_utf8_lossy(&message.data),
                        );
                    }
                    SwarmEvent::Behaviour(QtcBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                        for (peer_id, multiaddr) in list {
                            log::info!("mDNS discovered a new peer: {peer_id}");
                            swarm.dial(multiaddr).ok();
                        }
                    }
                    SwarmEvent::NewListenAddr { address, .. } => {
                        log::info!("Listening on {address}");
                    }
                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        log::info!("Connected to {peer_id}");
                    }
                    SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                        log::info!("Disconnected from {peer_id}: {cause:?}");
                    }
                    _ => {}
                }
            }
        }
    }
}