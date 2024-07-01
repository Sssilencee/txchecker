use log::LevelFilter;

pub fn init(dev: bool) {
    let mut builder = env_logger::builder();

    if dev {
        builder.filter_level(LevelFilter::Debug);
    }

    builder
        .filter_module("hyper_util::client::legacy::connect::dns", LevelFilter::Error)
        .filter_module("hyper_util::client::legacy::connect::http", LevelFilter::Error)
        .filter_module("h2::client", LevelFilter::Error)
        .filter_module("h2::codec::framed_write", LevelFilter::Error)
        .filter_module("h2::codec::framed_read", LevelFilter::Error)
        .filter_module("hyper_util::client::legacy::pool", LevelFilter::Error)
        .filter_module("h2::proto::settings", LevelFilter::Error)
        .filter_module("h2::proto::connection", LevelFilter::Error)
        .init();
}