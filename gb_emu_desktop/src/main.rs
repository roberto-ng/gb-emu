mod config;
#[cfg(target_family = "wasm")]
mod wasm;

use config::*;
use gb_emu_common::cartridge::header::Header;
use gilrs::{Event as GamepadEvent, Gilrs};
use macroquad::prelude::*;
use std::error::Error;
use std::path::{Path, PathBuf};

#[cfg(not(target_family = "wasm"))]
use directories::UserDirs;
#[cfg(not(target_family = "wasm"))]
use native_dialog::FileDialog;

#[cfg(target_family = "wasm")]
use std::cell::RefCell;
#[cfg(target_family = "wasm")]
use std::rc::Rc;
#[cfg(target_family = "wasm")]
use wasm::WebEvents;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

const GB_SCREEN_WIDTH: f32 = 160.;
const GB_SCREEN_HEIGHT: f32 = 144.;
const MENU_BAR_HEIGHT: f32 = 23.;

pub struct State {
    pub quit: bool,
    pub show_menu_bar: bool,
    pub show_rom_info_window: bool,
    pub rom_info_description: Option<String>,
    pub is_waiting_file_callback: bool,
    pub last_used_dir: Option<String>,
    pub last_gamepad_event: Option<String>,
    pub error: Option<Box<dyn Error>>,
    pub show_error: bool,
}

impl State {
    pub fn new() -> State {
        let last_used_dir = read_config(ConfigFile::LastUsedDirectory).unwrap_or(None);

        State {
            quit: false,
            show_menu_bar: true,
            show_rom_info_window: false,
            rom_info_description: None,
            is_waiting_file_callback: false,
            last_used_dir,
            last_gamepad_event: None,
            error: None,
            show_error: false,
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

fn conf() -> Conf {
    Conf {
        window_title: String::from("RustBoy"),
        fullscreen: false,
        ..Default::default()
    }
}

#[macroquad::main(conf)]
async fn main() {
    let mut gilrs = Gilrs::new().unwrap();
    let mut state = State::new();

    #[cfg(target_family = "wasm")]
    let web_events: Rc<RefCell<WebEvents>> = Rc::new(RefCell::new(WebEvents::new()));

    // Set up event listeners on the web version
    cfg_if::cfg_if! {
        if #[cfg(target_family = "wasm")] {
            wasm::add_event_listeners(web_events.clone());
        }
    };

    loop {
        clear_background(BLACK);

        if state.quit {
            break;
        }

        // Handle events specific to the web version
        cfg_if::cfg_if! {
            if #[cfg(target_family = "wasm")] {
                let result = wasm::handle_web_events(&web_events, &mut state);
                if let Err(err) = result {
                    state.error = Some(err);
                    state.show_error = true;
                }
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
                egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                    egui::menu::bar(ui, |ui| {
                        ui.menu_button("File", |ui| {
                            if ui.button("Open").clicked() {
                                cfg_if::cfg_if! {
                                    if #[cfg(target_family = "wasm")] {
                                        let _ = wasm::open_file_chooser(web_events.clone());
                                    } else {
                                        let result = handle_open_file_btn_click(&mut state);
                                        if let Err(err) = result {
                                            state.error = Some(err);
                                            state.show_error = true;
                                        }
                                    }
                                }

                                ui.close_menu();
                            }

                            if cfg!(not(target_family = "wasm")) {
                                if ui.button("Quit").clicked() {
                                    state.quit = true;
                                    ui.close_menu();
                                }
                            }
                        });

                        cfg_if::cfg_if! {
                            if #[cfg(target_family = "wasm")] {
                                ui.menu_button("View", |ui| {
                                    if ui.button("Fullscreen").clicked() {
                                        unsafe {
                                            wasm::js_open_fullscreen();
                                        }

                                        ui.close_menu();
                                    }
                                });
                            }
                        }
                    });
                });
            }

            if let Some(description) = &state.rom_info_description {
                egui::Window::new("ROM info")
                    .open(&mut state.show_rom_info_window)
                    .show(ctx, |ui| {
                        ui.label(description);
                    });
            }

            if let Some(event) = &state.last_gamepad_event {
                egui::Window::new("Gamepad event").show(ctx, |ui| {
                    ui.label(event);
                });
            }

            if let Some(err) = &state.error {
                let error_text = format!("{err}");
                egui::Window::new("Error")
                    .open(&mut state.show_error)
                    .show(ctx, |ui| {
                        ui.label(&error_text);
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

#[cfg(not(target_family = "wasm"))]
fn handle_open_file_btn_click(state: &mut State) -> Result<()> {
    let last_folder_path = state.last_used_dir.clone().map(PathBuf::from);
    let rom_path = open_file(&last_folder_path).expect("Could not read file");

    if let Some(rom_path) = rom_path {
        let mut current_folder = PathBuf::from(&rom_path);
        current_folder.pop(); // remove file name from path

        if let Some(last_folder_path) = last_folder_path {
            if last_folder_path != current_folder {
                if let Some(current_folder) = current_folder.to_str() {
                    save_config(ConfigFile::LastUsedDirectory, current_folder)?;
                    state.last_used_dir = Some(String::from(current_folder));
                }
            }
        } else if let Some(current_folder) = current_folder.to_str() {
            save_config(ConfigFile::LastUsedDirectory, current_folder)?;
            state.last_used_dir = Some(String::from(current_folder));
        }

        let rom = std::fs::read(&rom_path)?;
        let header = Header::read_rom_header(&rom)?;
        let rom_title = header.title.unwrap_or_else(|| String::from("<NO TITLE>"));
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
        state.show_rom_info_window = true; // Show ROM information window
        state.show_error = false;
    }

    Ok(())
}

#[cfg(not(target_family = "wasm"))]
fn open_file(last_folder: &Option<PathBuf>) -> Result<Option<String>> {
    let home_path = match UserDirs::new() {
        Some(user_dirs) => user_dirs.home_dir().to_owned(),
        None => PathBuf::from(""),
    };

    let start_path = last_folder.to_owned().unwrap_or(home_path);
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
