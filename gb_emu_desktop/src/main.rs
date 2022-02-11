mod config;

use config::*;
use gb_emu_common::cartridge::header::Header;
use gilrs::{Event as GamepadEvent, Gilrs};
use macroquad::prelude::*;
use once_cell::sync::Lazy;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Mutex;

#[cfg(not(target_family = "wasm"))]
use directories::UserDirs;
#[cfg(not(target_family = "wasm"))]
use native_dialog::FileDialog;

#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;
#[cfg(target_family = "wasm")]
use wasm_bindgen::JsCast;
#[cfg(target_family = "wasm")]
use web_sys::{Event, FileReader, HtmlInputElement};

#[cfg(target_family = "wasm")]
extern "C" {
    fn js_open_file();
    fn js_open_fullscreen();
}

const GB_SCREEN_WIDTH: f32 = 160.;
const GB_SCREEN_HEIGHT: f32 = 144.;
const MENU_BAR_HEIGHT: f32 = 23.;

static WASM_ROM: Lazy<Mutex<Option<Vec<u8>>>> = Lazy::new(|| Mutex::new(None));

struct State {
    pub quit: bool,
    pub show_menu_bar: bool,
    pub is_fullscreen: bool,
    pub show_rom_info_window: bool,
    pub rom_info_description: Option<String>,
    pub is_waiting_file_callback: bool,
    pub last_used_dir: Option<String>,
    pub last_gamepad_event: Option<String>,
}

impl State {
    pub fn new() -> State {
        let last_used_dir = read_config(ConfigFile::LastUsedDirectory).unwrap_or(None);
        State {
            quit: false,
            show_menu_bar: true,
            is_fullscreen: false,
            show_rom_info_window: false,
            rom_info_description: None,
            is_waiting_file_callback: false,
            last_used_dir,
            last_gamepad_event: None,
        }
    }
}

fn conf() -> Conf {
    Conf {
        window_title: String::from("RustBoy"),
        fullscreen: false,
        ..Default::default()
    }
}

enum FullscreenEvent {
    Enter,
    Leave,
    None,
}

struct WebEvents {
    fullscreen_event: FullscreenEvent,
}

impl WebEvents {
    fn new() -> WebEvents {
        WebEvents {
            fullscreen_event: FullscreenEvent::None,
        }
    }
}

#[macroquad::main(conf)]
async fn main() {
    let mut gilrs = Gilrs::new().unwrap();
    let mut state = State::new();
    let web_events: Rc<RefCell<WebEvents>> = Rc::new(RefCell::new(WebEvents::new()));

    cfg_if::cfg_if! {
        if #[cfg(target_family = "wasm")] {
            let window = web_sys::window().expect("No global `window` exists");
            let document = window.document().expect("Should have a document on window");

            let canvas = document
                .query_selector("canvas#glcanvas")
                .expect("Could not find canvas element")
                .expect("Could not find canvas element");

            let events = web_events.clone();
            let document_clone = document.clone();
            let closure = Closure::wrap(Box::new(move |event: web_sys::Event| {
                match document_clone.fullscreen_element() {
                    Some(_) => {
                        // entering fullscreen
                        events.borrow_mut().fullscreen_event = FullscreenEvent::Enter;
                    }

                    None => {
                        // leaving fullscreen
                        events.borrow_mut().fullscreen_event = FullscreenEvent::Leave;
                    }
                }
                //panic!("Test")
            }) as Box<dyn FnMut(_)>);
            document.add_event_listener_with_callback("fullscreenchange", closure.as_ref().unchecked_ref()).unwrap();
            closure.forget();
        }
    };

    loop {
        clear_background(BLACK);

        if state.quit {
            break;
        }

        // This is a hack used to load files on WASM
        if state.is_waiting_file_callback {
            let mut mutex = WASM_ROM.lock().unwrap();
            if let Some(rom) = mutex.as_ref() {
                let header = Header::read_rom_header(&rom).expect("Error while reading ROM header");
                let rom_title = header.title.unwrap_or(String::from("NO TITLE"));
                let cartridge_type = header.cartridge_type;
                let file_size = rom.len();
                let rom_banks = header.rom_bank_amount;
                let description = format!(
                    "\
                    Title: {rom_title}\n\
                    Cartridge type: {cartridge_type}\n\
                    File size: {file_size} bytes\n\
                    ROM banks: {rom_banks}\
                    "
                );

                state.rom_info_description = Some(description);
                state.show_rom_info_window = true;
                state.is_waiting_file_callback = false;
                *mutex = None;
            }
        }

        if cfg!(target_family = "wasm") {
            let mut web_events = web_events.borrow_mut();
            match web_events.fullscreen_event {
                FullscreenEvent::Enter => {
                    state.show_menu_bar = false;
                    web_events.fullscreen_event = FullscreenEvent::None;
                }

                FullscreenEvent::Leave => {
                    state.show_menu_bar = true;
                    web_events.fullscreen_event = FullscreenEvent::None;
                }

                _ => {}
            }
        }

        // Process keys, mouse etc.

        if is_key_pressed(KeyCode::RightAlt) {
            state.show_menu_bar = !state.show_menu_bar;
        }

        // Gamepad events
        while let Some(GamepadEvent { id, event, time }) = gilrs.next_event() {
            let event_description = format!("{:?} New event from {}: {:?}", time, id, event);
            state.last_gamepad_event = Some(event_description);
            //active_gamepad = Some(id);
        }

        egui_macroquad::ui(|ctx| {
            if state.show_menu_bar {
                egui::TopBottomPanel::top("top_panel").show(&ctx, |ui| {
                    egui::menu::bar(ui, |ui| {
                        ui.menu_button("File", |ui| {
                            if ui.button("Open").clicked() {
                                handle_open_file_btn_click(&mut state);
                                ui.close_menu();
                            }

                            if cfg!(not(target_family = "wasm")) {
                                if ui.button("Quit").clicked() {
                                    state.quit = true;
                                    ui.close_menu();
                                }
                            }
                        });

                        if cfg!(target_family = "wasm") {
                            ui.menu_button("View", |ui| {
                                if ui.button("Fullscreen").clicked() {
                                    open_fullscreen();
                                    //state.show_menu_bar = false;
                                    ui.close_menu();
                                }
                            });
                        }
                    });
                });
            }

            if let Some(description) = &state.rom_info_description {
                egui::Window::new("ROM info")
                    .open(&mut state.show_rom_info_window)
                    .show(&ctx, |ui| {
                        ui.label(description);
                    });
            }

            if let Some(event) = &state.last_gamepad_event {
                egui::Window::new("Gamepad event").show(ctx, |ui| {
                    ui.label(event);
                });
            }
        });

        // draw things before egui

        let menu_bar_end = MENU_BAR_HEIGHT + 1.0;
        let screen_height = if state.show_menu_bar {
            screen_height() - menu_bar_end
        } else {
            screen_height()
        };
        let screen_width = screen_width();

        let (w, h) = scale_image(
            GB_SCREEN_WIDTH,
            GB_SCREEN_HEIGHT,
            screen_width,
            screen_height,
        );
        let x = (screen_width - w) / 2.0;
        let y = (screen_height - h) / 2.0;
        let offset_y = if state.show_menu_bar {
            menu_bar_end
        } else {
            0.0
        };
        // draw gb screen
        draw_rectangle(x, y + offset_y, w, h, WHITE);

        // draw egui
        egui_macroquad::draw();

        next_frame().await;
    }
}

#[cfg(target_family = "wasm")]
fn handle_open_file_btn_click(state: &mut State) {
    unsafe {
        js_open_file();
    }

    state.is_waiting_file_callback = true;
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen]
pub fn js_open_file_callback(buffer: js_sys::Uint8Array) {
    let rom = buffer.to_vec();
    *WASM_ROM.lock().unwrap() = Some(rom);
}

#[cfg(not(target_family = "wasm"))]
fn handle_open_file_btn_click(state: &mut State) {
    let last_folder_path = state
        .last_used_dir
        .clone()
        .map(|last_folder| PathBuf::from(last_folder));
    let rom_path = open_file(&last_folder_path).expect("Could not read file");

    if let Some(rom_path) = rom_path {
        let mut current_folder = PathBuf::from(&rom_path);
        current_folder.pop(); // remove file name from path

        if let Some(last_folder_path) = last_folder_path {
            if last_folder_path != current_folder {
                if let Some(current_folder) = current_folder.to_str() {
                    save_config(ConfigFile::LastUsedDirectory, current_folder)
                        .expect("Error saving config");
                    state.last_used_dir = Some(String::from(current_folder));
                }
            }
        } else {
            if let Some(current_folder) = current_folder.to_str() {
                save_config(ConfigFile::LastUsedDirectory, current_folder)
                    .expect("Error saving config");
                state.last_used_dir = Some(String::from(current_folder));
            }
        }

        let rom = std::fs::read(&rom_path).expect("Could not read file");
        let header = Header::read_rom_header(&rom).expect("Error while reading ROM header");
        let rom_title = header.title.unwrap_or(String::from("NO TITLE"));
        let file_name: &str = Path::new(&rom_path)
            .file_name()
            .map(|file_name| file_name.to_str())
            .flatten()
            .unwrap_or(&rom_path);
        let cartridge_type = header.cartridge_type;
        let file_size = rom.len();
        let rom_banks = header.rom_bank_amount;
        let description = format!(
            "\
            Title: {rom_title}\n\
            File name: {file_name}\n\
            Cartridge type: {cartridge_type}\n\
            File size: {file_size} bytes\n\
            ROM banks: {rom_banks}\
            "
        );
        state.rom_info_description = Some(description);
        state.show_rom_info_window = true // Show ROM information window
    }
}

#[cfg(target_family = "wasm")]
fn open_fullscreen() {
    unsafe {
        js_open_fullscreen();
    }
}

#[cfg(not(target_family = "wasm"))]
fn open_fullscreen() {}

fn scale_image(src_width: f32, src_height: f32, max_width: f32, max_height: f32) -> (f32, f32) {
    let ratio_w = max_width / src_width;
    let ratio_h = max_height / src_height;

    if ratio_w < ratio_h {
        let resize_width = max_width;
        let resize_height = (ratio_w * src_height).round();
        (resize_width, resize_height)
    } else {
        let resize_width = (ratio_h * src_width).round();
        let resize_height = max_height;
        (resize_width, resize_height)
    }
}

#[cfg(not(target_family = "wasm"))]
fn open_file(last_folder: &Option<PathBuf>) -> Result<Option<String>, native_dialog::Error> {
    let home_path = match UserDirs::new() {
        Some(user_dirs) => user_dirs.home_dir().to_owned(),
        None => PathBuf::from(""),
    };

    let start_path = last_folder.to_owned().unwrap_or(home_path.to_owned());
    let path = FileDialog::new()
        .set_location(&start_path)
        .add_filter("GB/GBC ROM", &["gb", "gbc"])
        .add_filter("All files", &["*"])
        .show_open_single_file()?
        .map(|path| {
            path.into_os_string()
                .into_string()
                .expect("Error converting OsString to String")
        });

    Ok(path)
}
