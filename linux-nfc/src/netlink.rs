use std::{
    intrinsics::transmute,
    io::{Error, ErrorKind, Read, Result as IOResult, Write},
    mem::size_of,
    ptr::slice_from_raw_parts,
};

use rsnl::socket::NetlinkSocket as RsnlNetlinkSocket;

use libc::{
    c_int, c_void, close, nlmsghdr, recv, send, socket, AF_NETLINK, NETLINK_GENERIC, NLMSG_DONE,
    NLM_F_REQUEST, NLM_F_ROOT, SOCK_RAW,
};
use tracing::{error, instrument, trace, warn};

/*
  NetlinkSocket
  uint32 fd
impl Drom for NetlinkSocket -> closes the socket
impl Write for NetlinkSocket
impl Read for NetlinkSocket
impl Debug for NetlinkSocket //
impl NetlinkSocket
  new() -> Result<Self>

  read_response() // read() + parse CMD response
  read_
  get_device_list() -> Vec<?>
  get_device(dev: u32) -> ?
  get_targets(dev: u32) -> Vec<NfcTargetInfo>

  start_poll(dev: u32)
  stop_poll(dev: u32)
  poll(dev: u32) -> Result<NfcTargetInfo>
     equivalent to start_poll(), then read waiting for TARGETS_FOUND event, followed by calling get_targets()
  */

const MAX_READ: usize = 4096;

const FLAGS_SEND: i32 = 0;
const FLAGS_RECV: i32 = 0;

const NFC_CMD_GET_DEVICE: i32 = 1;

#[derive(Debug)]
pub struct NetlinkSocket {
    fd: c_int,
}

impl NetlinkSocket {
    pub fn new() -> IOResult<Self> {
        let fd = match unsafe { socket(AF_NETLINK, SOCK_RAW, NETLINK_GENERIC) } {
            -1 => {
                let err = Error::last_os_error();
                error!(%err, "Failed to create netlink socket");
                return Err(err);
            }
            fd => fd,
        };
        Ok(Self { fd })
    }
}

impl Write for NetlinkSocket {
    #[instrument]
    fn write(&mut self, buf: &[u8]) -> IOResult<usize> {
        let len = buf.len();
        let buf = buf.as_ptr() as *const c_void;
        match unsafe { send(self.fd, buf, len.into(), FLAGS_SEND) } {
            -1 => {
                let err = Error::last_os_error();
                error!(%err, "Netlink socket send failed");
                return Err(err);
            }
            sent => {
                trace!({ len }, "Send successful");
                Ok(sent as usize)
            }
        }
    }

    fn flush(&mut self) -> IOResult<()> {
        Ok(())
    }
}

impl Read for NetlinkSocket {
    fn read(&mut self, buf: &mut [u8]) -> IOResult<usize> {
        let buf = buf.as_mut_ptr() as *mut c_void;
        match unsafe { recv(self.fd, buf, MAX_READ, FLAGS_RECV) } {
            -1 => {
                let err = Error::last_os_error();
                error!(%err, "Netlink socket recv failed");
                return Err(err);
            }
            received => {
                trace!({ received }, "Receive successful");
                Ok(received as usize)
            }
        }
    }
}

impl Drop for NetlinkSocket {
    #[instrument]
    fn drop(&mut self) {
        match unsafe { close(self.fd) } {
            0 => (),
            err => warn!(%err, "Failed to close netlink socket"),
        };
    }
}

unsafe fn get_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::std::slice::from_raw_parts((p as *const T) as *const u8, ::std::mem::size_of::<T>())
}

impl NetlinkSocket {
    pub fn get_device_list(&mut self) -> IOResult<()> {
        let payload = &[0u8; 4];
        let header = nlmsghdr {
            nlmsg_len: (size_of::<nlmsghdr>() + payload.len()) as u32,
            nlmsg_type: NLMSG_DONE as u16,
            nlmsg_flags: (NLM_F_REQUEST) as u16,
            nlmsg_seq: 0,
            nlmsg_pid: 0,
        };

        let mut buf = vec![];
        buf.extend(unsafe { get_u8_slice::<nlmsghdr>(&header) });
        buf.extend(payload);
        println!("len={}, buf={:?}", buf.len(), buf);
        self.write(buf.as_slice())?;

        println!("Written. Waiting");
        let mut buf = vec![];
        let response = self.read(&mut buf)?;

        println!("Response: {:?}", response);

        Ok(())
    }
}
