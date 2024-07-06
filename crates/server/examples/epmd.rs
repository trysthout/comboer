use proto::EpmdServer;




#[tokio::main]
async fn main() {
   let mut epmd_server = EpmdServer::new();
   epmd_server.accept("0.0.0.0:9000").await.unwrap();

}