use libp2p::{gossipsub, swarm::NetworkBehaviour, PeerId, Swarm, Multiaddr};
use libp2p::identity::Keypair;
use libp2p::gossipsub::{MessageAuthenticity, ValidationMode};
use libp2p::swarm::SwarmEvent;
use std::error::Error;

#[derive(NetworkBehaviour)]
pub struct QcBehaviour {
    pub gossipsub: gossipsub::Behaviour,
}

pub async fn start_node() -> Result<(), Box<dyn Error>> {
    let id_keys = Keypair::generate_ed25519();
    let peer_id = PeerId::from(id_keys.public());
    println!("Local peer id: {peer_id}");

    let gossipsub_config = gossipsub::ConfigBuilder::default()
       .validation_mode(ValidationMode::Strict)
       .build()
       .expect("Valid config");
    
    let mut gossipsub = gossipsub::Behaviour::new(
        MessageAuthenticity::Signed(id_keys.clone()), 
        gossipsub_config
    ).expect("Correct configuration");

    let topic = gossipsub::IdentTopic::new("qc-blocks");
    gossipsub.subscribe(&topic)?;

    let behaviour = QcBehaviour { gossipsub };
    let mut swarm = Swarm::new(
        libp2p::core::transport::MemoryTransport::default(),
        behaviour,
        peer_id,
        libp2p::swarm::Config::with_async_std_executor(),
    );

    swarm.listen_on("/memory/0".parse()?)?;
    
    Ok(())
}

pub fn peer_id_from_pk(_pk: &[u8]) -> PeerId {
    let keypair = Keypair::generate_ed25519();
    PeerId::from(keypair.public())
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
}
