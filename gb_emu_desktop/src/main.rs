mod config;

use config::*;
use directories::UserDirs;
use gb_emu_common::cartridge::header::Header;
use macroquad::prelude::*;
use native_dialog::FileDialog;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

const GB_SCREEN_WIDTH: f32 = 160.;
const GB_SCREEN_HEIGHT: f32 = 144.;
const MENU_BAR_HEIGHT: f32 = 23.;

fn conf() -> Conf {
    Conf {
        window_title: String::from("RustBoy"),
        fullscreen: false,
        ..Default::default()
    }
}

#[macroquad::main(conf)]
async fn main() {
    let mut quit = false;
    let mut is_fullscreen = false;

    let rom_data_description: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
    let last_folder: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(
        read_config(&ConfigFile::LastUsedDirectory).expect("Could not open config"),
    ));

    let handle_open_file_btn_click = || {
        let last_folder_path = last_folder
            .borrow()
            .clone()
            .map(|last_folder| PathBuf::from(last_folder));
        let rom_path = open_file(&last_folder_path).expect("Could not read file");

        if let Some(rom_path) = rom_path {
            let mut current_folder = PathBuf::from(&rom_path);
            current_folder.pop(); // remove file name from path
            
            if let Some(last_folder_path) = last_folder_path {
                if last_folder_path != current_folder {
                    if let Some(current_folder) = current_folder.to_str() {
                        save_config(&ConfigFile::LastUsedDirectory, current_folder).expect("Error saving config");
                        last_folder.replace(Some(String::from(current_folder)));
                    }
                }
            } else {
                if let Some(current_folder) = current_folder.to_str() {
                    save_config(&ConfigFile::LastUsedDirectory, current_folder).expect("Error saving config");
                    last_folder.replace(Some(String::from(current_folder)));
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
            let description = format!(
                "Title: {rom_title}\n\
                File name: {file_name}\n\
                Cartridge type: {cartridge_type}\n\
                File size: {file_size} bytes\
                "
            );
            rom_data_description.replace(Some(description));
        }
    };

    loop {
        clear_background(BLACK);

        if quit {
            break;
        }

        // process keys, mouse etc.

        if is_key_pressed(KeyCode::F11) {
            is_fullscreen = !is_fullscreen;
        }

        egui_macroquad::ui(|ctx| {
            if !is_fullscreen {
                egui::TopBottomPanel::top("top_panel").show(&ctx, |ui| {
                    egui::menu::bar(ui, |ui| {
                        ui.menu_button("File", |ui| {
                            if ui.button("Open").clicked() {
                                ui.close_menu();
                                handle_open_file_btn_click();
                            }

                            if ui.button("Quit").clicked() {
                                quit = true;
                                ui.close_menu();
                            }
                        });
                    });
                });
            }

            if let Some(description) = rom_data_description.borrow().as_ref() {
                egui::Window::new("ROM data").show(&ctx, |ui| {
                    ui.label(description);
                });
            }
        });

        // draw things before egui

        let menu_bar_end = MENU_BAR_HEIGHT + 1.0;
        let screen_height = if is_fullscreen {
            screen_height()
        } else {
            screen_height() - menu_bar_end
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
        let offset_y = if is_fullscreen { 0.0 } else { menu_bar_end };
        // draw gb screen
        draw_rectangle(x, y + offset_y, w, h, WHITE);

        // draw egui
        egui_macroquad::draw();

        next_frame().await;
    }
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

fn open_file(last_folder: &Option<PathBuf>) -> Result<Option<String>, native_dialog::Error> {
    let home_path = match UserDirs::new() {
        Some(user_dirs) => user_dirs.home_dir().to_owned(),
        None => PathBuf::from(""),
    };

    let start_path = last_folder.to_owned().unwrap_or(home_path.to_owned());
    let path = FileDialog::new()
        .set_location(&start_path)
        .add_filter("GB ROM", &["gb"])
        .add_filter("All files", &["*"])
        .show_open_single_file()?
        .map(|path| {
            path.into_os_string()
                .into_string()
                .expect("Erro ao converter")
        });

    Ok(path)
}
