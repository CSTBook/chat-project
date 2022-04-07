use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;

const LOCAL: &str = "127.0.0.1:6000";
const MSG_SIZE: usize = 256;

fn sleep() {
    thread::sleep(::std::time::Duration::from_millis(100));
}
fn main() {
    //listen for connections on local addr
    let server = TcpListener::bind(LOCAL).expect("Listener failed to bind");
    //does not wait until it responds to a request (multiple can send messages at a time)
    server.set_nonblocking(true).expect("failed to initialize non-blocking");

    //empty vector to contain possible clients
    let mut clients = vec![];
    //channel to send/receive strings
    let (tx, rx) = mpsc::channel::<String>();
    
    //while true
    loop {
        //if able to establish a connection with an address (returns Result<TcpStream, SocketAddr>)
        if let Ok((mut socket, addr)) = server.accept() {
            println!("Client {} connected", addr);

            //make a clone of the transmitter for the new client
            let tx = tx.clone();

            //add a clone of the socket (client endpoint) to clients vector
            clients.push(socket.try_clone().expect("Failed to clone client"));

            //spawn a thread, moving ownership of all vars into it
            thread::spawn(move || loop {
                //create buffer of size MSG_SIZE
                let mut buf = vec![0;MSG_SIZE];

                //read as many characters as possible into buffer
                match socket.read_exact(&mut buf) {
                    Ok(_) => {
                        //turns the buffer up to the null byte (0) into a vector
                        let msg = buf.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        //turns the vector into a string using the from_utf8 method
                        let msg = String::from_utf8(msg).expect("Invalid utf8 message");

                        println!("{}", msg);
                        
                        //send message to channel (server owns this channel)
                        tx.send(msg).expect("Failed to send message to rx");
                    },
                    //if operation goes against non-blocking rules ignore
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                    Err(_) => {
                        //disconnect from client and end thread connection
                        println!("closing connection with {}", addr);
                        break;
                    }
                }

                sleep();
            });
        }

        //if something is present in the server's channel
        if let Ok(msg) = rx.try_recv(){
            //iterate over clients, only keeping what returns with a Some(_) when
            clients = clients.into_iter().filter_map(|mut client| {
                //turns the message into a vector of bytes
                let mut buf = msg.clone().into_bytes();
                //limits the size of message to message size
                buf.resize(MSG_SIZE, 0);
                
                //send message through the tcp stream
                client.write_all(&buf).map(|_| client).ok() 
            //recollect clients as a vector
            }).collect::<Vec<_>>();
        }

        sleep();
    }
}
