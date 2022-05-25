use gelato_relay::*;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let gelato = GelatoClient::default();

    let task_status = gelato
        .get_task_status("0x6265e895da9daf43ad028392669809127e4524760528488953e775f2be2b2b3e")
        .await
        .unwrap();
    println!("Task status: {:?}", task_status);

    Ok(())
}
