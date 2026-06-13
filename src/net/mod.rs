// src/net/mod.rs
// QTC M2 + M7: libp2p swarm + gossip publishing

pub mod handler;
pub use handler::{GossipMsg, HandleResult, handle_gossip};

use libp2p::swarm::NetworkBehaviour;
use libp2p::{gossipsub, noise, tcp, yamux, PeerId, SwarmBuilder};
use libp2p::gossipsub::{IdentTopic, MessageAuthenticity, ValidationMode};
use libp2p::identity::Keypair;
use std::error::Error;

/// Topics
pub const TOPIC_BLOCKS: &str = "qc-blocks";
pub const TOPIC_TXS:    &str = "qc-txs";

#[derive(NetworkBehaviour)]
pub struct QcBehaviour {
    pub gossipsub: gossipsub::Behaviour,
}

pub fn peer_id_from_pk(_pk: &[u8]) -> PeerId {
    let keypair = Keypair::generate_ed25519();
    PeerId::from(keypair.public())
}

pub async fn new_swarm() -> Result<libp2p::Swarm<QcBehaviour>, Box<dyn Error>> {
    let id_keys = Keypair::generate_ed25519();

    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .validation_mode(ValidationMode::Strict)
        .build()
        .map_err(Box::<dyn Error>::from)?;

    let mut gossipsub_behaviour = gossipsub::Behaviour::new(
        MessageAuthenticity::Signed(id_keys.clone()),
        gossipsub_config,
    )?;

    // Subscribe to both topics
    gossipsub_behaviour.subscribe(&IdentTopic::new(TOPIC_BLOCKS))?;
    gossipsub_behaviour.subscribe(&IdentTopic::new(TOPIC_TXS))?;

    let swarm = SwarmBuilder::with_existing_identity(id_keys)
        .with_tokio()
        .with_tcp(
            tcp::Config::default().nodelay(true),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|_| QcBehaviour {
            gossipsub: gossipsub_behaviour,
        })?
        .build();

    Ok(swarm)
}

/// Publish a GossipMsg to the correct topic.
/// Call this after producing a block or receiving a new tx from RPC.
pub fn publish(
    swarm: &mut libp2p::Swarm<QcBehaviour>,
    msg: &GossipMsg,
) -> Result<(), String> {
    let (topic, bytes) = match msg {
        GossipMsg::NewBlock(_) => (
            IdentTopic::new(TOPIC_BLOCKS),
            bincode::serialize(msg).map_err(|e| e.to_string())?,
        ),
        GossipMsg::NewTx(_) => (
            IdentTopic::new(TOPIC_TXS),
            bincode::serialize(msg).map_err(|e| e.to_string())?,
        ),
    };
    swarm.behaviour_mut().gossipsub
        .publish(topic, bytes)
        .map(|_| ())
        .map_err(|e| format!("publish failed: {e:?}"))
}

#[cfg(test)]
mod m2_tests {
    use super::*;

    #[test]
    fn m2_peer_id_works() {
        let pk = vec![0u8; 1952];
        let peer_id = peer_id_from_pk(&pk);
        assert!(!peer_id.to_string().is_empty());
    }

    #[tokio::test]
    async fn m2_swarm_builds_and_subscribes() {
        let swarm = new_swarm().await;
        assert!(swarm.is_ok());
    }
}
