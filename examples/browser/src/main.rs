fn main() {
    let messages = a2ui_browser_example::surface_messages();
    let json = serde_json::to_string(&messages).expect("A2UI messages must serialize");
    println!("{json}");
}
