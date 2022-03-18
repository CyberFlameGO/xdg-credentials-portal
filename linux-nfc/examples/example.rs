use std::error::Error;

use linux_nfc::netlink::NetlinkSocket;

fn main() -> Result<(), Box<dyn Error>> {
    let mut socket = NetlinkSocket::new()?;
    println!("{:?}", socket);

    socket.get_device_list()?;

    Ok(())
}
