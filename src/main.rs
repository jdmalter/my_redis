use tokio::fs::File;
use tokio::io;
use tokio::io::AsyncReadExt;

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut f = File::open("src\\main.rs").await?;
    let mut buffer = [0; 10];

    let n = f.read(&mut buffer[..]).await?;

    dbg!(&buffer[..n]);
    Ok(())
}
