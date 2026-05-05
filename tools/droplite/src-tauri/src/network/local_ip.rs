use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};

pub fn detect_local_ip() -> Option<Ipv4Addr> {
    let socket = UdpSocket::bind(SocketAddr::from(([0, 0, 0, 0], 0))).ok()?;
    socket.connect(SocketAddr::from(([8, 8, 8, 8], 80))).ok()?;

    match socket.local_addr().ok()?.ip() {
        IpAddr::V4(ip) if !ip.is_loopback() => Some(ip),
        _ => None,
    }
}
