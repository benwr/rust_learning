use std::io;
use std::io::prelude::*;
use std::net::UdpSocket;
use std::str::from_utf8;
use std::time::Duration;
use std::thread;

fn main() {
    let socket = match UdpSocket::bind("localhost:1234") {
        Result::Ok(s)  => s,
        Result::Err(_) => panic!("failed to connect"),
    };

    match socket.set_read_timeout(Some(Duration::new(1, 1000))) {
        Result::Ok(_) => println!("set read timeout to 1000"),
        Result::Err(_) => panic!("failed to set read timeout"),
    };

    thread::spawn(move|| {
        let socket = match UdpSocket::bind("localhost:1111") {
            Result::Ok(s)  => s,
            Result::Err(_) => panic!("failed to connect"),
        };
        let mut end = false;
        
        while !end {
            let mut bs = [0; 65507];
            let (amt, src) = socket.recv_from(&mut bs).unwrap();
            println!("{}", amt);
            let result = from_utf8(&bs).unwrap();
            end = result == "/q\n";
            println!("{}", result);
        }
    });
    // let mut bytes = [0; 1];

    let mut end = false;
    while !end {
        let mut l : String = "".to_string();
        io::stdin().read_line(&mut l);
        if l == "/q\n".to_string() {
            end = true;
        } 
        socket.send_to(l.as_bytes(), "localhost:1111");
    }
}
