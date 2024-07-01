use tokio::sync::mpsc::UnboundedReceiver;

#[derive(Default)]
pub struct LazyUnboundedReceiver<T> {
    rx: Option<UnboundedReceiver<T>>
}

impl<T> LazyUnboundedReceiver<T> {
    pub async fn recv(&mut self) -> Option<T> {
        match &mut self.rx {
            Some(ref mut rx) => rx.recv().await,
            None => None
        }
    }

    pub fn is_closed(&self) -> bool {
        match &self.rx {
            Some(rx) => rx.is_closed(),
            None => true,
        }
    }

    pub fn close(&mut self) {
        self.rx = None;
    }

    pub fn init(&mut self, rx: UnboundedReceiver<T>) {
        self.rx = Some(rx);
    }
}