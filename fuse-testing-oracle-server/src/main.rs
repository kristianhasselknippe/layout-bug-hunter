extern crate sdl2;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate serde_yaml;
extern crate winapi;
extern crate user32;
extern crate kernel32;
extern crate gdi32;
extern crate clap;
extern crate rand;
extern crate mio;
extern crate time;
#[macro_use] extern crate itertools;

use sdl2::pixels::Color;
use sdl2::rect::{Point,Rect};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::BlendMode;

use sdl2::ttf::{init as init_ttf};

use itertools::*;

pub mod server;
pub mod test_sets;
pub mod test_runner;
pub mod drawing;
pub mod random_color;
pub mod layout_validator;
pub mod baseline_finder;

use baseline_finder::*;
use test_sets::*;

use layout_validator::*;
use layout_validator::validity_rules::*;
use layout_validator::overlap_and_overflow::{OverflowRect};
use test_runner::*;
use test_runner::test_script::*;
use drawing::*;

use std::net::*;
use std::io::Result;

use server::{Server};

use std::thread;

use std::convert::From;

use std::str;
use std::env;
use std::collections::{HashSet,HashMap};
use std::hash::{Hash,Hasher};
use std::cmp::{max,min};

use std::path::{Path,PathBuf};
use std::io::Write;
use std::fmt::Write as WriteFmt;


use std::sync::mpsc::*;
use std::sync::RwLock;
use std::sync::Arc;

use std::rc::Rc;

use std::cell::RefCell;

use clap::{Arg, App};

use rand::{Rng, thread_rng};

use time::{now,Tm};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageType {
    LayoutData,
    RequestLayoutData,
    None
}

impl MessageType {
    pub fn new(val: i32) -> MessageType {
        if val == 0 {
            MessageType::LayoutData
        } else {
            MessageType::None
        }
    }

    pub fn as_i32(&self) -> i32 {
        match self {
            &MessageType::LayoutData => 0,
            &MessageType::RequestLayoutData => 1,
            &MessageType::None => -1,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageData {
    pub json_string: String
}

impl MessageData {
    pub fn as_json_string(&self) -> String {
        let serialized = serde_json::to_string(self).unwrap();
        serialized
    }

    pub fn from_json_string(json: String) -> MessageData {
        MessageData {
            json_string: json
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub message_type: MessageType,
    pub length: i32,
    pub data: MessageData,
}

impl Message {
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut ret = Vec::new();
        let message_type = format!("{}", self.message_type.as_i32());
        let data_json_string = self.data.as_json_string();
        let message_data = data_json_string.as_bytes();
        let message_length = format!("{}", message_data.len());

        ret.extend_from_slice(message_type.as_bytes());
        ret.push(b'\n');
        ret.extend_from_slice(message_length.as_bytes());
        ret.push(b'\n');
        ret.extend_from_slice(data_json_string.as_bytes());
        ret
    }

    pub fn decode(buf: &[u8]) -> Option<(Message,u32)> {
        let buffer_string = String::from_utf8(Into::<Vec<u8>>::into(buf)).unwrap();
        //println!("BufferString: {}", buffer_string);
        let mut buffer = buf.to_vec();
        let mut bytes_read: u32 = 0;
        if let Some(i) = buffer.iter().position(|&b| b == b'\n') {
            let message_type_string = String::from_utf8(buffer.drain(0..i).collect()).unwrap();
            let message_type = MessageType::new(message_type_string.parse::<i32>().unwrap());

            //removes the \n
            buffer.remove(0);
            bytes_read += (i+1) as u32;

            if let Some(i2) = buffer.iter().position(|&b| b == b'\n') {
                let message_length_string = String::from_utf8(buffer.drain(0..i2).collect()).unwrap();
                let message_length : i32 = message_length_string.parse().unwrap();
                //removes the \n
                buffer.remove(0);
                bytes_read += (i2+1) as u32;

                if buffer.len() >= message_length as usize {
                    let data = buffer.drain(0..(message_length as usize));
                    let data_string = String::from_utf8(data.collect()).unwrap();
                    let ret_msg = Message {
                        message_type: message_type,
                        length: message_length,
                        data: MessageData::from_json_string(data_string),
                    };
                    bytes_read += message_length as u32;
                    Some((ret_msg,bytes_read as u32))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }

    }

    pub fn encode(&mut self) -> Vec<u8> {
        self.as_bytes()
    }
}

fn enter_to_continue() {
    println!("Press enter to continue");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input);
}

fn run_all_tests(draw_context: &mut DrawContext,
                 test_sets: &TestSets,
                 name: &str,
                 baseline_threshold: Option<f32>,
                 overlap_overflow_threshold: Option<f32>) -> LayoutViolations {
    println!("Running tests");

    for (_,ref test_set) in &test_sets.sets {
        let ref id = test_set.id;
        let nodes = test_set.nodes.clone();
        let ref tab_stops = test_set.tab_stops;
        let ref screen_size = test_set.screen_size;
        draw_context.renderer.window_mut().unwrap().set_size(screen_size.0 as u32, screen_size.1 as u32);
        println!("Rendering window_size: {:?}", screen_size);

        draw_context.draw_nodes(&nodes);
        let path = format!("output/{}/size{}x{}.png", name, screen_size.0, screen_size.1);
        draw_context.save_to_png(&path);

        draw_context.present();
    }

    //LAYOUT VALIDATION
    let mut violations = validate_layout(&test_sets);

    //FINDING BASELINE

    if let Some(baseline_threshold) = baseline_threshold {
        println!("We got a baseline threshold of {}", baseline_threshold);
        let n_test_sets = test_sets.sets.iter().len();
        let baseline = find_baseline(&violations, n_test_sets as i32, baseline_threshold, overlap_overflow_threshold);

        remove_baseline_violations(&mut violations, &baseline);
    } else {
        println!("We are not finding a baseline");
    }

    return violations
}

struct AlignmentChangeKey {
    pub left_node: i32,
    pub right_node: i32,
    pub left_node_line: i32,
    pub right_node_line: i32,
}

impl Hash for AlignmentChangeKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_i32(max(self.left_node, self.right_node));
        state.write_i32(min(self.left_node, self.right_node));
        state.write_i32(max(self.left_node_line, self.right_node_line));
        state.write_i32(min(self.left_node_line, self.right_node_line));
    }
}

pub fn generate_violations_report(test_sets: &TestSets, violations: &LayoutViolations) -> String {
    let mut ret = String::new();


    let mut sorted_by_test_set: Vec<LayoutViolation> = violations.all();

    let violation_type_counts = sorted_by_test_set.iter().fold((0,0,0), |acc, v| {
        match v {
            &LayoutViolation::Overlap { .. } => (acc.0 + 1,acc.1,acc.2),
            &LayoutViolation::Overflow { .. } => (acc.0, acc.1 + 1, acc.2),
            &LayoutViolation::AlignmentLost { .. } => (acc.0, acc.1, acc.2 + 1),
        }
    });

    let total_errors = violation_type_counts.0 + violation_type_counts.1 + violation_type_counts.2;
    let total_overlaps = violation_type_counts.0;
    let total_overflows = violation_type_counts.1;
    let total_layout_changes = violation_type_counts.2;

    let mut lines = Vec::new();



    let mut test_set = -1;
    let mut violation_id = 0;
    for violation in &sorted_by_test_set {
        match violation {
            &LayoutViolation::Overlap { node1, node2, intersection_rect, test_set} => {
                let ts = test_sets.sets.get(&test_set).unwrap();

                let n1 = ts.nodes.get_from_id(node1).unwrap();
                let n2 = ts.nodes.get_from_id(node2).unwrap();
                lines.push(format!("- Overlap - test-set:{} => {} - {}", test_set, n1, n2));
            },
            &LayoutViolation::Overflow { node1, node2, ref overflow_rect, test_set } => {
                let ts = test_sets.sets.get(&test_set).unwrap();

                let n1 = ts.nodes.get_from_id(node1).unwrap();
                let n2 = ts.nodes.get_from_id(node2).unwrap();
                lines.push(format!("- Overflow - test-set:{} => {} - {}", test_set.0, n1, n2));
            },
            &LayoutViolation::AlignmentLost { a:a, b:b, count:count, test_sets: ref tss } => {
                let ts = test_sets.sets.get(&tss.get(0).unwrap()).unwrap();

                let n1_line = ts.nodes.get_from_id(a.node).unwrap().node_data.line;
                let n2_line = ts.nodes.get_from_id(b.node).unwrap().node_data.line;

                let mut l = String::new();

                write!(&mut l, "- AlignmentLost => {} @ L:{} - {} @ L:{} - aligned in {} test sets", a, n1_line, b, n2_line, count);
                lines.push(l);
            },
        }
        violation_id += 1;
    }

    writeln!(ret, "* # of test sets: - {}", test_sets.sets.iter().len());
    writeln!(ret, "* Total errors: - {}", total_errors);
    writeln!(ret, "\t* Overlaps ------------- : {}", total_overlaps);
    writeln!(ret, "\t* Overflows ------------ : {}", total_overflows);
    writeln!(ret, "\t* Total alignment changes: {}", total_layout_changes);

    for l in lines {
        writeln!(&mut ret, "{}", l);
    }

    println!("{}",ret);

    ret
}

fn main() {

    let matches = App::new("Fuse layout testing oracle")
        .version("0.0")
        .author("Kristian Fjeld Hasselknippe")
        .about("This application will test your fuse app to make sure the layout is valid on all supported screen sizes.")
        .arg(Arg::with_name("project")
             .short("p")
             .long("project")
             .help("The path to the Fuse project to be test")
             .takes_value(true))
        .arg(Arg::with_name("test_script")
             .short("t")
             .help("The path to the test script to be run for this project")
             .takes_value(true))
        .arg(Arg::with_name("auto_run")
             .short("-r")
             .help("Automatically runs the test script for all projects and ouputs images for the errors it finds"))
        .arg(Arg::with_name("baseline")
             .short("b")
             .takes_value(true)
             .help("Baseline threshold value. Number between 0.0 and 1.0, where 1 has higher chance of reporting false negatives."))
        .arg(Arg::with_name("overlap_overflow_baseline_threshold")
             .short("o")
             .takes_value(true)
             .help("Threshold for including overlaps and overflows in baseline."))
        .get_matches();


    let project_path = matches.value_of("project");
    let test_script_path = matches.value_of("test_script").unwrap();
    let auto_run = matches.is_present("auto_run");

    let baseline_match = matches.value_of("baseline");
    let oo_baseline_match = matches.value_of("overlap_overflow_baseline_threshold");

    let mut baseline_threshold = None;
    if let Some(baseline_match) = baseline_match {
        baseline_threshold = Some(baseline_match.parse::<f32>().unwrap());
    }

    let mut overlap_overflow_threshold = None;
    if let Some(oo_baseline_match) = oo_baseline_match {
        overlap_overflow_threshold = Some(oo_baseline_match.parse::<f32>().unwrap());
    }


    let (sdl_context, mut window) = init_sdl();
    let window_size = window.size();
    let mut renderer = window.renderer().build().unwrap();


    renderer.set_blend_mode(BlendMode::Blend);
    let mut event_pump = sdl_context.event_pump().unwrap();

    println!("WindowSize: {:?}", window_size);

    let ttf_context = init_ttf().unwrap();
    let current_dir = env::current_dir().unwrap();
    println!("Current directory {}", current_dir.display());
    let mut font = ttf_context.load_font("./assets/Roboto-Regular.ttf", 14).unwrap();

    let mut draw_context = DrawContext::new(window_size.clone(), font, renderer);

    let mut mouse_pos: (i32,i32) = (0,0);

    let mut mouse_clicked = false;
    let mut mouse_down = false;

    let mut last_mouse_pos = (0,0);

    let mut image_name_counter = 0;

    let mut test_set_id_counter = 0;

    let mut test_sets: Option<TestSets> = None;

    println!("We have test script path: {}", test_script_path);
    println!("Starting preview for project: {:?}", project_path);

    let test_script = TestScript::from_path(test_script_path);

    println!("Starting server, listening for app to connect");
    let mut server = Server::start_new("127.0.0.1:12345");

    if auto_run {

        for project in &test_script.projects {

            let (exe_path, name) = TestRunnerContext::build_and_get_exe_path(Path::new(&project.path));
            let mut test_runner_context = TestRunnerContext::start_preview_for_example(exe_path.as_path());

            server.wait_for_client();
            println!("Client connected");
            thread::sleep_ms(500);

            let mut test_data = Vec::new();
            test_runner_context.test_all_screen_sizes(&test_script, |id, screen_size| {


                //enter_to_continue();
                thread::sleep_ms(400);
                let (nodes, id) = server.request_layout_data(id).unwrap();
                test_data.push((TestSetId(id),nodes,screen_size));

            });

            let now = now();

            //make sure the output directories exists
            let mut p = PathBuf::new();
            p.push("output");
            let directory_name = format!("{}_{}", &name, now.ctime());
            let directory_name = directory_name.replace(" ", "_");
            let directory_name = directory_name.replace(":", "_");
            println!("DirName: {}", directory_name);
            p.push(&directory_name);
            std::fs::create_dir(p);

            let tab_stop_merge_threshold = 10.0; //TODO: THIS IS NOT IN USE
            let test_sets = Some(generate_test_sets(test_data, tab_stop_merge_threshold)).unwrap();
            let violations = run_all_tests(&mut draw_context, &test_sets, &directory_name, baseline_threshold, overlap_overflow_threshold);

            let report = generate_violations_report(&test_sets, &violations);

            let report_file_name = format!("./output/{}/report", &directory_name);
            println!("Report_file_name: {}", report_file_name);
            let mut file = std::fs::File::create(report_file_name).unwrap();
            file.write_all(report.as_bytes());

            draw_context.save_overflow_violations(&test_sets, &violations, &directory_name);
            draw_context.save_overlap_violations(&test_sets, &violations, &directory_name);
            draw_context.save_alignment_changed_violations(&test_sets, &violations, &directory_name);

            test_runner_context.kill_process();
            server.close_current_connection();
        }
    }

    /*'running: loop {

        for event in event_pump.poll_iter() {
            match event {
                Event::KeyUp { keycode: Some(Keycode::F), .. } => {
                    /*let (ts, v) = run_all_tests(&mut test_runner_context, &mut draw_context, &server);
                    test_sets = Some(ts);
                    violations = v;*/
                },
                Event::KeyUp { keycode: Some(Keycode::T), .. } => { // DRAW TAB STOPS
                    if let &Some(ref test_sets) = &test_sets {
                        draw_context.draw_tab_stops(test_sets);
                    }
                },
                Event::KeyUp { keycode: Some(Keycode::P), .. } => { // DRAW OVERLAP
                    // TODO: DRAW THOSE OVERLAPS
                    if let &Some(ref test_sets) = &test_sets {
                        //draw_context.save_overlap_violations(test_sets, &violations,"");
                    }
                },
                Event::KeyUp { keycode: Some(Keycode::W), .. } => { // DRAW OVERFLOW
                    if let &Some(ref test_sets) = &test_sets {
                        //draw_context.save_overflow_violations(test_sets, &violations,"");
                    }
                },
                Event::KeyUp { keycode: Some(Keycode::A), .. } => { // DRAW ALIGNMENT CHANGES
                    // TODO: DRAW THOSE ALIGNMENTS
                },
                Event::KeyUp { keycode: Some(Keycode::G), .. } => { //print all nodes
                    if let &Some(ref test_sets) = &test_sets {
                        for (id, t) in &test_sets.sets {
                            println!("TestSet: {}", id.0);
                            let sorted = t.nodes.sorted_by_line();
                            for &ref node in &sorted {
                                let ref data = node.node_data;
                                println!("\tNode: {}, line: {} x:{},y{}, w:{},h:{} , rw:{}, rh:{}", data.name, data.line,
                                         data.actual_position_x,
                                         data.actual_position_y,
                                         data.actual_width, data.actual_height,
                                         data.render_width, data.render_height);
                            }
                        }
                    }
                },
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::MouseMotion { x, y, .. } => {
                    //println!("MousePos: {},{}", x,y);
                    mouse_pos = (x,y);
                    let mut mouse_delta = (0,0);
                    if last_mouse_pos != (0,0) {
                        mouse_delta = (mouse_pos.0 - last_mouse_pos.0, mouse_pos.1 - last_mouse_pos.1);
                    }
                    last_mouse_pos = mouse_pos;
                    if mouse_down {
                        let new_camera_pos = (draw_context.camera_pos.0 + mouse_delta.0, draw_context.camera_pos.1 + mouse_delta.1);
                        draw_context.set_camera(new_camera_pos);
                    }
                },
                Event::MouseButtonDown { .. } => {
                    mouse_down = true;
                },
                Event::MouseButtonUp { .. } => {
                    mouse_clicked = true;
                    mouse_down = false;
                }
                _ => {}
            }
        }


    } */
}
