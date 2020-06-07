use crate::file::{Chunk, FileReader};
use crate::Exception;
use tokio::sync::{mpsc, oneshot};

pub(crate) async fn ranger_loop(
    mut receiver: mpsc::UnboundedReceiver<oneshot::Sender<Option<Chunk>>>,
    mut ranger: FileReader,
) -> Result<(), Exception> {
    while let Some(sender) = receiver.recv().await {
        match ranger.read_chunk().await {
            Some(result) => match result {
                Ok(chunk) => sender
                    .send(Some(chunk))
                    .map_err(|_| format!("cannot send data to send_loop"))?,
                Err(e) => return Err(e.into()),
            },
            None => {
                sender
                    .send(None)
                    .map_err(|_| format!("cannot send EOF to send_loop"))?;
                break;
            }
        }
    }

    Ok(())
}

pub(crate) async fn send_loop(
    client: reqwest::Client,
    url: String,
    file_name: String,
    sender: mpsc::UnboundedSender<oneshot::Sender<Option<Chunk>>>,
) -> Result<(), Exception> {
    loop {
        let (os, or) = oneshot::channel();

        sender.send(os)?;

        match or.await? {
            None => break,
            Some(chunk) => {
                let len = chunk.data.len();
                let index = chunk.index;

                let resp = send_part(&client, &url, &file_name, chunk).await?;

                println!(
                    "{}.part{:?} ({} bytes) uploaded, response: {}.",
                    file_name, index, len, resp
                );
            }
        }
    }

    Ok(())
}

pub(crate) async fn send_part(
    client: &reqwest::Client,
    url: &str,
    file_name: &str,
    chunk: Chunk,
) -> Result<String, Exception> {
    use reqwest::multipart::*;

    let Chunk {
        size,
        index,
        count,
        data,
    } = chunk;

    let file = Part::bytes(data).file_name(file_name.to_owned());
    let form = Form::new()
        .part("file", file)
        .text("name", file_name.to_owned())
        .text("chunks", count.to_string())
        .text("chunk", index.to_string())
        .text("size", size.to_string());

    let req = client.post(url).multipart(form).build()?;

    Ok(client.execute(req).await?.text().await?)
}
