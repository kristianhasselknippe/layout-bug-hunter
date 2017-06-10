use rand::{Rng,thread_rng};
use std::cell::RefCell;

pub struct RandomColor {
    colors_list: Vec<(u8,u8,u8)>,
    rng: RefCell<Box<Rng>>,
}

impl RandomColor
{
    pub fn new() -> RandomColor {
        RandomColor {
            colors_list: vec![(0xff,0,0),
                              (0,0xff,0),
                              (0,0,0xff),
                              (0xff,0xff,0),
                              (0xff,0,0xff),
                              (0,0xff,0xff),
                              (0xff,0,0x44),
                              (0xb8, 0x86, 0x0b),
                              (0xbc, 0x8f, 0x8f),
                              (0xcd, 0x5c, 0x5c),
                              (0x8b, 0x45, 0x13),
                              (0xa0, 0x52, 0x2d),
                              (0xcd, 0x85, 0x3f),
                              (0xde, 0xb8, 0x87),
                              (0xf5, 0xf5, 0xdc),
                              (0xf5, 0xde, 0xb3),
                              (0xf4, 0xa4, 0x60),
                              (0xd2, 0xb4, 0x8c),
                              (0xd2, 0x69, 0x1e),
                              (0xb2, 0x22, 0x22),
                              (0xa5, 0x2a, 0x2a),
                              (0xe9, 0x96, 0x7a),
                              (0xfa, 0x80, 0x72),
                              (0xff, 0xa0, 0x7a),
                              (0xff, 0xa5, 0x00),
                              (0xff, 0x8c, 0x00),
                              (0xff, 0x7f, 0x50),
                              (0xf0, 0x80, 0x80),
                              (0xff, 0x63, 0x47),
                              (0xff, 0x45, 0x00),
                              (0xff, 0x69, 0xb4),
                              (0xff, 0x14, 0x93),
                              (0xff, 0xc0, 0xcb),
                              (0xff, 0xb6, 0xc1),
                              (0xdb, 0x70, 0x93),
                              (0xb0, 0x30, 0x60),
                              (0xc7, 0x15, 0x85),
                              (0xd0, 0x20, 0x90),
                              (0xff, 0x00, 0xff),
                              (0xee, 0x82, 0xee),
                              (0xdd, 0xa0, 0xdd),
                              (0xda, 0x70, 0xd6),
                              (0xba, 0x55, 0xd3),
                              (0x99, 0x32, 0xcc),
                              (0x94, 0x00, 0xd3),
                              (0x8a, 0x2b, 0xe2),
                              (0xa0, 0x20, 0xf0),
                              (0x93, 0x70, 0xdb),
                              (0xd8, 0xbf, 0xd8),
                              (0xcd, 0xc9, 0xc9),
                              (0x8b, 0x89, 0x89)],
            rng: RefCell::new(Box::new(thread_rng())),
        }
    }

    pub fn get_a_color(&self, index: usize) -> (u8,u8,u8) {
        self.colors_list[index % self.colors_list.len()].clone()
    }

    pub fn get_random_color(&self) -> (u8,u8,u8) {
        let mut rng = self.rng.borrow_mut();
        let list_length = self.colors_list.len();
        let i : usize = rng.gen_range(0, list_length);
        self.colors_list[i % self.colors_list.len()].clone()
    }
}
