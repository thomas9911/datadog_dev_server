use std::io::Read;
use std::net::UdpSocket;

fn main() -> std::io::Result<()> {
    let socket = setup()?;

    loop {
        let mut buffer = String::new();
        std::io::stdin().read_to_string(&mut buffer)?;

        // println!("{:?}", buffer);
        socket.send(buffer.as_bytes())?;

        let mut buf = vec![0; 512];
        let (amt, src) = socket.recv_from(&mut buf)?;

        // println!("{:?}, {:?}", amt, src);
        println!("{:?}", std::str::from_utf8(&buf[0..amt]));
    }
    Ok(())
}

fn setup() -> std::io::Result<UdpSocket> {
    let socket = UdpSocket::bind("127.0.0.1:12345")?;
    socket.connect("127.0.0.1:34254")?;
    Ok(socket)
}
