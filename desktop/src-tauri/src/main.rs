// Windows: no console window for a GUI release build.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    personal_wiki_lib::run()
}
