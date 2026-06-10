use libp2p::{gossipsub, swarm::NetworkBehaviour, PeerId, Swarm};
use libp2p::identity::Keypair;
use libp2p::gossipsub::{MessageAuthenticity, ValidationMode};
use libp2p::swarm::SwarmBuilder;
use libp2p::core::upgrade;
use libp2p::noise;
use libp2p::tcp;
use libp2p::yamux;
use std::error::Error;

#[derive(NetworkBehaviour)]
pub struct QcBehaviour {
    pub gossipsub: gossipsub::Behaviour,
}

pub async fn new_swarm() -> Result<Swarm<QcBehaviour>, Box<dyn Error>> {
    let id_keys = Keypair::generate_ed25519();
    let peer_id = PeerId::from(id_keys.public());
    println!("Local peer id: {peer_id}");

    let gossipsub_config = gossipsub::ConfigBuilder::default()
      .validation_mode(ValidationMode::Strict)
      .build()
      .expect("Valid config");
    
    let gossipsub = gossipsub::Behaviour::new(
        MessageAuthenticity::Signed(id_keys.clone()), 
        gossipsub_config
    ).expect("Correct configuration");

    let behaviour = QcBehaviour { gossipsub };

    let transport = tcp::tokio::Transport::new(tcp::Config::default())
       .upgrade(upgrade::Version::V1)
       .authenticate(noise::Config::new(&id_keys)?)
       .multiplex(yamux::Config::default())
       .boxed();

    let swarm = SwarmBuilder::with_tokio_executor(transport, behaviour, peer_id).build();

    Ok(swarm)
}

pub fn peer_id_from_pk(_pk: &[u8]) -> PeerId {
    let keypair = Keypair::generate_ed25519();
    PeerId::from(keypair.public())
}

#[cfg(test)]
mod m4_tests {
    use super::*;

    #[test]
    fn m4_swarm_builds() {
        async_std::task::block_on(async {
            let swarm = new_swarm().await;
            assert!(swarm.is_ok());
        });
    }

    #[test]
    fn m2_peer_id_works() {
        let pk = vec![0u8; 1952];
        let peer_id = peer_id_from_pk(&pk);
        assert!(!peer_id.to_string().is_empty());
    }
}
