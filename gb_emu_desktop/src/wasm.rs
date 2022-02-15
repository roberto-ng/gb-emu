use gb_emu_common::cartridge::header::Header;
use js_sys::Uint8Array;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Event, File, FileReader, HtmlInputElement};

use crate::{State, Result};

type JsResult<T> = std::result::Result<T, JsValue>;

#[cfg(target_family = "wasm")]
extern "C" {
    pub fn js_open_fullscreen();
}

pub enum FullscreenEvent {
    Enter,
    Leave,
    None,
}

#[derive(Clone)]
pub enum FileEvent {
    Open(Vec<u8>),
    None,
}

pub struct WebEvents {
    fullscreen_event: FullscreenEvent,
    file_event: FileEvent,
}

impl WebEvents {
    pub fn new() -> WebEvents {
        WebEvents {
            fullscreen_event: FullscreenEvent::None,
            file_event: FileEvent::None,
        }
    }
}

pub fn handle_web_events(
    events: &Rc<RefCell<WebEvents>>,
    state: &mut State,
) -> Result<()> {
    let mut events = events.borrow_mut();
    match events.fullscreen_event {
        FullscreenEvent::Enter => {
            state.show_menu_bar = false;
            events.fullscreen_event = FullscreenEvent::None;
        }

        FullscreenEvent::Leave => {
            state.show_menu_bar = true;
            events.fullscreen_event = FullscreenEvent::None;
        }

        _ => {}
    }

    let file_event = events.file_event.clone();
    // Clear event
    events.file_event = FileEvent::None;

    if let FileEvent::Open(rom) = &file_event {
        let header = Header::read_rom_header(&rom)?;
        let rom_title = header.title.unwrap_or_else(|| String::from("NO TITLE"));
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
        state.show_error = false;
    }

    Ok(())
}

/// Set up the event listeners used in the web version of the app
pub fn add_event_listeners(events: Rc<RefCell<WebEvents>>) {
    let window = web_sys::window().expect("No global `window` exists");
    let document = window.document().expect("Should have a document on window");

    let document_clone = document.clone();
    let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
        // Check if the document has a fullscreen element
        match document_clone.fullscreen_element() {
            Some(_) => {
                // Entering fullscreen
                events.borrow_mut().fullscreen_event = FullscreenEvent::Enter;
            }

            None => {
                // Leaving fullscreen
                events.borrow_mut().fullscreen_event = FullscreenEvent::Leave;
            }
        }
    }) as Box<dyn FnMut(_)>);

    document
        .add_event_listener_with_callback("fullscreenchange", closure.as_ref().unchecked_ref())
        .unwrap();

    // Release memory management of this closure from Rust to the JS GC.
    closure.forget();
}

fn read_file(file: File, events: Rc<RefCell<WebEvents>>) -> JsResult<()> {
    let reader = FileReader::new()?;
    let reader_clone = reader.clone();

    // Callback used to get the result
    let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
        if let Ok(file) = reader_clone.result() {
            let rom = Uint8Array::new(&file).to_vec();
            events.borrow_mut().file_event = FileEvent::Open(rom);
        }
    }) as Box<dyn FnMut(_)>);
    reader.set_onload(Some(closure.as_ref().unchecked_ref()));

    // Release memory management of this closure from Rust to the JS GC.
    closure.forget();

    reader.read_as_array_buffer(&file)?;

    Ok(())
}

/// Opens the browser's file chooser dialog and reads the file as a Vec<u8>. Sets up a FileEvent on success.
pub fn open_file_chooser(events: Rc<RefCell<WebEvents>>) -> JsResult<()> {
    let window = web_sys::window().expect("No global `window` exists");
    let document = window.document().expect("Should have a document on window");
    let input = document
        .create_element("input")?
        .dyn_into::<HtmlInputElement>()?;

    input.set_type("file");
    input.set_accept(".gb,.gbc"); // Accept ".gb" and ".gbc" files

    let input_clone = input.clone();
    let closure = Closure::wrap(
        Box::new(move |_event: web_sys::Event| match input_clone.files() {
            Some(files) if files.length() > 0 => {
                let file = files.get(0).unwrap();
                let _ = read_file(file, events.clone());
            }

            _ => {}
        }) as Box<dyn FnMut(_)>,
    );

    input.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref())?;

    // Release memory management of this closure from Rust to the JS GC.
    closure.forget();

    // Click element
    input.click();
    Ok(())
}
