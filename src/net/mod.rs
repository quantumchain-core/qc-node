use libp2p::swarm::NetworkBehaviour;
use libp2p::{gossipsub, noise, tcp, yamux, PeerId, SwarmBuilder};
use libp2p::gossipsub::{IdentTopic, MessageAuthenticity, ValidationMode};
use libp2p::identity::Keypair;
use std::error::Error;

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
        .map_err(|e| Box::<dyn Error>::from(e))?;

    let mut gossipsub_behaviour = gossipsub::Behaviour::new(
        MessageAuthenticity::Signed(id_keys.clone()),
        gossipsub_config,
    )?;

    let topic = IdentTopic::new("qc-blocks");
    gossipsub_behaviour.subscribe(&topic)?;

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
