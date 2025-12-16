use std::path::PathBuf;

#[tokio::test]
async fn test_load_mainnet_fixtures() {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("fixtures/mainnet");

    let waypoint_path = d.join("waypoint.txt");
    let genesis_path = d.join("genesis.blob");

    // Test Waypoint Parsing
    let waypoint_str = std::fs::read_to_string(&waypoint_path).expect("Failed to read waypoint.txt");
    let waypoint = waypoint_str.trim().parse::<crate::types::waypoints::Waypoint>().expect("Failed to parse waypoint");
    println!("Parsed Waypoint: {}", waypoint);

    // Test Genesis Loading
    let genesis_bytes = std::fs::read(&genesis_path).expect("Failed to read genesis.blob");
    let _genesis_txn: crate::types::transaction::Transaction = bcs::from_bytes(&genesis_bytes).expect("Failed to deserialize genesis.blob");
    println!("Successfully deserialized Genesis Transaction ({} bytes)", genesis_bytes.len());
}
