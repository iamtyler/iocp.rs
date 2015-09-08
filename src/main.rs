extern crate libc;

mod queue;
mod socket;

fn main () {
    assert!(socket::init());

    let addr = socket::SocketAddr::V4(socket::SocketAddrV4 {
        ip: socket::IpAddrV4::new_from_octets(0, 0, 0, 0),
        port: 12345,
    });
    let mut listener = socket::TcpListener::new(addr).unwrap();

    let mut sock = listener.accept().unwrap();

    let mut buffer: [u8; 1024] = [0; 1024];
    let count = sock.read(&mut buffer).unwrap();
    let data = unsafe { std::str::from_utf8_unchecked(&buffer[..count]) };
    print!("{0}", data);

    let reply = "HTTP/1.1 200 OK\r\nContent-Length: 0\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n";
    sock.write(reply.as_ref()).unwrap();

    assert!(sock.close());
    assert!(listener.close());
    assert!(socket::cleanup());
}
