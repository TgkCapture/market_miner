use dotenv::dotenv;
use std::env;

pub fn get_env_var(key: &str) -> String {
    dotenv().ok(); 
    env::var(key).unwrap_or_else(|_| panic!("{} must be set in the .env file", key))
}