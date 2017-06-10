use std::clone::Clone;
use std::io;
use std::sync::mpsc::{Sender,Receiver,channel};
use std::sync::{Arc,Mutex};
//use mio::*;
//use mio::tcp::{TcpListener, TcpStream};
use std::net::*;
use std::io::{Read,Write};
use super::{Message, MessageType, MessageData};
use super::TestInstruction;
use super::serde_json;
use std::rc::Rc;
use std::cell::RefCell;
use std::ops::Deref;
use rand::Rng;
use super::random_color::*;
use super::layout_validator::*;
use std::collections::{HashSet,HashMap};
use std::thread::*;
use std::time::Duration;

#[derive(Serialize,Deserialize,Hash,Eq,PartialEq,Debug)]
pub struct JSONNode {
    #[serde(rename = "Children")] children: Vec<JSONNode>,
    #[serde(rename = "Name")] name: String,
    #[serde(rename = "Line")] line: i32,
    #[serde(rename = "File")] file: String,
    #[serde(rename = "ActualPositionX")] actual_position_x: i32,
    #[serde(rename = "ActualPositionY")] actual_position_y: i32,
    #[serde(rename = "ActualWidth")] actual_width: i32,
    #[serde(rename = "ActualHeight")] actual_height: i32,
    #[serde(rename = "RenderWidth")] render_width: i32,
    #[serde(rename = "RenderHeight")] render_height: i32,
    #[serde(rename = "RenderPositionX")] render_position_x: i32,
    #[serde(rename = "RenderPositionY")] render_position_y: i32,
}

impl JSONNode {
    fn into_validator_node_impl(&self, parent: Option<i32>, nodes: Rc<RefCell<Vec<Node>>>) {
        let mut id;
        {
            let mut nodes_ref = nodes.deref().borrow_mut();
            id = nodes_ref.len() as i32;
            let node = Node {
                id: id,
                parent: parent,
                node_data: NodeData {
                    name: self.name.to_string(),
                    line: self.line,
                    file: self.file.clone(),
                    actual_position_x: self.actual_position_x,
                    actual_position_y: self.actual_position_y,
                    actual_width: self.actual_width,
                    actual_height: self.actual_height,
                    render_width: self.render_width,
                    render_height: self.render_height,
                    render_position_x: self.render_position_x,
                    render_position_y: self.render_position_y,
                },
            };
            nodes_ref.push(node);
        }

        for c in &self.children {
            c.into_validator_node_impl(Some(id), nodes.clone());
        }
    }

    pub fn into_validator_node(&self) -> Vec<Node> {
        let nodes = Rc::new(RefCell::new(Vec::new()));
        self.into_validator_node_impl(None, nodes.clone());

        if let Ok(nodes_vec) = Rc::try_unwrap(nodes) {
            return nodes_vec.into_inner();
        } else {
            panic!("We had more than one reference to nodes vector when flattening");
        }
    }
}

#[derive(Deserialize,Debug)]
pub struct ScreenSize {
    #[serde(rename = "W")] w: f32,
    #[serde(rename = "H")] h: f32,
}

#[derive(Deserialize)]
pub struct LayoutChanged {
    #[serde(rename = "Id")] id: i32,
    #[serde(rename = "ScreenSize")] pub screen_size: ScreenSize,
    #[serde(rename = "Nodes")] pub nodes: JSONNode,
}

fn parse_json(json_string: &str) -> LayoutChanged {
    println!("JSON: {}", json_string);
    serde_json::from_str(json_string).unwrap()
}

pub struct ServerBuffer {
    buffer: Vec<u8>,
}

impl ServerBuffer {
    fn new() -> ServerBuffer {
        ServerBuffer {
            buffer: Vec::new()
        }
    }

    fn decode_messages(&mut self) -> Vec<Message> {
        let mut ret = Vec::new();
        while let Some((message,bytes_read)) = Message::decode(self.buffer.as_slice()) {
            self.buffer.drain(0..bytes_read as usize);
            //println!("We just drained {} bytes", bytes_read);
            ret.push(message);
        }
        ret
    }

    fn add_to_buffer(&mut self, buff: &mut [u8]) {
        self.buffer.extend_from_slice(buff);
    }
}

//const SERVER: Token = Token(0);

pub struct Server {
    sender: Sender<ServerCommand>,
    receiver: Receiver<ServerCommand>,
}

#[derive(Debug)]
enum ServerCommand {
    AcceptConnection,
    ClientConnected,
    CloseConnection,
    ClientDisconnected,
    RequestLayout,
    GotLayout(Nodes),
}

impl Server {

    pub fn start_new(addr: &str) -> Server {
        let addr = addr.to_string();

        let (from_server_tx, from_server_rx) = channel::<ServerCommand>();
        let (to_server_tx, to_server_rx) = channel::<ServerCommand>();

        //need to end the thread when we are done
        let server_thread_handle = spawn(move ||{
            let mut server_buffer = ServerBuffer::new();
            let listener = TcpListener::bind(addr).unwrap();

            for stream in listener.incoming() {



                match stream {
                    Ok(mut stream) => {

                        println!("new client!");
                        println!("WE GOT A NEW CONNECTION!");

                        from_server_tx.send(ServerCommand::ClientConnected);

                        //TODO: decode and encode message now!!

                        let mut id_counter = 0;

                        stream.set_nonblocking(true).expect("set_nonblocking call failed");

                        'outer: loop {


                            let mut bytes = [0;2048];
                            let mut br: i32 = -1;
                            if let Ok(bytes_read) = stream.read(&mut bytes) {
                                br = bytes_read as i32;

                                server_buffer.add_to_buffer(&mut bytes[0..bytes_read]);
                            }

                            //println!("Reading {}", br);

                            if server_buffer.buffer.len() > 0 {

                                let decoded_messages = server_buffer.decode_messages();
                                for message in decoded_messages {
                                    match message.message_type {
                                        MessageType::LayoutData => {
                                            let layout_changed = parse_json(&message.data.json_string);
                                            let screen_size = layout_changed.screen_size;
                                            //println!("Got sent screen size as well now {:?}", screen_size);
                                            let nodes = layout_changed.nodes;
                                            let validator_nodes = nodes.into_validator_node();
                                            let vn = Nodes::new(validator_nodes, (screen_size.w as i32, screen_size.h as i32));

                                            from_server_tx.send(ServerCommand::GotLayout(vn));
                                        },
                                        MessageType::RequestLayoutData => (),
                                        MessageType::None => ()
                                    }
                                }
                            }


                            while let Ok(command) = to_server_rx.try_recv() {
                                match command {
                                    ServerCommand::RequestLayout => {
                                        println!("requesting layout from the app");
                                        let data = format!("{}", id_counter);
                                        id_counter += 1;
                                        let message = Message {
                                            message_type: MessageType::RequestLayoutData,
                                            length: data.len() as i32,
                                            data: MessageData {
                                                json_string: data
                                            },
                                        };
                                        stream.write(message.as_bytes().as_slice());
                                    },
                                    ServerCommand::CloseConnection => {
                                        println!("Got a close connection command");
                                        break 'outer;
                                    },
                                    _ => ()
                                }
                            }


                            sleep_ms(50);
                        }
                    }
                    Err(e) => { /* connection failed */ }
                }

                from_server_tx.send(ServerCommand::ClientDisconnected);
            }
        });

        Server {
            sender: to_server_tx,
            receiver: from_server_rx,
        }
    }


    pub fn wait_for_client(&mut self) {
        println!("waiting for client");
        while let Ok(cc) = self.receiver.recv() {
            match cc {
                ServerCommand::ClientConnected => { break; }
                _ => ()
            }
        }
    }

    pub fn close_current_connection(&mut self) {
        println!("Trying to close current connection");
        self.sender.send(ServerCommand::CloseConnection);
    }

    pub fn request_layout_data(&self, id: i32) -> Option<(Nodes, i32)> {
        println!("Writing request");

        self.sender.send(ServerCommand::RequestLayout);

        for command in self.receiver.recv() {
            match command {
                ServerCommand::GotLayout(nodes) => {
                    println!("Got nodes in return");
                    return Some((nodes, id))
                }
                _ => {
                    println!("got other command: {:?}", command);
                }
            }
        }
        panic!("We should have gotten something here!");
    }




}
