use proto::models::ping_service_server::PingService;

pub fn main() {
    fn _assert_trait<T: PingService>() {}
}