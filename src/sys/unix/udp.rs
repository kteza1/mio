use {io, poll, Evented, EventSet, Poll, PollOpt, Token};
use io::MapNonBlock;
use unix::EventedFd;
use std::net::{self, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::os::unix::io::{RawFd, IntoRawFd, AsRawFd, FromRawFd};
use std::sync::atomic::{AtomicUsize, Ordering};

#[allow(unused_imports)] // only here for Rust 1.8
use net2::UdpSocketExt;

#[derive(Debug)]
pub struct UdpSocket {
    io: net::UdpSocket,
    selector_id: AtomicUsize,
}

impl UdpSocket {
    pub fn new(socket: net::UdpSocket) -> io::Result<UdpSocket> {
        try!(socket.set_nonblocking(true));
        Ok(UdpSocket {
            io: socket,
            selector_id: AtomicUsize::new(0),
        })
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.io.local_addr()
    }

    pub fn try_clone(&self) -> io::Result<UdpSocket> {
        self.io.try_clone().map(|io| {
            UdpSocket {
                io: io,
                selector_id: AtomicUsize::new(self.selector_id.load(Ordering::SeqCst)),
            }
        })
    }

    pub fn send_to(&self, buf: &[u8], target: &SocketAddr)
                   -> io::Result<Option<usize>> {
        self.io.send_to(buf, target)
            .map_non_block()
    }

    pub fn recv_from(&self, buf: &mut [u8])
                     -> io::Result<Option<(usize, SocketAddr)>> {
        self.io.recv_from(buf)
            .map_non_block()
    }

    pub fn broadcast(&self) -> io::Result<bool> {
        self.io.broadcast()
    }

    pub fn set_broadcast(&self, on: bool) -> io::Result<()> {
        self.io.set_broadcast(on)
    }

    pub fn multicast_loop_v4(&self) -> io::Result<bool> {
        self.io.multicast_loop_v4()
    }

    pub fn set_multicast_loop_v4(&self, on: bool) -> io::Result<()> {
        self.io.set_multicast_loop_v4(on)
    }

    pub fn multicast_ttl_v4(&self) -> io::Result<u32> {
        self.io.multicast_ttl_v4()
    }

    pub fn set_multicast_ttl_v4(&self, ttl: u32) -> io::Result<()> {
        self.io.set_multicast_ttl_v4(ttl)
    }

    pub fn multicast_loop_v6(&self) -> io::Result<bool> {
        self.io.multicast_loop_v6()
    }

    pub fn set_multicast_loop_v6(&self, on: bool) -> io::Result<()> {
        self.io.set_multicast_loop_v6(on)
    }

    pub fn ttl(&self) -> io::Result<u32> {
        self.io.ttl()
    }

    pub fn set_ttl(&self, ttl: u32) -> io::Result<()> {
        self.io.set_ttl(ttl)
    }

    pub fn join_multicast_v4(&self,
                             multiaddr: &Ipv4Addr,
                             interface: &Ipv4Addr) -> io::Result<()> {
        self.io.join_multicast_v4(multiaddr, interface)
    }

    pub fn join_multicast_v6(&self,
                             multiaddr: &Ipv6Addr,
                             interface: u32) -> io::Result<()> {
        self.io.join_multicast_v6(multiaddr, interface)
    }

    pub fn leave_multicast_v4(&self,
                              multiaddr: &Ipv4Addr,
                              interface: &Ipv4Addr) -> io::Result<()> {
        self.io.leave_multicast_v4(multiaddr, interface)
    }

    pub fn leave_multicast_v6(&self,
                              multiaddr: &Ipv6Addr,
                              interface: u32) -> io::Result<()> {
        self.io.leave_multicast_v6(multiaddr, interface)
    }

    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        self.io.take_error()
    }

    // pub fn set_multicast_loop(&self, on: bool) -> io::Result<()> {
    //     nix::setsockopt(self.as_raw_fd(), nix::sockopt::IpMulticastLoop, &on)
    //         .map_err(super::from_nix_error)
    // }
    //
    // pub fn join_multicast(&self, multi: &IpAddr) -> io::Result<()> {
    //     match *multi {
    //         IpAddr::V4(ref addr) => {
    //             // Create the request
    //             let req = nix::ip_mreq::new(nix::Ipv4Addr::from_std(addr), None);
    //
    //             // Set the socket option
    //             nix::setsockopt(self.as_raw_fd(), nix::sockopt::IpAddMembership, &req)
    //                 .map_err(super::from_nix_error)
    //         }
    //         IpAddr::V6(ref addr) => {
    //             // Create the request
    //             let req = nix::ipv6_mreq::new(nix::Ipv6Addr::from_std(addr));
    //
    //             // Set the socket option
    //             nix::setsockopt(self.as_raw_fd(), nix::sockopt::Ipv6AddMembership, &req)
    //                 .map_err(super::from_nix_error)
    //         }
    //     }
    // }
    //
    // pub fn leave_multicast(&self, multi: &IpAddr) -> io::Result<()> {
    //     match *multi {
    //         IpAddr::V4(ref addr) => {
    //             // Create the request
    //             let req = nix::ip_mreq::new(nix::Ipv4Addr::from_std(addr), None);
    //
    //             // Set the socket option
    //             nix::setsockopt(self.as_raw_fd(), nix::sockopt::IpDropMembership, &req)
    //                 .map_err(super::from_nix_error)
    //         }
    //         IpAddr::V6(ref addr) => {
    //             // Create the request
    //             let req = nix::ipv6_mreq::new(nix::Ipv6Addr::from_std(addr));
    //
    //             // Set the socket option
    //             nix::setsockopt(self.as_raw_fd(), nix::sockopt::Ipv6DropMembership, &req)
    //                 .map_err(super::from_nix_error)
    //         }
    //     }
    // }
    //
    // pub fn set_multicast_time_to_live(&self, ttl: i32) -> io::Result<()> {
    //     let v = if ttl < 0 {
    //         0
    //     } else if ttl > 255 {
    //         255
    //     } else {
    //         ttl as u8
    //     };
    //
    //     nix::setsockopt(self.as_raw_fd(), nix::sockopt::IpMulticastTtl, &v)
    //         .map_err(super::from_nix_error)
    // }

    fn associate_selector(&self, poll: &Poll) -> io::Result<()> {
        let selector_id = self.selector_id.load(Ordering::SeqCst);

        if selector_id != 0 && selector_id != poll::selector(poll).id() {
            Err(io::Error::new(io::ErrorKind::Other, "socket already registered"))
        } else {
            self.selector_id.store(poll::selector(poll).id(), Ordering::SeqCst);
            Ok(())
        }
    }
}

impl Evented for UdpSocket {
    fn register(&self, poll: &Poll, token: Token, interest: EventSet, opts: PollOpt) -> io::Result<()> {
        try!(self.associate_selector(poll));
        EventedFd(&self.as_raw_fd()).register(poll, token, interest, opts)
    }

    fn reregister(&self, poll: &Poll, token: Token, interest: EventSet, opts: PollOpt) -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).deregister(poll)
    }
}

impl FromRawFd for UdpSocket {
    unsafe fn from_raw_fd(fd: RawFd) -> UdpSocket {
        UdpSocket {
            io: net::UdpSocket::from_raw_fd(fd),
            selector_id: AtomicUsize::new(0),
        }
    }
}

impl IntoRawFd for UdpSocket {
    fn into_raw_fd(self) -> RawFd {
        self.io.into_raw_fd()
    }
}

impl AsRawFd for UdpSocket {
    fn as_raw_fd(&self) -> RawFd {
        self.io.as_raw_fd()
    }
}
