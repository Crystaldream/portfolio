//! WASM Components for Portfolio
//!
//! Client-side interactivity without heavy JavaScript frameworks.

use wasm_bindgen::prelude::*;
use web_sys::{window, Document, Element, HtmlElement, MouseEvent};

// =============================================================================
// Initialization
// =============================================================================

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    let window = window().expect("no global window");
    let document = window.document().expect("no document");

    init_theme(&window, &document)?;
    init_scroll_reveal(&document)?;
    init_cursor_glow(&document)?;
    init_mobile_nav(&document)?;
    init_year_display(&document)?;

    Ok(())
}

// =============================================================================
// Theme Toggle
// =============================================================================

fn init_theme(window: &web_sys::Window, document: &Document) -> Result<(), JsValue> {
    let storage = window.local_storage()?.expect("no local storage");

    let stored_theme = storage.get_item("theme")?;
    let system_prefers_light = window
        .match_media("(prefers-color-scheme: light)")?
        .map(|mql| mql.matches())
        .unwrap_or(false);

    let initial_theme = stored_theme
        .unwrap_or_else(|| if system_prefers_light { "latte" } else { "mocha" }.to_string());

    if let Some(html) = document.document_element() {
        html.set_attribute("data-theme", &initial_theme)?;
    }

    // Set up toggle button listener
    if let Some(button) = document.query_selector("[data-theme-toggle]")? {
        let document_clone = document.clone();
        let storage_clone = storage.clone();

        let callback = Closure::wrap(Box::new(move |_: web_sys::Event| {
            toggle_theme(&document_clone, &storage_clone).ok();
        }) as Box<dyn FnMut(_)>);

        button.add_event_listener_with_callback("click", callback.as_ref().unchecked_ref())?;
        callback.forget();
    }

    Ok(())
}

fn toggle_theme(document: &Document, storage: &web_sys::Storage) -> Result<(), JsValue> {
    if let Some(html) = document.document_element() {
        let current = html.get_attribute("data-theme").unwrap_or_default();
        let new_theme = if current == "mocha" { "latte" } else { "mocha" };

        html.set_attribute("data-theme", new_theme)?;
        storage.set_item("theme", new_theme)?;
    }
    Ok(())
}

// =============================================================================
// Scroll Reveal Animations
// =============================================================================

fn init_scroll_reveal(document: &Document) -> Result<(), JsValue> {
    let elements = document.query_selector_all("[data-reveal]")?;

    if elements.length() == 0 {
        return Ok(());
    }

    // Use simpler approach - just reveal on load with delays
    let window = window().expect("no window");
    
    for i in 0..elements.length() {
        if let Some(node) = elements.get(i) {
            if let Some(element) = node.dyn_ref::<Element>() {
                let el = element.clone();
                let delay = (i * 100) as i32;
                
                let callback = Closure::wrap(Box::new(move || {
                    el.class_list().add_1("revealed").ok();
                }) as Box<dyn FnMut()>);

                window.set_timeout_with_callback_and_timeout_and_arguments_0(
                    callback.as_ref().unchecked_ref(),
                    delay,
                )?;

                callback.forget();
            }
        }
    }

    Ok(())
}

// =============================================================================
// Cursor Glow Effect
// =============================================================================

fn init_cursor_glow(document: &Document) -> Result<(), JsValue> {
    let glow = document.query_selector(".cursor-glow")?;

    if let Some(glow_element) = glow {
        let glow_el: HtmlElement = glow_element.unchecked_into();

        let callback = Closure::wrap(Box::new(move |event: MouseEvent| {
            let x = event.client_x();
            let y = event.client_y();

            glow_el.style().set_property("left", &format!("{}px", x)).ok();
            glow_el.style().set_property("top", &format!("{}px", y)).ok();
        }) as Box<dyn FnMut(_)>);

        document.add_event_listener_with_callback("mousemove", callback.as_ref().unchecked_ref())?;
        callback.forget();
    }

    Ok(())
}

// =============================================================================
// Mobile Navigation
// =============================================================================

fn init_mobile_nav(document: &Document) -> Result<(), JsValue> {
    let toggle = document.query_selector("[data-nav-toggle]")?;
    let nav = document.query_selector(".nav")?;

    if let (Some(toggle_el), Some(nav_el)) = (toggle, nav) {
        let nav_clone = nav_el.clone();
        let toggle_clone = toggle_el.clone();

        let callback = Closure::wrap(Box::new(move |_: web_sys::Event| {
            let is_open = nav_clone.class_list().contains("is-open");

            if is_open {
                nav_clone.class_list().remove_1("is-open").ok();
                toggle_clone.set_attribute("aria-expanded", "false").ok();
            } else {
                nav_clone.class_list().add_1("is-open").ok();
                toggle_clone.set_attribute("aria-expanded", "true").ok();
            }
        }) as Box<dyn FnMut(_)>);

        toggle_el.add_event_listener_with_callback("click", callback.as_ref().unchecked_ref())?;
        callback.forget();
    }

    Ok(())
}

// =============================================================================
// Utility Functions
// =============================================================================

fn init_year_display(document: &Document) -> Result<(), JsValue> {
    let year_elements = document.query_selector_all("[data-year]")?;
    let year = js_sys::Date::new_0().get_full_year();

    for i in 0..year_elements.length() {
        if let Some(node) = year_elements.get(i) {
            if let Some(element) = node.dyn_ref::<Element>() {
                element.set_text_content(Some(&year.to_string()));
            }
        }
    }

    Ok(())
}

// =============================================================================
// Exported Functions
// =============================================================================

#[wasm_bindgen]
pub fn toggle_theme_manual() -> Result<(), JsValue> {
    let window = window().expect("no global window");
    let document = window.document().expect("no document");
    let storage = window.local_storage()?.expect("no local storage");

    toggle_theme(&document, &storage)
}

#[wasm_bindgen]
pub fn refresh_reveals() -> Result<(), JsValue> {
    let window = window().expect("no global window");
    let document = window.document().expect("no document");

    let elements = document.query_selector_all("[data-reveal]:not(.revealed)")?;

    for i in 0..elements.length() {
        if let Some(node) = elements.get(i) {
            if let Some(element) = node.dyn_ref::<Element>() {
                let el = element.clone();
                let callback = Closure::wrap(Box::new(move || {
                    el.class_list().add_1("revealed").ok();
                }) as Box<dyn FnMut()>);

                window.set_timeout_with_callback_and_timeout_and_arguments_0(
                    callback.as_ref().unchecked_ref(),
                    50 * (i as i32),
                )?;

                callback.forget();
            }
        }
    }

    Ok(())
}