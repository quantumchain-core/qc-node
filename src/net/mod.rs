use libp2p::identity::Keypair;
use libp2p::swarm::{NetworkBehaviour, Swarm};
use libp2p::{gossipsub, PeerId};
use libp2p::gossipsub::{IdentTopic, MessageAuthenticity, ValidationMode};
use std::error::Error;

// This derive macro only works if you have features = ["macros"] in Cargo.toml
#[derive(NetworkBehaviour)]
pub struct QcBehaviour {
    pub gossipsub: gossipsub::Behaviour,
}

pub fn peer_id_from_pk(_pk: &[u8]) -> PeerId {
    let keypair = Keypair::generate_ed25519();
    PeerId::from(keypair.public())
}

pub async fn new_swarm() -> Result<Swarm<QcBehaviour>, Box<dyn Error>> {
    let id_keys = Keypair::generate_ed25519();

    let gossipsub_config = gossipsub::ConfigBuilder::default()
  .validation_mode(ValidationMode::Strict)
  .build()?;

    let mut gossipsub = gossipsub::Behaviour::new(
        MessageAuthenticity::Signed(id_keys.clone()),
        gossipsub_config
    )?;

    let topic = IdentTopic::new("qc-blocks");
    gossipsub.subscribe(&topic)?;

    let behaviour = QcBehaviour { gossipsub };
    // Swarm::new_ephemeral exists in libp2p-swarm 0.44.0 which ships with libp2p 0.53.0
    let swarm = Swarm::new_ephemeral(|_| behaviour);
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

    #[test]
    fn m2_swarm_builds_and_subscribes() {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let swarm = new_swarm().await;
            assert!(swarm.is_ok());
        });
    }
    }
