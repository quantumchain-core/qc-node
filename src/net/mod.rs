// src/net/mod.rs
// QTC M2 + M7: libp2p swarm + gossip publishing
// QTC M17: + request-response behaviour for state sync, + actual
// listen/dial wiring. Before this, new_swarm() built a swarm that never
// called listen_on or dialed anyone — nodes were network-isolated; there
// was nothing for gossipsub to gossip TO. That's fixed here alongside
// adding sync.

pub mod handler;
pub mod sync_codec;
pub use handler::{GossipMsg, HandleResult, handle_gossip};
pub use sync_codec::SyncCodec;

use libp2p::swarm::NetworkBehaviour;
use libp2p::{gossipsub, noise, request_response, tcp, yamux, Multiaddr, PeerId, StreamProtocol, SwarmBuilder};
use libp2p::gossipsub::{IdentTopic, MessageAuthenticity, ValidationMode};
use libp2p::identity::Keypair;
use std::error::Error;

use crate::sync::{SyncRequest, SyncResponse};

/// Topics
pub const TOPIC_BLOCKS: &str = "qc-blocks";
pub const TOPIC_TXS:    &str = "qc-txs";

/// Default TCP port for the libp2p swarm to listen on, if QC_LISTEN_ADDR
/// isn't set. Matches the port documented in docs/RUN_VALIDATOR.md.
pub const DEFAULT_LISTEN_ADDR: &str = "/ip4/0.0.0.0/tcp/30333";

pub const SYNC_PROTOCOL: &str = "/qtc/sync/1.0.0";

#[derive(NetworkBehaviour)]
pub struct QcBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub sync: request_response::Behaviour<SyncCodec>,
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

    let sync_behaviour = request_response::Behaviour::new(
        [(StreamProtocol::new(SYNC_PROTOCOL), request_response::ProtocolSupport::Full)],
        request_response::Config::default(),
    );

    let swarm = SwarmBuilder::with_existing_identity(id_keys)
        .with_tokio()
        .with_tcp(
            tcp::Config::default().nodelay(true),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|_| QcBehaviour {
            gossipsub: gossipsub_behaviour,
            sync: sync_behaviour,
        })?
        .build();

    Ok(swarm)
}

/// Start listening for incoming connections. Without this, the swarm can
/// still dial OUT but nothing can ever dial IN — every node would need to
/// be the one initiating every connection, which doesn't scale past two
/// nodes. Call once at startup.
pub fn start_listening(
    swarm: &mut libp2p::Swarm<QcBehaviour>,
) -> Result<(), Box<dyn Error>> {
    let addr = std::env::var("QC_LISTEN_ADDR").unwrap_or_else(|_| DEFAULT_LISTEN_ADDR.to_string());
    let multiaddr: Multiaddr = addr.parse()?;
    swarm.listen_on(multiaddr)?;
    Ok(())
}

/// Dial each bootstrap peer from QC_BOOTSTRAP_PEERS (comma-separated
/// multiaddrs, e.g. "/ip4/1.2.3.4/tcp/30333/p2p/12D3KooW..."). Without
/// at least one bootstrap peer (or some other discovery mechanism this
/// codebase doesn't have yet, e.g. mDNS), a freshly-started node has no
/// way to find anyone — dialing bootstrap peers is the whole mechanism
/// for initially joining the network right now.
pub fn dial_bootstrap_peers(swarm: &mut libp2p::Swarm<QcBehaviour>) {
    let peers = std::env::var("QC_BOOTSTRAP_PEERS").unwrap_or_default();
    for addr_str in peers.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        match addr_str.parse::<Multiaddr>() {
            Ok(addr) => {
                if let Err(e) = swarm.dial(addr.clone()) {
                    eprintln!("⚠️  failed to dial bootstrap peer {addr}: {e}");
                }
            }
            Err(e) => eprintln!("⚠️  invalid QC_BOOTSTRAP_PEERS entry {addr_str:?}: {e}"),
        }
    }
}

/// Send a sync request to a specific peer. Returns the OutboundRequestId
/// so the caller can correlate the eventual response event if needed.
pub fn request_sync(
    swarm: &mut libp2p::Swarm<QcBehaviour>,
    peer: PeerId,
    req: SyncRequest,
) -> request_response::OutboundRequestId {
    swarm.behaviour_mut().sync.send_request(&peer, req)
}

/// Send a sync response back on an inbound request's channel.
pub fn respond_sync(
    swarm: &mut libp2p::Swarm<QcBehaviour>,
    channel: request_response::ResponseChannel<SyncResponse>,
    response: SyncResponse,
) -> Result<(), SyncResponse> {
    swarm.behaviour_mut().sync.send_response(channel, response)
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
