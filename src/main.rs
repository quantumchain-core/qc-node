use libp2p::{
    gossipsub, mdns, noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, SwarmBuilder,
};
use libp2p::futures::StreamExt; // FIX 1: This gives you .select_next_some()
use pqcrypto::sign::dilithium3::*;
use pqcrypto_traits::sign::{PublicKey as _, SecretKey as _, SignedMessage as _}; // FIX 2: Import trait for .as_bytes()
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
    info!("Starting QuantumChain Rampura Testnet Node M3 - Dilithium");

    let (pk, sk) = keypair();
    info!("Node Dilithium PK: {}", hex::encode(pk.as_bytes()));

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
        .build();

    let topic = gossipsub::IdentTopic::new("rampura-blocks");
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    let mut block_height = 0u64;
    let mut block_interval = time::interval(Duration::from_secs(30));

    loop {
        // FIX 3: Use swarm.select_next_some() not swarm directly
        tokio::select! {
            _ = block_interval.tick() => {
                block_height += 1;
                let peer_id = *swarm.local_peer_id();
                let block_data = format!("Rampura Block #{} from {}", block_height, peer_id);
                
                let sm = sign(block_data.as_bytes(), &sk);
                
                let block = Block {
                    height: block_height,
                    data: block_data.clone(),
                    signature: sm.as_bytes().to_vec(), // FIX 4: .as_bytes() works now
                    public_key: pk.as_bytes().to_vec(),
                };

                let json = serde_json::to_string(&block)?;
                if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic.clone(), json.as_bytes()) {
                    error!("Failed to publish block: {e}");
                } else {
                    info!("Published M3 Block #{} with Dilithium sig", block_height);
                }
            }
            // FIX 5: Correctly poll the swarm
            event = swarm.select_next_some() => match event {
                SwarmEvent::NewListenAddr { address, .. } => {
                    info!("Local node listening on {address}");
                }
                SwarmEvent::Behaviour(QcBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer_id, _multiaddr) in list {
                        info!("mDNS discovered peer: {peer_id}");
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                    }
                },
                // FIX 6: Add `..` to ignore message_id field
                SwarmEvent::Behaviour(QcBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                    propagation_source: peer_id,
                    message,
                    .. // Ignores message_id and other new fields
                })) => {
                    if let Ok(block) = serde_json::from_slice::<Block>(&message.data) {
                        // FIX 7: Correct pqcrypto verify API
                        if let Ok(pk) = PublicKey::from_bytes(&block.public_key) {
                            // open() verifies and returns the original message
                            match open(&block.signature, &pk) {
                                Ok(verified_data) => {
                                    if verified_data == block.data.as_bytes() {
                                        info!("✅ Valid M3 Block #{} from {} - Dilithium verified", block.height, peer_id);
                                    } else {
                                        error!("❌ Data mismatch Block #{} from {}", block.height, peer_id);
                                    }
                                }
                                Err(_) => error!("❌ Invalid Dilithium sig Block #{} from {}", block.height, peer_id),
                            }
                        } else {
                            error!("❌ Invalid Dilithium pubkey Block #{} from {}", block.height, peer_id);
                        }
                    }
                },
                _ => {}
            }
        }
    }
}
