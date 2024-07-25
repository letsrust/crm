use anyhow::Result;
use crm::pb::{user_service_client::UserServiceClient, CreateUserRequest};
use tonic::Request;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = UserServiceClient::connect("http://[::1]:50051").await?;

    let request = Request::new(CreateUserRequest {
        name: String::from("zhangsan"),
        email: String::from("zhangsan@gmail.com"),
    });

    let resp = client.create_user(request).await?;
    let user = resp.into_inner();
    println!("User created: {user:?}");
    Ok(())
}
