use qc_node::crypto::dilithium::RAMPURA_TESTNET_0_CHAIN_ID;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 && args[1] == "--version" {
        println!("qc-node 0.1.0-alpha ({})", RAMPURA_TESTNET_0_CHAIN_ID);
        return;
    }
    
    if args.len() > 1 && args[1] == "start" {
        println!("Starting qc-node for {}...", RAMPURA_TESTNET_0_CHAIN_ID);
        println!("Status: Draft. Not implemented.");
        return;
    }
    
    println!("QuantumChain Node v0.1.0-alpha");
    println!("Network: {}", RAMPURA_TESTNET_0_CHAIN_ID);
    println!("Usage: qc-node [start|status|--version]");
  }
