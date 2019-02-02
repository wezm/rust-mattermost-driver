pub mod channel;
pub mod client;
pub mod team;
pub mod user;

pub use crate::client::Client;
pub use crate::client::Error;
pub use crate::client::UnauthenticatedClient;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
