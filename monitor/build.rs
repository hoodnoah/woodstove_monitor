use std::env;

fn main() {
    // load .env file for build
    dotenvy::dotenv().ok();

    // re-export as cargo env vars
    if let Ok(ssid) = env::var("WIFI_SSID") {
        println!("cargo:rustc-env=WIFI_SSID={}", ssid);
    }

    if let Ok(wifi_pass) = env::var("WIFI_PASSWORD") {
        println!("cargo:rustc-env=WIFI_PASSWORD={}", wifi_pass);
    }

    println!("cargo:rerun-if-changed=.env");

    // embuild stuff
    embuild::espidf::sysenv::output();
}
