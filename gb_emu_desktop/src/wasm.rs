use crate::{WebEvents, FileEvent};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Event, FileReader, HtmlInputElement, File};
use js_sys::Uint8Array;
use std::cell::RefCell;
use std::rc::Rc;

fn read_file(file: File, events: Rc<RefCell<WebEvents>>) -> Result<(), JsValue> {
    let reader = FileReader::new()?;
    let reader_clone = reader.clone();
    let closure = Closure::wrap(
        Box::new(move |event: web_sys::Event| {
            if let Ok(file) = reader_clone.result() {
                let rom = Uint8Array::new(&file).to_vec();
                events.borrow_mut().file_event = FileEvent::Open(rom);
            }
        }) as Box<dyn FnMut(_)>,
    );
    reader.set_onload(Some(closure.as_ref().unchecked_ref()));
    closure.forget();
    reader.read_as_array_buffer(&file)?;

    Ok(())
}

pub fn open_file_chooser(events: Rc<RefCell<WebEvents>>) -> Result<(), JsValue> {
    let window = web_sys::window().expect("No global `window` exists");
    let document = window.document().expect("Should have a document on window");
    let input = document
        .create_element("input")?
        .dyn_into::<HtmlInputElement>()?;

    input.set_type("file");

    let input_clone = input.clone();
    let closure = Closure::wrap(
        Box::new(move |event: web_sys::Event| {
            match input_clone.files() {
                Some(files) if files.length() > 0 => {
                    let file = files.get(0).unwrap();
                    read_file(file, events.clone());
                }
    
                _ => {}
            }
        }) as Box<dyn FnMut(_)>,
    );

    input.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref())?;
    closure.forget();
    input.click();
    Ok(())
}
