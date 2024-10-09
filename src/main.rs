// TCP Socket currently

use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;
use std::io::{ErrorKind, Read, Write};
use std::time::Duration;

const LOCAL_SERVER: &str = "127.0.0.1:8888";
const MSG_SIZE: usize = 1024;

fn main() {
    let server = TcpListener::bind(LOCAL_SERVER).expect("Listen Failed on Server");
    server
        .set_nonblocking(true)
        .expect("Failed to nonblocking");

    let mut clients = vec![];

    let (tx, rx) = mpsc::channel::<String>();

    loop {
        // Socket monitoring
        if let Ok((mut socket, addr)) = server.accept(){
            println!("{} connected", addr);
            clients.push(socket.try_clone().expect("Failed to clone client"));
            let tx = tx.clone();
            thread::spawn(move || loop {
                let mut buff = vec![0; MSG_SIZE];
                match socket.read_exact(&mut buff){
                    Ok(_) => {
                        let msg = buff.into_iter().take_while(|&x|x!=0).collect::<Vec<_>>();
                        let msg_string = String::from_utf8(msg).expect("Invalid message");

                        println!("{}: {:?}", addr, msg_string);
                        tx.send(msg_string).expect("Failed message to channel");
                    }
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                    Err(_) => {
                        println!("Closing connection with {}", addr);
                        break;
                    }
                }
                thread::sleep(Duration::from_millis(100));
            });
        }
        if let Ok(msg) = rx.try_recv(){
            clients = clients.into_iter().filter_map(|mut client|{
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                client.write_all(&buff).map(|_|client).ok()
            })
                .collect::<Vec<_>>();
        }
        thread::sleep(Duration::from_millis(100));
    }
}