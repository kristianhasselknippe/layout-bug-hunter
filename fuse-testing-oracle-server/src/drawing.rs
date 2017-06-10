use sdl2::{Sdl,init};
use sdl2::render::{Renderer,TextureQuery,Texture,BlendMode};
use sdl2::pixels::{Color,PixelFormatEnum};
use sdl2::video::Window;
use sdl2::rect::{Rect,Point};
use sdl2::surface::Surface;
use sdl2::ttf::{init as init_ttf,Font,Sdl2TtfContext};
use sdl2::image::SaveSurface;

use random_color::*;
use std::cmp::Ordering;
use layout_validator::*;
use test_runner::*;
use layout_validator::validity_rules::*;
use layout_validator::overlap_and_overflow::*;
use random_color::*;
use std::collections::{HashSet,HashMap};

use std::path::Path;

use test_sets::*;


pub fn init_sdl() -> (Sdl, Window) {
    let sdl_context = init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("LayoutValidator", 600, 1200)
        .position_centered()
        .opengl()
        .resizable()
        .build()
        .unwrap();

    (sdl_context, window)
}

pub struct DrawContext<'a,'b> {
    pub camera_pos: (i32,i32),
    window_size: (i32,i32),
    random_color: RandomColor,
    random_color_counter: i32,
    pub renderer: Renderer<'a>,
    font: Font<'a,'b>,

    label_positions: HashSet<(i32,i32)>,

    textures: HashMap<&'static str,Texture>,
}

impl<'a,'b> DrawContext<'a,'b> {
    pub fn new(window_size: (u32,u32), font: Font<'a,'b>, renderer: Renderer<'a>) -> DrawContext<'a,'b> {
        let mut textures = HashMap::new();

        let grain_surf = Surface::load_bmp("assets/grain.bmp").unwrap();
        let pixels_surf = Surface::load_bmp("assets/pixels.bmp").unwrap();
        let crystal_surf = Surface::load_bmp("assets/crystal.bmp").unwrap();

        let mut grain_tex = renderer.create_texture_from_surface(grain_surf).unwrap();
        let mut pixels_tex = renderer.create_texture_from_surface(pixels_surf).unwrap();
        let mut crystal_tex = renderer.create_texture_from_surface(crystal_surf).unwrap();

        grain_tex.set_blend_mode(BlendMode::Blend);
        grain_tex.set_alpha_mod(0x99);


        textures.insert("grain", grain_tex);
        textures.insert("pixels", pixels_tex);
        textures.insert("crystal", crystal_tex);

        DrawContext {
            camera_pos: (0,0),
            window_size: (window_size.0 as i32, window_size.1 as i32),
            random_color: RandomColor::new(),
            random_color_counter: 0,
            renderer: renderer,
            font: font,
            label_positions: HashSet::new(),
            textures: textures,
        }
    }

    pub fn set_camera(&mut self, pos: (i32,i32)) {
        self.camera_pos = pos;
    }

    pub fn present(&mut self) {
        self.random_color_counter = 0;
        self.renderer.present();
    }

    pub fn clear(&mut self) {
        self.renderer.set_draw_color(Color::RGBA(255,255,255,255));
        self.renderer.clear();
    }

    pub fn draw_line(&mut self, color: (u8,u8,u8), p1: (i32,i32), p2: (i32,i32)) {
        self.renderer.set_draw_color(Color::RGBA(color.0, color.1, color.2, 0xff));
        let p1 = Point::new(p1.0, p1.1);
        let p2 = Point::new(p2.0, p2.1);
        self.renderer.draw_line(p1, p2);
    }

    pub fn draw_text(&mut self, label: String, color: (u8,u8,u8,u8), pos: (i32,i32)) -> (u32,u32) {
        let l = label;
        let surface = self.font.render(&l).blended(Color::RGBA(color.0,color.1,color.2,color.3)).unwrap();
        let mut texture = self.renderer.create_texture_from_surface(&surface).unwrap();
        let TextureQuery { width, height, .. } = texture.query();

        let half_padding: i32 = 2;
        self.renderer.set_draw_color(Color::RGBA(0,0,0,0x44));

        //let mut pos = (pos.0 + self.camera_pos.0, pos.1 + self.camera_pos.1);

        /*if self.label_positions.contains(&pos) {
            pos = (pos.0,pos.1 + height as i32);
        }*/
        self.label_positions.insert(pos);

        self.renderer.fill_rect(Rect::new(pos.0 as i32, pos.1 as i32,
                                          width + (half_padding * 4) as u32, height + (half_padding * 4) as u32));
        self.renderer.copy(&mut texture, None,
                           Some(Rect::new((pos.0 + half_padding) as i32, (pos.1 + half_padding) as i32,
                                          width as u32, height as u32))).unwrap();
        (width,height)
    }

    pub fn draw_tab_stop(&mut self, tab_stop: &TabStop, label: String, mouse_pos: (i32,i32), mouse_clicked: bool) {

        //these guys don't do anything anymore
        let mut counter = 0;
        let mut label_counter_v = 0;
        let mut label_counter_h = 0;

        let pos = tab_stop.pos;
        let ref orientation = tab_stop.orientation;
        let root_size = self.window_size;
        let c = (0x00, 0x00, 0x00);
        self.renderer.set_draw_color(Color::RGBA(c.0, c.1, c.2, 0xff));

        let half_padding: i32 = 2;
        self.renderer.set_draw_color(Color::RGBA(0,0,0,255));

        let mut x: i32;
        let mut y: i32;
        let mut line_width: u32 = 2;

        let mut mouse_over = false;
        match orientation {
            &Orientation::Horizontal => {
                y = pos + self.camera_pos.1;
                x = 50 + (label_counter_v * 50) as i32 + self.camera_pos.0;
                label_counter_v += 1;



                let mp = mouse_pos;
                if (mp.1 - pos).abs() < 15 {
                    self.renderer.set_draw_color(Color::RGBA(0x00,0x55,0x55,0xff));
                    line_width = 4;
                    mouse_over = true;
                }
            },
            &Orientation::Vertical => {
                y = (root_size.1 / 2) + (label_counter_h * 20) as i32 + self.camera_pos.1;
                x =  pos + self.camera_pos.0;
                label_counter_h += 1;

                let mp = mouse_pos;
                if (mp.0 - pos).abs() < 15 {
                    self.renderer.set_draw_color(Color::RGBA(0x55,0x00,0x55,0xff));
                    line_width = 4;
                    mouse_over = true;
                }
            },
        }

        match orientation {
            &Orientation::Horizontal => {
                self.renderer.fill_rect(Rect::new(0 + self.camera_pos.0, pos + self.camera_pos.1, root_size.0 as u32, line_width));
            },
            &Orientation::Vertical => {
                self.renderer.fill_rect(Rect::new(pos + self.camera_pos.0, 0 + self.camera_pos.1, line_width, root_size.1 as u32));
            },
        }

        counter += 1;

        if mouse_over {
            self.draw_text(label, (255,255,255,255), (x,y));
        }
    }

    pub fn draw_node_label(&mut self, n: &Node) {
        let data = &n.node_data;
        let x = data.actual_position_x + self.camera_pos.0;
        let y = data.actual_position_y + self.camera_pos.1;
        let w = data.actual_width as u32;
        let h = data.actual_height as u32;
        let rect = Rect::new(x,y,w,h);
        let label = format!("ID:{}", n.id);
        let (w,h) = self.draw_text(label, (255,255,255,255), (x + (w/2) as i32,y + (h/2) as i32));
    }

    pub fn draw_rect(&mut self, r: Rect, color: (u8,u8,u8,u8)) {
        self.renderer.set_draw_color(Color::RGBA(color.0, color.1, color.2, color.3));
        let x = r.left() + self.camera_pos.0;
        let y = r.top() + self.camera_pos.1;
        let mut r = r.clone();
        r.set_x(x);
        r.set_y(y);
        self.renderer.fill_rect(r);
    }

    pub fn draw_grainy_rect(&mut self, r: Rect) {
        let grainy_tex = self.textures.get("grain").unwrap();

        self.renderer.copy(grainy_tex, None, Some(r));
    }

    pub fn stroke_rect(&mut self, r: Rect, color: (u8,u8,u8,u8)) {
        self.renderer.set_draw_color(Color::RGBA(color.0, color.1, color.2, color.3));
        let x = r.left() + self.camera_pos.0;
        let y = r.top() + self.camera_pos.1;
        let mut r = r.clone();
        r.set_x(x);
        r.set_y(y);
        self.renderer.draw_rect(r);
    }

    pub fn draw_node(&mut self, n: &Node) {

        let data = &n.node_data;
        let x = data.render_position_x + self.camera_pos.0;
        let y = data.render_position_y + self.camera_pos.1;
        let w = data.render_width as u32;
        let h = data.render_height as u32;
        let rect = Rect::new(x,y,w,h);
        let c = self.random_color.get_a_color(n.id as usize);
        self.renderer.set_draw_color(Color::RGBA(c.0, c.1, c.2, 0xff));
        self.renderer.fill_rect(rect);
        self.random_color_counter += 1;
    }

    pub fn draw_nodes(&mut self, nodes: &Nodes) {
        let mut sorted_nodes = nodes.nodes.iter().map(|(id,n)| { (id.clone(), n.clone()) }).collect::<Vec<(i32,Node)>>();

        sorted_nodes.sort_by(|&(ref id1, ref n1),&(ref id2, ref n2)| {
            let wh1 = ((n1.node_data.render_width.pow(2) + n1.node_data.render_height.pow(2)) as f32).sqrt() as i32;
            let wh2 = ((n2.node_data.render_width.pow(2) + n2.node_data.render_height.pow(2)) as f32).sqrt() as i32;

            return wh2.cmp(&wh1);
        });

        for &(ref id, ref n) in &sorted_nodes {
            self.draw_node(&n);
        }
    }

    pub fn save_to_png(&self, path: &str) {
        let window_size = self.renderer.window().unwrap().size();
        if let Ok(mut pixels) = self.renderer.read_pixels(None, PixelFormatEnum::RGBA8888) {
            let save_surface = Surface::from_data(&mut pixels,
                                                  window_size.0,
                                                  window_size.1,
                                                  window_size.0 * 4,
                                                  PixelFormatEnum::RGBA8888).unwrap();
            save_surface.save(path);
        }
    }

    pub fn draw_tab_stops(&mut self, test_sets: &TestSets) {
        let mouse_pos = (0,0);
        for (id, test_set) in test_sets.sets.iter() {
            let ref nodes = test_set.nodes;
            let ref tab_stops = test_set.tab_stops;
            let ref ss = test_set.screen_size;
            self.clear();
            self.draw_nodes_and_tab_stops(&nodes, &tab_stops, mouse_pos);
            self.present();
            let save_path = format!("output/tab_stops{}x{}.png", ss.0, ss.1);
            self.save_to_png(&save_path);
        }

    }

    pub fn draw_violations(&mut self, layout_violations: &LayoutViolations, nodes: &Nodes) {
        for violation in &layout_violations.all() {
            match violation {
                &LayoutViolation::Overlap {
                    ref test_set,
                    ref node1,
                    ref node2,
                    ref intersection_rect,
                } => {
                    self.draw_rect(intersection_rect.clone(), (0,0,0,255));
                },
                &LayoutViolation::Overflow {
                    ref test_set,
                    ref node1,
                    ref node2,
                    ref overflow_rect,
                } => {
                    match overflow_rect {
                        &OverflowRect::Partial { left, top, right, bottom } => {
                            if let Some(left) = left {
                                self.draw_rect(left.clone(), (0x88,0,0,255));
                            }
                            if let Some(top) = top {
                                self.draw_rect(top.clone(), (0x88,0,0,255));
                            }
                            if let Some(right) = right {
                                self.draw_rect(right.clone(), (0x88,0,0,255));
                            }
                            if let Some(bottom) = bottom {
                                self.draw_rect(bottom.clone(), (0x88,0,0,255));
                            }
                        },
                        &OverflowRect::Complete( r ) => {
                            self.draw_rect(r.clone(), (0x77,0x33,0,255));
                        },
                    }
                },
                &LayoutViolation::AlignmentLost { a, b, count, .. } => {
                    let p1_data = &nodes.get_from_id(a.node).unwrap().node_data;
                    let p2_data = &nodes.get_from_id(b.node).unwrap().node_data;
                    let p1 = (p1_data.actual_position_x + (p1_data.actual_width / 2), p1_data.actual_position_y + (p1_data.actual_height / 2));
                    let p2 = (p2_data.actual_position_x + (p2_data.actual_width / 2), p2_data.actual_position_y + (p2_data.actual_height / 2));
                    println!("Drawing line from {:?} to {:?}", p1, p2);
                    self.draw_line((0xff,0x00,0x00), p1, p2);
                },
            }
        }
    }

    pub fn draw_nodes_and_tab_stops(&mut self, nodes: &Nodes, tab_stops: &TabStops, mouse_pos: (i32,i32)) {
        self.clear();
        //clear
        self.draw_nodes(nodes);

        let mut sorted_nodes = nodes.nodes.iter().map(|(id,n)| { (id.clone(), n.clone()) }).collect::<Vec<(i32,Node)>>();
        for &(ref id, ref n) in &sorted_nodes {
            self.draw_node_label(&n);
        }
    }

    pub fn draw_test_set(&mut self, test_set: &TestSet) {
        self.draw_nodes(&test_set.nodes);
    }

    pub fn save_overlap_violations(&mut self, test_sets: &TestSets, violations: &LayoutViolations, folder: &str) {
        for violation in &violations.all() {


            match violation {
                &LayoutViolation::Overlap { node1, node2, intersection_rect, test_set } => {
                    let test_set = test_sets.sets.get(&test_set).unwrap();

                    let ref nodes = test_set.nodes;
                    let ss = test_set.screen_size;

                    self.renderer.window_mut().unwrap().set_size(ss.0 as u32, ss.1 as u32);

                    self.clear();
                    self.draw_nodes(&nodes);
                    self.draw_rect(Rect::new(0,0,ss.0 as u32,ss.1 as u32), (0xff,0xff,0xff,0xdd));

                    let n1 = nodes.get_from_id(node1).unwrap();
                    let n2 = nodes.get_from_id(node2).unwrap();
                    self.draw_node(&n1);
                    self.draw_node(&n2);
                    self.draw_grainy_rect(intersection_rect);

                    let ref n1 = nodes.get_from_id(node1).unwrap().node_data;
                    let ref n2 = nodes.get_from_id(node2).unwrap().node_data;

                    let save_path = format!("output/{}/overlap-{}_{}-{}x{}.png",folder,n1.line,n2.line, ss.0, ss.1);
                    println!("savepath: {}", save_path);
                    self.save_to_png(&save_path);

                    self.present();
                },
                _ => (),
            }
        }
    }

    pub fn save_overflow_violations(&mut self, test_sets: &TestSets, violations: &LayoutViolations, folder: &str) {
        for violation in &violations.all() {
            match violation {

                &LayoutViolation::Overflow { node1, node2, ref overflow_rect, test_set } => {

                    let test_set = test_sets.sets.get(&test_set).unwrap();

                    let ref nodes = test_set.nodes;
                    let ss = test_set.screen_size;


                    self.renderer.window_mut().unwrap().set_size(ss.0 as u32, ss.1 as u32);

                    let ref n1 = nodes.get_from_id(node1).unwrap().node_data;
                    let ref n2 = nodes.get_from_id(node2).unwrap().node_data;

                    self.clear();
                    self.draw_nodes(&nodes);
                    self.draw_rect(Rect::new(0,0,ss.0 as u32,ss.1 as u32), (0xff,0xff,0xff,0xdd));

                    let n_1 = nodes.get_from_id(node1).unwrap();
                    let n_2 = nodes.get_from_id(node2).unwrap();
                    self.draw_node(&n_1);
                    self.draw_node(&n_2);

                    match overflow_rect {
                        &OverflowRect::Partial { left, top, right, bottom } => {
                            if let Some(left) = left { self.draw_grainy_rect(left) }
                            if let Some(top) = top { self.draw_grainy_rect(top) }
                            if let Some(right) = right { self.draw_grainy_rect(right) }
                            if let Some(bottom) = bottom { self.draw_grainy_rect(bottom) }
                        },
                        &OverflowRect::Complete(rect) => {
                            self.draw_grainy_rect(rect);
                        },
                    }

                    let save_path = format!("output/{}/overflow-L{}_L{}-{}x{}.png",
                                            folder,
                                            n1.line,n2.line,
                                            ss.0, ss.1);
                    println!("savepath: {}", save_path);
                    self.save_to_png(&save_path);
                    self.present();
                },
                _ => (),
            }
        }
    }

    pub fn save_alignment_changed_violations(&mut self, test_sets: &TestSets, violations: &LayoutViolations, folder: &str) {
        for violation in &violations.all() {
            match violation {
                &LayoutViolation::AlignmentLost { a: a,b: b, count: count,test_sets: ref violation_test_sets } => {

                    let ts_ = violation_test_sets.get(0).unwrap();

                    let test_set = test_sets.sets.get(&ts_).unwrap();

                    let ref nodes = test_set.nodes;
                    let ss = test_set.screen_size;

                    self.renderer.window_mut().unwrap().set_size((ss.0 as u32) * 2, (ss.1 as u32));

                    self.clear();
                    self.draw_nodes(&nodes);
                    self.draw_rect(Rect::new(0,0,ss.0 as u32,ss.1 as u32), (0xff,0xff,0xff,0xdd));

                    let n1 = &nodes.get_from_id(a.node).unwrap();
                    let n2 = &nodes.get_from_id(b.node).unwrap();

                    self.draw_node(n1);
                    self.draw_node(n2);

                    let ts1 = test_set.tab_stops.tab_stop_connected_to_node_side(&a);
                    let ts2 = test_set.tab_stops.tab_stop_connected_to_node_side(&b);

                    self.draw_tab_stop(&ts1, "node".to_string(), (0,0), false);
                    self.draw_tab_stop(&ts2, "lost alignment to".to_string(), (0,0), false);

                    let ref p1_data = n1.node_data;
                    let ref p2_data = n2.node_data;
                    let p1 = (p1_data.actual_position_x + (p1_data.actual_width / 2), p1_data.actual_position_y + (p1_data.actual_height / 2));
                    let p2 = (p2_data.actual_position_x + (p2_data.actual_width / 2), p2_data.actual_position_y + (p2_data.actual_height / 2));
                    self.draw_line((0xff,0x00,0x00), p1, p2);

                    let save_path = format!("output/{}/alignment_change--A{}-B{}--{}x{}--L{}_L{}.png",
                                            folder,
                                            a.node, b.node,
                                            ss.0, ss.1,
                                            p1_data.line,p2_data.line);

                    let save_path = save_path.replace(":","-");

                    //TODO: this is not the best place to do this check, but it should be ok for now

                    if !Path::new(&save_path).exists() {
                        println!("savepath: {}", save_path);
                        self.save_to_png(&save_path);
                    }
                    self.present();

                },
                _ => (),
            }
        }
    }
}
