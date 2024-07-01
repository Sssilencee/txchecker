use sonic_rs::Deserialize;

#[derive(Deserialize)]
pub struct RpcRes<T> {
    pub result: T,
}

#[derive(Deserialize)]
pub struct RpcNotification<T> {
    pub params: NotificationParams<T>,
}

#[derive(Deserialize)]
pub struct NotificationParams<T> {
    pub result: T,
}