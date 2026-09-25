use rabovel_api::{router_from_env, ServerConfig};

#[tokio::main]
async fn main() {
    let app = router_from_env()
        .await
        .unwrap_or_else(|error| panic!("backend startup failed: {error}"));

    let server = ServerConfig::from_env().expect("invalid server configuration");
    let listener = tokio::net::TcpListener::bind(server.bind_addr)
        .await
        .expect("failed to bind rabovel-api");

    println!("Rabovel API listening on http://{}", server.bind_addr);
    #[cfg(debug_assertions)]
    println!(
        "Swagger UI available at http://{}/swagger-ui",
        server.bind_addr
    );

    axum::serve(listener, app)
        .await
        .expect("rabovel-api server failed");
}
