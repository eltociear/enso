//! A debug scene which shows the Dynamic drop-down component.

#![recursion_limit = "1024"]
// === Features ===
// #![feature(associated_type_defaults)]
// #![feature(drain_filter)]
// #![feature(fn_traits)]
// #![feature(trait_alias)]
// #![feature(type_alias_impl_trait)]
// #![feature(unboxed_closures)]
// === Standard Linter Configuration ===
#![deny(non_ascii_idents)]
#![warn(unsafe_code)]
#![allow(clippy::bool_to_int_with_if)]
#![allow(clippy::let_and_return)]


use ensogl_core::prelude::*;
use wasm_bindgen::prelude::*;

use ensogl_core::application::Application;
use ensogl_core::display::navigation::navigator::Navigator;
use ensogl_core::display::object::ObjectOps;
use ensogl_drop_down::Dropdown;
use ensogl_drop_down::DropdownValue;
use ensogl_hardcoded_theme as theme;
use ensogl_text as text;
use ensogl_text_msdf::run_once_initialized;



// ===================
// === Entry Point ===
// ===================

/// An entry point.
#[entry_point]
#[allow(dead_code)]
pub fn main() {
    run_once_initialized(|| {
        let app = Application::new("root");
        init(&app);
        mem::forget(app);
    });
}



// ========================
// === Init Application ===
// ========================

fn init(app: &Application) {
    theme::builtin::dark::register(app);
    theme::builtin::light::register(app);
    theme::builtin::light::enable(app);

    let world = &app.display;
    let scene = &world.default_scene;
    let navigator = Navigator::new(scene, &scene.camera());
    navigator.disable_wheel_panning();

    app.views.register::<Dropdown<EntryData>>();
    let main_dropdown = setup_dropdown(app);
    world.add_child(&main_dropdown);

    let output_text = setup_output_text(app, &main_dropdown);


    let multi_config_dropdown = app.new_view::<Dropdown<SelectConfigEntry<_>>>();
    world.add_child(&multi_config_dropdown);
    multi_config_dropdown.set_xy(Vector2(-200.0, 0.0));
    multi_config_dropdown.set_max_size(Vector2(150.0, 200.0));
    multi_config_dropdown.set_all_entries(vec![
        SelectConfigEntry("Single select".into(), false),
        SelectConfigEntry("Multi select".into(), true),
    ]);
    multi_config_dropdown.set_open(true);


    let open_dropdown = app.new_view::<Dropdown<SelectConfigEntry<_>>>();
    world.add_child(&open_dropdown);
    open_dropdown.set_xy(Vector2(-200.0, -100.0));
    open_dropdown.set_max_size(Vector2(150.0, 200.0));
    open_dropdown.set_all_entries(vec![
        SelectConfigEntry("Opened".into(), true),
        SelectConfigEntry("Closed".into(), false),
    ]);
    open_dropdown.set_open(true);

    let secondary_dropdown = app.new_view::<Dropdown<EntryData>>();
    world.add_child(&secondary_dropdown);
    secondary_dropdown.set_xy(Vector2(100.0, 0.0));
    secondary_dropdown.set_open(true);


    let static_entries =
        vec!["Hello", "World", "This", "Is", "A", "Test", "Dropdown", "With", "Static", "Strings"];
    let dropdown_static1 = app.new_view::<Dropdown<&str>>();
    dropdown_static1.set_xy(Vector2(300.0, 0.0));
    dropdown_static1.set_all_entries(static_entries.clone());
    dropdown_static1.set_open(true);

    let dropdown_static2 = app.new_view::<Dropdown<&str>>();
    dropdown_static2.set_xy(Vector2(400.0, 0.0));
    dropdown_static2.set_all_entries(static_entries.clone());
    dropdown_static2.set_open(true);
    world.add_child(&dropdown_static1);
    world.add_child(&dropdown_static2);


    frp::new_network! { network
        main_dropdown.set_multiselect <+ multi_config_dropdown.single_selected_entry.unwrap().map(SelectConfigEntry::item);
        main_dropdown.set_open <+ open_dropdown.single_selected_entry.unwrap().map(SelectConfigEntry::item);

        dropdown_static1.set_selected_entries <+ dropdown_static2.selected_entries;
        dropdown_static2.set_selected_entries <+ dropdown_static1.selected_entries;

        secondary_dropdown.set_all_entries <+ main_dropdown.selected_entries.map(|entries| {
            entries.iter().cloned().collect()
        });
    }



    std::mem::forget((
        main_dropdown,
        multi_config_dropdown,
        open_dropdown,
        secondary_dropdown,
        dropdown_static1,
        dropdown_static2,
        navigator,
        output_text,
        network,
    ));
}


fn entry_for_row(row: usize) -> EntryData {
    EntryData(row as i32 * 3 / 2 - 70)
}

fn setup_dropdown(app: &Application) -> Dropdown<EntryData> {
    let dropdown = app.new_view::<Dropdown<EntryData>>();
    dropdown.set_xy(Vector2(0.0, 0.0));
    dropdown.set_number_of_entries(2000);

    frp::new_network! { network
        entries <- dropdown.entries_in_range_needed.map(|range| (range.clone(), range.clone().map(entry_for_row).collect()));
        dropdown.provide_entries_at_range <+ entries;
    }

    std::mem::forget(network);
    dropdown
}

fn setup_output_text(app: &Application, dropdown: &Dropdown<EntryData>) -> text::Text {
    let text = app.new_view::<text::Text>();
    app.display.add_child(&text);
    text.set_xy(Vector2(-200.0, -200.0));

    frp::new_network! { network
        text.set_content <+ dropdown.selected_entries.map(|entries| {
            use std::fmt::Write;
            let mut buf = String::new();
            for EntryData(val) in entries {
                write!(&mut buf, "{val} ").unwrap();
            }
            ImString::from(buf)
        });
    }

    std::mem::forget(network);
    text
}



// ========================
// === Demo entry types ===
// ========================

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct EntryData(i32);

impl DropdownValue for EntryData {
    fn label(&self) -> ImString {
        format!("{}", self.0).into()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct SelectConfigEntry<T>(ImString, T);

impl<T> DropdownValue for SelectConfigEntry<T>
where T: Debug + Clone + PartialEq + Eq + Hash + 'static
{
    fn label(&self) -> ImString {
        self.0.clone()
    }
}
impl<T> SelectConfigEntry<T>
where T: Clone
{
    fn item(&self) -> T {
        self.1.clone()
    }
}
