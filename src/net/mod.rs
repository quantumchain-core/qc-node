use libp2p::{
    gossipsub, mdns, noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, 
    Swarm, SwarmBuilder,
};
use libp2p::futures::StreamExt;
use std::error::Error;
use tokio::time::{Duration, interval};
use crate::crypto::dilithium::RAMPURA_TESTNET_0_CHAIN_ID;

// This macro generates QtcBehaviourEvent automatically
#[derive(NetworkBehaviour)]
pub struct QtcBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
}

pub async fn start_network() -> Result<(), Box<dyn Error>> {
    let mut swarm = SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|key| {
            let gossipsub_config = gossipsub::ConfigBuilder::default()
                .heartbeat_interval(Duration::from_secs(1))
                .build()
                .expect("Valid config");
                
            let gossipsub = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(key.clone()),
                gossipsub_config,
            )?;
            
            let mdns = mdns::tokio::Behaviour::new(
                mdns::Config::default(), 
                key.public().to_peer_id()
            )?;
            
            Ok(QtcBehaviour { gossipsub, mdns })
        })?
        .build();

    let topic = gossipsub::IdentTopic::new(RAMPURA_TESTNET_0_CHAIN_ID);
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    println!("Started P2P node for {}", RAMPURA_TESTNET_0_CHAIN_ID);
    println!("Local PeerId: {}", swarm.local_peer_id());

    let mut tick = interval(Duration::from_secs(3));
    loop {
        tokio::select! {
            _ = tick.tick() => {
                let block_data = format!("block-{}-{}", RAMPURA_TESTNET_0_CHAIN_ID, chrono::Utc::now().timestamp());
                if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic.clone(), block_data.as_bytes()) {
                    println!("Publish error: {:?}", e);
                }
            }
            event = swarm.next().await => {
                match event {
                    Some(SwarmEvent::NewListenAddr { address, .. }) => {
                        println!("Listening on {}", address);
                    }
                    Some(SwarmEvent::Behaviour(QtcBehaviourEvent::Mdns(mdns::Event::Discovered(list)))) => {
                        for (peer_id, _multiaddr) in list {
                            println!("mDNS discovered peer: {}", peer_id);
                            swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                        }
                    }
                    Some(SwarmEvent::Behaviour(QtcBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                        message, ..
                    }))) => {
                        println!("Received block: {}", String::from_utf8_lossy(&message.data));
                    }
                    _ => {}
                }
            }
        }
    }
                }
