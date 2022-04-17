use crate::client;
use crate::constant;
use sciter;
pub fn run() {
    use sciter::SCRIPT_RUNTIME_FEATURES::*;
    sciter::set_options(sciter::RuntimeOptions::ScriptFeatures(
        ALLOW_FILE_IO as u8 | ALLOW_SOCKET_IO as u8 | ALLOW_EVAL as u8 | ALLOW_SYSINFO as u8,
    ))
    .unwrap();
    let mut frame = sciter::WindowBuilder::main_window()
        .glassy()
        .resizeable()
        .with_title()
        .create();
    use std::env;
    use std::fs;
    let event_hanlder = client::event_handler().unwrap();
    let host_handler=client::host_handler().unwrap();
    let appbin = fs::read(env::current_dir().unwrap().join("app.bin")).unwrap();
    let mut app_title=String::from(constant::APP);
    app_title.push_str(constant::VERSION);
    frame.archive_handler(&appbin).unwrap();
    frame.event_handler(event_hanlder);
    frame.sciter_handler(host_handler);
    frame.set_title(app_title.as_str());
    frame.load_file("this://app/index.htm");
    frame.run_app()
}
