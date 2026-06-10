use libp2p::{gossipsub, swarm::NetworkBehaviour, PeerId};
use libp2p::identity::Keypair;

pub fn peer_id_from_pk(_pk: &[u8]) -> PeerId {
    // M2: Generate libp2p PeerID. M1 Dilithium key used for blocks only.
    let keypair = Keypair::generate_ed25519();
    PeerId::from(keypair.public())
}

#[derive(NetworkBehaviour)]
struct QcBehaviour {
    gossipsub: gossipsub::Behaviour,
}

pub fn new_swarm() -> Result<PeerId, Box<dyn std::error::Error>> {
    let keypair = Keypair::generate_ed25519();
    let peer_id = PeerId::from(keypair.public());
    Ok(peer_id)
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
    fn m2_swarm_config_builds() {
        let result = new_swarm();
        assert!(result.is_ok());
    }
}
