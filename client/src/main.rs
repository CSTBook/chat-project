use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;
use std::sync::Arc;

const LOCAL: &str = "127.0.0.1:6000";
const MSG_SIZE: usize = 256;

fn main() {
    //connect to the tcp socket opened by server
    let mut client = TcpStream::connect(LOCAL).expect("Stream failed to connect");
    println!("Connected to {:?} with address {:?}", client.peer_addr(), client.local_addr());
    client.set_nonblocking(true).expect("failed to initiate non-blocking");

    let local_port = client.local_addr().unwrap().port();

    //create a new channel
    let (tx, rx) = mpsc::channel::<String>();

    println!("What's your name: ");
    let mut name = String::new(); 
    io::stdin().read_line(&mut name).expect("unable to read name from stdin");
    name.truncate(name.len() - 1);
    let name = Arc::new(name);

    {
        let name = Arc::clone(&name);
        //start a thread to listen for new messages, refreshes every 100ms
        thread::spawn(move || loop {
            //create a buffer of size MSG_SIZE
            let mut buff = vec![0;MSG_SIZE];

            //read in from the connection
            match client.read_exact(&mut buff) {
                Ok(_) => {
                    //collect the message into a vector of bytes up until the null byte of the message
                    let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                    //convert the vec of bytes into a string and print to user
                    //println!("{}", String::from_utf8(msg).expect("Invalid utf8 message"));
                    check_for_own_message(String::from_utf8(msg).unwrap(), (*name).to_string(), local_port);
                },
                Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                Err(_) => {
                    println!("connection with server severed");
                    break;
                }
            }

            //returns any values on the channel
            match rx.try_recv() {
                Ok(msg) => {
                    //if there is something in the channel (message to send)
                    let mut buff = msg.clone().into_bytes();
                    buff.resize(MSG_SIZE, 0);
                    //send to server
                    client.write_all(&buff).expect("writing to socket failed");
                },
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => break
            }

            thread::sleep(Duration::from_millis(100));
        });
    }
    println!("Write a Message: \n");
    loop {
        //create a buffer to hold stdin
        let mut buff = String::new();
        io::stdin().read_line(&mut buff).expect("reading from stdin failed");
        //trim buffer to remove whitespace
        let msg = buff.trim().to_string();
        //send message to local channel (sending queue) | if message is quit or sending message to channel fails, kill client
        if msg == ":quit" || tx.send(format!("{}_{}: {}", name, local_port, msg)).is_err() {break}
    }
    println!("good bye");
}

//only prints message if the client hasn't sent it (looks cleaner)
fn check_for_own_message (msg: String, local_name: String, local_port: u16) {
    //message is split up into two parts [sender_port, message_body]
    let (user_info, message_body) = msg.split_once(":").unwrap();
    let (sender_name, sender_port) = user_info.split_once("_").unwrap();
    //if the client is NOT the sender of the msg
    if sender_port == local_port.to_string() && sender_name == local_name {
        //print out the message, 
        println!("");
    } else {
        println!("{}_{} 🢡 {}\n", sender_name, sender_port, message_body);
    }
}