use std::mem;
use std::ptr;

mod win32 {
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]

    use libc;
    use std::ptr;

    #[cfg(target_pointer_width = "32")]
    pub type SOCKET = u32;
    #[cfg(target_pointer_width = "64")]
    pub type SOCKET = u64;

    pub const WSADESCRIPTION_LEN: usize = 256;
    pub const WSASYS_STATUS_LEN: usize = 128;

    pub const INVALID_SOCKET: SOCKET = !0 as SOCKET;
    pub const SOMAXCONN: i32 = 0x7fffffff;
    pub const SOCKET_ERROR: i32 = -1;

    pub const AF_UNSPEC: i32 = 0;
    pub const AF_INET: i32 = 2;
    pub const AF_INET6: i32 = 23;

    pub const SOCK_STREAM: i32 = 1;
    pub const SOCK_DGRAM: i32 = 2;

    pub const IPPROTO_TCP: i32 = 6;
    pub const IPPROTO_UDP: i32 = 17;

    pub const AI_NONE: i32 = 0x00000000;
    pub const AI_PASSIVE: i32 = 0x00000001;

    #[repr(C)]
    pub struct WSAData {
        wVersion: u16,
        wHighVersion: u16,
        szDescription: [u8; WSADESCRIPTION_LEN + 1],
        szSystemStatus: [u8; WSASYS_STATUS_LEN + 1],

        // Ignore for v2 and up
        iMaxSockets: u16,
        iMaxUdpDg: u16,
        lpVendorInfo: *mut u8,
    }

    impl WSAData {
        pub fn new () -> WSAData {
            return WSAData {
                wVersion: 0,
                wHighVersion: 0,
                szDescription: [0; WSADESCRIPTION_LEN + 1],
                szSystemStatus: [0; WSASYS_STATUS_LEN + 1],
                iMaxSockets: 0,
                iMaxUdpDg: 0,
                lpVendorInfo: ptr::null_mut()
            };
        }
    }

    #[repr(C)]
    pub struct in_addr {
        pub s_b1: u8,
        pub s_b2: u8,
        pub s_b3: u8,
        pub s_b4: u8,
    }

    #[repr(C)]
    pub struct sockaddr_in {
        pub sin_family: i16,
        pub sin_port: u16,
        pub sin_addr: in_addr,
        pub sa_zero: [u8; 8],
    }

    #[link(name = "Ws2_32")]
    extern "stdcall" {
        pub fn WSAStartup (
            wVersionRequested: u16, // IN
            lpWSAData: *mut WSAData // OUT
        ) -> i32;

        pub fn WSACleanup () -> i32;

        pub fn socket (
            af: i32,       // IN
            socktype: i32, // IN
            protocol: i32  // IN
        ) -> SOCKET;

        pub fn bind (
            s: SOCKET,                // IN
            name: *const sockaddr_in, // IN
            namelen: i32              // IN
        ) -> i32;

        pub fn closesocket (
            s: SOCKET // IN
        ) -> i32;

        pub fn listen (
            s: SOCKET,   // IN
            backlog: i32 // IN
        ) -> i32;

        pub fn accept (
            s: SOCKET,              // IN
            addr: *mut sockaddr_in, // OUT OPT
            addrlen: *mut i32       // IN OUT OPT
        ) -> SOCKET;

        pub fn recv (
            s: SOCKET,    // IN
            buf: *mut u8, // OUT
            len: i32,     // IN
            flags: i32    // IN
        ) -> i32;

        pub fn send (
            s: SOCKET,    // IN
            buf: *mut u8, // IN
            len: i32,     // IN
            flags: i32    // IN
        ) -> i32;
    }
}

pub fn init () -> bool {
    let mut data = win32::WSAData::new();
    let result;
    unsafe {
        result = win32::WSAStartup(
            2 + (2 << 8),
            &mut data as *mut win32::WSAData
        );
    }
    return result == 0;
}

pub fn cleanup () -> bool {
    let result;
    unsafe {
        result = win32::WSACleanup();
    }
    return result == 0;
}

#[derive(Debug)]
pub struct IpAddrV4 {
    octets: [u8; 4]
}

impl IpAddrV4 {
    pub fn new_from_octets (
        o1: u8,
        o2: u8,
        o3: u8,
        o4: u8
    ) -> IpAddrV4 {
        IpAddrV4 {
            octets: [o1, o2, o3, o4]
        }
    }

    pub fn octets (&self) -> [u8; 4] {
        return self.octets;
    }
}

#[derive(Debug)]
pub struct SocketAddrV4 {
    pub ip: IpAddrV4,
    pub port: u16,
}

impl SocketAddrV4 {
    pub fn new (
        ip: IpAddrV4,
        port: u16
    ) -> SocketAddrV4 {
        SocketAddrV4 {
            ip: ip,
            port: port
        }
    }
}

#[derive(Debug)]
pub enum SocketAddr {
    V4(SocketAddrV4),
}

pub struct TcpStream {
    socket: win32::SOCKET,
}

impl TcpStream {
    pub fn read (&mut self, buffer: &mut [u8]) -> Option<usize> {
        let count;
        unsafe {
            count = win32::recv(
                self.socket,
                buffer.as_mut_ptr(),
                buffer.len() as i32,
                0
            );
        }
        if count < 0 {
            return None;
        }

        return Some(count as usize);
    }

    pub fn write (&mut self, buffer: &[u8]) -> Option<usize> {
        let count;
        unsafe {
            count = win32::send(
                self.socket,
                buffer.as_ptr() as *mut u8,
                buffer.len() as i32,
                0
            );
        }
        if count < 0 {
            return None;
        }

        return Some(count as usize);
    }

    pub fn close (self) -> bool {
        let result;
        unsafe {
            result = win32::closesocket(self.socket);
        }
        return result == 0;
    }
}

pub struct TcpListener {
    socket: win32::SOCKET,
}

impl TcpListener {
    pub fn new (addr: SocketAddr) -> Option<TcpListener> {
        // Create socket
        let socket;
        unsafe {
            socket = win32::socket(
                win32::AF_INET,
                win32::SOCK_STREAM,
                win32::IPPROTO_TCP
            );
        }
        if socket == win32::INVALID_SOCKET {
            return None;
        }

        // Create sockaddr for binding
        let SocketAddr::V4(v4) = addr;
        let octets = v4.ip.octets();
        let port = ((v4.port & 0xff) << 8) + ((v4.port & 0xff00) >> 8);
        let sockaddr = win32::sockaddr_in {
            sin_family: win32::AF_INET as i16,
            sin_port: port,
            sin_addr: win32::in_addr {
                s_b1: octets[0],
                s_b2: octets[1],
                s_b3: octets[2],
                s_b4: octets[3],
            },
            sa_zero: [0; 8]
        };

        // Bind socket to address
        let mut result;
        unsafe {
            result = win32::bind(
                socket,
                &sockaddr as *const win32::sockaddr_in,
                mem::size_of::<win32::sockaddr_in>() as i32
            );
        }
        if result != 0 {
            // Close socket and return
            unsafe {
                win32::closesocket(socket);
            }
            return None;
        }

        // Listen
        unsafe {
            result = win32::listen(
                socket,
                win32::SOMAXCONN
            );
        }
        if result != 0 {
            // Close socket and return
            unsafe {
                win32::closesocket(socket);
            }
            return None;
        }

        // Return listener
        return Some(TcpListener {
            socket: socket
        });
    }

    pub fn close (self) -> bool {
        let result;
        unsafe {
            result = win32::closesocket(self.socket);
        }
        return result == 0;
    }

    pub fn accept (&mut self) -> Option<TcpStream> {
        let socket;
        unsafe {
            socket = win32::accept(
                self.socket,
                ptr::null_mut(),
                ptr::null_mut()
            );
        }
        if socket == win32::INVALID_SOCKET {
            return None;
        }

        return Some(TcpStream {
            socket: socket
        });
    }
}
