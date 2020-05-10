use std::{collections::HashMap, sync::MutexGuard};

fn port_is_available(port: u16) -> bool {
    match std::net::TcpListener::bind(("127.0.0.1", port)) {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub fn get_available_port(used_ports: &MutexGuard<HashMap<u16, bool>>) -> Option<u16> {
    (10000..11000).find(|port| {
        if port_is_available(*port) && !used_ports.contains_key(&port) {
            true
        } else {
            false
        }
    })
}
