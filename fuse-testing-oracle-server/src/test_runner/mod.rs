pub mod test_script;

use test_script::*;
use std::thread;

use std::collections::HashSet;
use std::process::{Command,Child};
use std::thread::spawn;

use std::path::{Path,PathBuf};

use super::layout_validator::{Nodes,TabStops};

#[cfg(target_os = "windows")]
use winapi::windef::{HWND,HBITMAP};
#[cfg(target_os = "windows")]
use winapi::minwindef::{LPARAM, BOOL, LPDWORD};
#[cfg(target_os = "windows")]
use winapi::wingdi::{HORZSIZE,VERTSIZE,SRCCOPY,BITMAP,LPBITMAPINFO,BITMAPINFO};
#[cfg(target_os = "windows")]
use user32::{EnumWindows, GetWindowThreadProcessId, MoveWindow, PrintWindow, GetDC};
#[cfg(target_os = "windows")]
use gdi32::{CreateCompatibleDC,CreateCompatibleBitmap,GetDeviceCaps,BitBlt,SelectObject,DeleteDC,GetPixel,GetDIBits};
#[cfg(target_os = "windows")]
use kernel32::GetLastError;

use std::mem::transmute;
use std::sync::mpsc::channel;

pub enum TestInstruction {
    ResizeWindow(i32, i32, i32, i32)
}

#[cfg(target_os = "windows")]
static mut FUSE_WINDOW_HANDLE: Option<HWND> = None;

#[cfg(target_os = "windows")]
unsafe extern "system" fn window_enum_proc(window_handle: HWND, param: LPARAM) -> BOOL {

    let mut process_id : u32 = 1;
    let mut window_process: LPDWORD = &mut process_id as *mut u32;
    let thread_id = GetWindowThreadProcessId(window_handle, window_process);

    if param == process_id as i64 {
        //println!("we found our window handle");
        //println!("Process for window handle {:?} is {:?}, we are looking for {:?}", window_handle, process_id, param);

        /*let mut zorder = 0;
        let topmost = zorder as HWND;
        SetWindowPos(window_handle, topmost, 100, 100, 200, 200, 0x0040);*/

        FUSE_WINDOW_HANDLE = Some(window_handle.clone());
        return 0;
    }
    1
}

#[cfg(target_os = "windows")]
fn set_window_size_for_proc_id(proc_id: u32, pos: (i32, i32), size: (i32, i32)) -> i64 {
    unsafe {
        EnumWindows(Some(window_enum_proc), proc_id as i64);

        if let Some(window_handle) = FUSE_WINDOW_HANDLE {

            let moved_result = MoveWindow(window_handle, pos.0, pos.1, size.0, size.1, 0);
            let last_error = GetLastError();

            return transmute::<HWND,i64>(window_handle)
        } else {
            panic!("Could not move window");
        }
    }
    panic!("Coult not move window_");
}

pub struct TestRunnerContext {
    process: Child
}

pub fn build_project(project_path: &Path) {
    println!("Building project: {:?}", project_path);
    let project_path = project_path.to_str().unwrap();
    let process = if cfg!(any(target_os = "windows", target_os = "macos")) {
        Command::new("uno")
            .arg("build")
            .arg(project_path)
            .arg("-tdotnet")
            .spawn()
    } else {
        panic!("os not supported");
    };
    let status = process.unwrap().wait().unwrap();
    println!("Building project done with status: {:?}", status);
}

impl TestRunnerContext {

    pub fn start_preview_for_example(project_path: &Path) -> TestRunnerContext {
        println!("starting preview process");

        println!("We have project path");
        let path_string = project_path.to_str().unwrap();

        let process = if cfg!(any(target_os = "windows", target_os = "macos")) {
            println!("program path: {}", &path_string);
            Command::new(&path_string).spawn() //since we need to use uno instead of fuse preview (for testing), we need to start the compiled exe
        } else {
            panic!("unsupported os");
        };

        TestRunnerContext {
            process: process.unwrap()
        }
    }

    pub fn kill_process(&mut self) {
        self.process.kill();
    }

    pub fn build_and_get_exe_path(project_path: &Path) -> (PathBuf, String) {
        build_project(project_path);

        let file_name = project_path.file_name().unwrap().to_str().unwrap();
        let file_stem = file_name.split('.').nth(0).unwrap();

        let exe_name = format!("{}.exe", file_stem);
        println!("exe_name: {}", &exe_name);


        let mut exe_path = project_path.to_path_buf();
        exe_path.pop();
        exe_path.push("build");
        exe_path.push("DotNet");
        exe_path.push("Debug");
        exe_path.push(&exe_name);

        println!("Exe path: {:?}", exe_path);

        (exe_path, file_stem.to_string())
    }

    pub fn capture_window(&self, window_handle: i64) {
        /*unsafe {
            let window_dc = GetDC(window_handle);
            let memory_dc = CreateCompatibleDC(window_dc);

            let window_width = GetDeviceCaps(window_dc, HORZSIZE);
            let window_height = GetDeviceCaps(window_dc, VERTSIZE);

            let bitmap = CreateCompatibleBitmap(window_dc, window_width, window_height);
            let old_bitmap = SelectObject(memory_dc, bitmap);

            BitBlt(memory_dc,0,0,window_width,window_height, window_dc,0,0,SRCCOPY);

            let bitmap_handle = SelectObject(memory_dc, old_bitmap) as HBITMAP;


            let mut* bitmap_info = BIAMPINFO {
                bmiHeader:
            } as LPBITMAPINFO;
            GetDIBits(memory_dc, bitmap_handle, 0,0, 0, )



            DeleteDC(window_dc);
            DeleteDC(memory_dc);

            let bitmap_object = GetObjectA(bitmap_obj, bitmap_handle) as BITMAP;
        }*/
    }

    pub fn test_all_screen_sizes<F>(&mut self, test_script: &TestScript, mut request_layout: F)
        where F: FnMut(i32, (i32,i32)) {

        let mut id = 0;
        for screen_size in &test_script.screen_sizes {

            let ss = ((screen_size.width as f32 / screen_size.pixels_per_point) as i32, (screen_size.height as f32 / screen_size.pixels_per_point) as i32);

            println!("Testing screen size: {:?}", ss);

            let proc_id = self.process.id();

            println!("Proc id: {}", proc_id);

            let window_handle = set_window_size_for_proc_id(proc_id, (0,0), ss);

            request_layout(id, (screen_size.width, screen_size.height));
            id += 1;
        }
    }
}
