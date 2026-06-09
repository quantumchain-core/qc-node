use libp2p::{
    gossipsub, mdns, noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, PeerId, SwarmBuilder,
};
use pqcrypto_dilithium::dilithium3::*;
use pqcrypto_traits::sign::{PublicKey, SecretKey, SignedMessage};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use tokio::time;
use log::{info, error};

#[derive(NetworkBehaviour)]
struct QcBehaviour {
    gossipsub: gossipsub::Behaviour,
    mdns: mdns::tokio::Behaviour,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Block {
    height: u64,
    data: String,
    signature: Vec<u8>,
    public_key: Vec<u8>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    info!("Starting QuantumChain Rampura Testnet Node M3");

    // Generate Dilithium keypair for this node
    let (pk, sk) = keypair();
    info!("Node Dilithium PublicKey: {:?}", pk.as_bytes());

    let mut swarm = SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|key| {
            let message_id_fn = |message: &gossipsub::Message| {
                let mut s = DefaultHasher::new();
                message.data.hash(&mut s);
                gossipsub::MessageId::from(s.finish().to_string())
            };

            let gossipsub_config = gossipsub::ConfigBuilder::default()
                .heartbeat_interval(Duration::from_secs(10))
                .validation_mode(gossipsub::ValidationMode::Strict)
                .message_id_fn(message_id_fn)
                .build()
                .map_err(|msg| std::io::Error::new(std::io::ErrorKind::Other, msg))?;

            let gossipsub = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(key.clone()),
                gossipsub_config,
            )?;

            let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())?;
            Ok(QcBehaviour { gossipsub, mdns })
        })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    let topic = gossipsub::IdentTopic::new("rampura-blocks");
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    let mut block_height = 0u64;
    let mut block_interval = time::interval(Duration::from_secs(30));

    loop {
        tokio::select! {
            _ = block_interval.tick() => {
                // Create new block every 30 seconds
                block_height += 1;
                let block_data = format!("Rampura Block #{} from {}", block_height, PeerId::from(swarm.local_peer_id()));
                
                // Sign with Dilithium
                let signature = detached_sign(block_data.as_bytes(), &sk);
                
                let block = Block {
                    height: block_height,
                    data: block_data.clone(),
                    signature: signature.as_bytes().to_vec(),
                    public_key: pk.as_bytes().to_vec(),
                };

                let json = serde_json::to_string(&block)?;
                if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic.clone(), json.as_bytes()) {
                    error!("Failed to publish block: {e}");
                } else {
                    info!("Published M3 Block #{} with Dilithium signature", block_height);
                }
            }
            event = swarm.select_next_some() => match event {
                SwarmEvent::Behaviour(QcBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer_id, _multiaddr) in list {
                        info!("mDNS discovered peer: {peer_id}");
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                    }
                },
                SwarmEvent::Behaviour(QcBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                    propagation_source: peer_id,
                    message_id: _id,
                    message,
                })) => {
                    if let Ok(block) = serde_json::from_slice::<Block>(&message.data) {
                        // Verify Dilithium signature
                        let pk = PublicKey::from_bytes(&block.public_key).unwrap();
                        let sig = DetachedSignature::from_bytes(&block.signature).unwrap();
                        
                        match verify_detached_signature(&sig, block.data.as_bytes(), &pk) {
                            Ok(_) => info!("✅ Valid M3 Block #{} from {} - Dilithium signature verified", block.height, peer_id),
                            Err(_) => error!("❌ Invalid Dilithium signature on Block #{} from {}", block.height, peer_id),
                        }
                    }
                },
                SwarmEvent::NewListenAddr { address, .. } => {
                    info!("Local node listening on {address}");
                }
                _ => {}
            }
        }
    }
}
