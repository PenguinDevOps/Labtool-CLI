use lazy_static::lazy_static;
use reqwest::blocking::Client;

lazy_static! {
   pub static ref API_CLIENT: Client = Client::new();
}
