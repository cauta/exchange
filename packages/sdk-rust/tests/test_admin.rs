/// Quick test of admin endpoint
use exchange_sdk::ExchangeClient;
use exchange_test_utils::TestServer;

#[tokio::test]
async fn test_admin_create_token() {
    let server = TestServer::start().await.expect("Failed to start server");
    let client = ExchangeClient::new(&server.base_url);

    let result = client
        .admin_create_token("BTC".to_string(), 18, "Bitcoin".to_string())
        .await;

    match &result {
        Ok(token) => println!("✅ Created token: {:?}", token),
        Err(e) => println!("❌ Error: {:?}", e),
    }

    assert!(result.is_ok(), "Should create token successfully");
}

#[tokio::test]
async fn test_admin_create_market() {
    let server = TestServer::start().await.expect("Failed to start server");
    let client = ExchangeClient::new(&server.base_url);

    // First create tokens
    client
        .admin_create_token("BTC".to_string(), 18, "Bitcoin".to_string())
        .await
        .expect("Failed to create BTC");

    client
        .admin_create_token("USDC".to_string(), 18, "USD Coin".to_string())
        .await
        .expect("Failed to create USDC");

    // Then create market
    let result = client
        .admin_create_market(
            "BTC".to_string(),
            "USDC".to_string(),
            1000,
            1000000,
            1000000,
            10,
            20,
        )
        .await;

    match &result {
        Ok(market) => println!("✅ Created market: {:?}", market),
        Err(e) => println!("❌ Error: {:?}", e),
    }

    assert!(result.is_ok(), "Should create market successfully");
}
