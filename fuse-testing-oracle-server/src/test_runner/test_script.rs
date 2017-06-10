use serde_yaml;
use std::fs::File;
use std::io::Read;

#[derive(Deserialize)]
pub enum ScreenOrientation {
    Portrait,
    Landscape
}

fn pixels_per_point_default() -> f32 { 1.0 }
fn physical_pixels_per_inch_default() -> f32 { 1.0 }
fn physical_pixels_per_pixel_default() -> f32 { 1.0 }
fn default_orientation_default() -> ScreenOrientation { ScreenOrientation::Portrait }

#[derive(Deserialize)]
pub struct ScreenSize {
    #[serde(rename = "Name")] pub name: String,
    #[serde(rename = "Width")] pub width: i32,
    #[serde(rename = "Height")] pub height: i32,
    #[serde(rename = "PixelsPerPoint", default = "pixels_per_point_default" )]
    pub pixels_per_point: f32,
    #[serde(rename = "PhysicalPixelsPerInch", default = "physical_pixels_per_inch_default")]
    pub physical_pixels_per_inch: f32,
    #[serde(rename = "PhysicalPixelsPerPixel", default = "physical_pixels_per_pixel_default")]
    pub physical_pixels_per_pixel: f32,
    #[serde(rename = "DefaultOrientation", default= "default_orientation_default")]
    pub default_orientation: ScreenOrientation,
}

#[derive(Deserialize)]
pub struct Project {
    #[serde(rename = "Path")] pub path: String,
}

#[derive(Deserialize)]
pub struct TestScript {
    #[serde(rename = "ScreenSizes")] pub screen_sizes: Vec<ScreenSize>,
    #[serde(rename = "Projects")] pub projects: Vec<Project>,
}

fn parse_test_script(path: &str) -> Option<TestScript> {
    println!("We are parsing testscript {}", path);

    if let Ok(mut file) = File::open(path) {
        let mut yaml_str = String::new();
        if let Ok(bytes_read) = file.read_to_string(&mut yaml_str) {
            println!("We read some bytes ({}) from yaml script", bytes_read);
            return serde_yaml::from_str(&yaml_str).unwrap();
        }
    }

    None
}

impl TestScript {
    pub fn from_path(path: &str) -> TestScript {
        let ret = parse_test_script(path).unwrap();
        for ss in &ret.screen_sizes {
            println!("ScreenSize: {},{}", ss.width, ss.height);
        }
        ret
    }
}
