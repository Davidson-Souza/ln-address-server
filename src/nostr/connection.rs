use futures_util::stream::SplitSink;
use futures_util::stream::SplitStream;
use futures_util::SinkExt;
use futures_util::StreamExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::WebSocketStream;

#[allow(unused)]
pub struct WebsocketConnection {
    read_loop_hadle: JoinHandle<()>,
    writer: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    url: String,
    id: usize,
}

impl WebsocketConnection {
    pub fn id(&self) -> usize {
        self.id
    }

    pub fn address(&self) -> String {
        self.url.clone()
    }

    pub async fn new(id: usize, url: String, msg_sender: Sender<(usize, Message)>) -> Self {
        let (ws_stream, _) = connect_async(&url).await.expect("Failed to connect");
        let (writer, reader) = ws_stream.split();
        let read_loop_hadle = tokio::task::spawn(Self::read_loop(id, reader, msg_sender));

        Self {
            id,
            writer,
            read_loop_hadle,
            url,
        }
    }

    pub async fn write_to_connection(
        &mut self,
        message: Message,
    ) -> Result<(), tokio_tungstenite::tungstenite::Error> {
        self.writer.send(message).await
    }

    async fn read_loop(
        id: usize,
        mut reader: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
        msg_sender: Sender<(usize, Message)>,
    ) {
        while let Some(Ok(message)) = reader.next().await {
            msg_sender
                .send((id, message))
                .await
                .expect("main loop is broken");
        }
    }
}
