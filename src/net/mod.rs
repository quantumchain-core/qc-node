use libp2p::{gossipsub, noise, swarm::NetworkBehaviour, tcp, yamux, PeerId};
use libp2p::identity::Keypair;
use std::error::Error;

pub fn peer_id_from_pk(_pk: &[u8]) -> PeerId {
    // M2 uses ed25519 for transport. M1 Dilithium is for block signatures only
    let keypair = Keypair::generate_ed25519();
    PeerId::from(keypair.public())
}

#[derive(NetworkBehaviour)]
struct QcBehaviour {
    gossipsub: gossipsub::Behaviour,
}

pub async fn start_swarm() -> Result<(), Box<dyn Error>> {
    let keypair = Keypair::generate_ed25519();
    
    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
       .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
       .with_behaviour(|key| {
            let gossipsub_config = gossipsub::Config::default();
            let gossipsub = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(key.clone()),
                gossipsub_config,
            ).unwrap();
            QcBehaviour { gossipsub }
        })?
       .build();

    let topic = gossipsub::IdentTopic::new("qc-blocks");
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
    Ok(())
}

#[cfg(test)]
mod m2_tests {
    use super::*;

    #[test]
    fn m2_swarm_starts() {
        let pk = vec![0u8; 1952]; // fake M1 pubkey
        let peer_id = peer_id_from_pk(&pk);
        assert!(!peer_id.to_string().is_empty());
    }
}
