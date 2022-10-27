use mini_redis::{client, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Connect to mini-redis
    let mut client = client::connect("127.0.0.1:6379").await?;

    // Set key/value pair
    client.set("hello", "world".into()).await?;

    // Get key
    let result = client.get("hello").await?;

    dbg!(result);

    Ok(())
}
