/*!
    An application that runs in the system tray.

    Requires the following features: `cargo run --example system_tray --features "tray-notification message-window menu cursor"`
*/

extern crate chrono;
extern crate ini;
extern crate native_windows_derive as nwd;
extern crate native_windows_gui as nwg;
use chrono::{DateTime, Utc};
use ini::ini::Error;
use ini::Ini;
use nwg::NativeUi;
use serde::{Deserialize, Serialize};
use std::fs::{self};
use std::io;

#[derive(Serialize, Deserialize)]
struct MyConfig {
    path: String,
}

/// `MyConfig` implements `Default`
impl ::std::default::Default for MyConfig {
    fn default() -> Self {
        Self { path: "".into() }
    }
}

#[derive(Default)]
pub struct SystemTray {
    window: nwg::MessageWindow,
    icon: nwg::Icon,
    icon2: nwg::Icon,
    tray: nwg::TrayNotification,
    tray_menu: nwg::Menu,
    tray_item1: nwg::MenuItem,
    tray_item2: nwg::MenuItem,
    tray_item3: nwg::MenuItem,
    tray_item4: nwg::MenuItem,
}

impl SystemTray {
    fn show_menu(&self) {
        let (x, y) = nwg::GlobalCursor::position();
        self.tray_menu.popup(x, y);
    }

    fn get_config_path_from_confy(&self) -> String {
        let result_cfg: Result<MyConfig, confy::ConfyError> = confy::load("rust-win-gui");
        match result_cfg {
            Ok(cfg) => cfg.path,
            Err(e) => format!("{:?}", e),
        }
    }

    fn get_property_from_timesnapper_ini(&self, property: &str) -> String {
        //first get path from this apps config using confy
        let config_path = &self.get_config_path_from_confy();

        let result_conf: Result<Ini, Error> = Ini::load_from_file_noescape(config_path);
        match result_conf {
            Ok(conf) => {
                let section: &ini::ini::Properties = conf.section(None::<String>).unwrap();
               section.get(property).unwrap().to_string()
            }
            Err(e) => match e {
                Error::Io(io) => "This is probably an error because we do not know where your Timesnapper Settings.ini is\r\nYou should find it here:\r\nC:\\Users\\<your-user>\\AppData\\Roaming\\TimeSnapper\\Settings.ini\r\nPlease update your config.".to_string(),
                Error::Parse(pe) => pe.msg,
            },
        }
    }

    fn about(&self) {
        let config_path = &self.get_property_from_timesnapper_ini("Path");
        let content = format!("This tool will read the contents of a folder at the specified path.\r\nYou can update the path via the config file.\r\nIt's probably saved here\r\n'C:/Users/<yourusername>/AppData/Roaming/rust-win-gui/config'\r\n\r\nCurrent Path is:\r\n{:?}", config_path);
        nwg::simple_message("Settings", &content);
    }

    fn todays_stats(&self) {
        //nwg::simple_message("Hello", &self.get_todays_stats());
        //nwg::simple_message("Hello", &self.get_todays_snapshots_folder());
        self.get_todays_snapshots_folder()
    }

    fn notification(&self) {
        let flags = nwg::TrayNotificationFlags::USER_ICON | nwg::TrayNotificationFlags::LARGE_ICON;
        self.tray.show(
            "Timesnapper checker",
            Some("testy"),
            Some(flags),
            Some(&self.icon2),
        );
    }

    fn get_todays_snapshots_folder(&self) {
        let config_path = self.get_property_from_timesnapper_ini("Path");
        let interval_string = self.get_property_from_timesnapper_ini("Interval");
        let result = &self.get_vec_direntries(config_path.clone());
        match result {
            Ok(vec_direntries) => {
                let result_todays_directory_path = &self.get_todays_directory_path(vec_direntries);
                let files = &self.get_todays_directory_files(result_todays_directory_path);

                //TODO
                //get x screensperminute from timesnapper ini
                //count total, total x screensperminute for notification
                let count = files.len() as u32;
                let result_interval: Result<u32, std::num::ParseIntError> = interval_string.parse();
                let interval = match result_interval {
                    Ok(i) => i,
                    Err(e) => 999,
                };
                let total = count * interval;
                let total_hours = (total / 60);
                let remaining_minutes = total - (total_hours * 60);
                let str_time = if total_hours == 0 {
                    format!("{:?}mins", total)
                } else if remaining_minutes == 0 {
                    if total_hours == 1 {
                        format!("{:?}hr", total_hours)
                    } else {
                        format!("{:?}hrs", total_hours)
                    }
                } else {
                    if total_hours == 1 {
                        format!("{:?}hr {:?}mins", total_hours, remaining_minutes)
                    } else {
                        format!("{:?}hrs {:?}mins", total_hours, remaining_minutes)
                    }
                };
                //sort files by date added, filter by last hour, count number below X in size in last hour - to notify about black ones

                //elsewhere - run this every x minutes

                //allow editing: ini location

                let flags =
                    nwg::TrayNotificationFlags::USER_ICON | nwg::TrayNotificationFlags::LARGE_ICON;
                self.tray.show(
                    "Timesnapper checker",
                    Some(&format!("{} so far today", str_time)),
                    Some(flags),
                    Some(&self.icon2),
                );

                println!("count:{}", count);
                println!("interval:{:?}", interval);
                println!("total:{:?}", total);
                println!("str_time:{:?}", str_time);
                println!("files: {:?}", files);
            }
            Err(e) => println!(
                "error from get_vec_direntries -- {} -- {:?}",
                config_path, e
            ),
        }
        //format!("get all folders from {}", &config_path)
    }

    fn get_vec_direntries(&self, config_path: String) -> Result<Vec<std::fs::DirEntry>, io::Error> {
        let result = fs::read_dir(&config_path)?
            //.filter_map(|r| r.ok())
            //.map(|res| res.map(|e| e.metadata()))
            .collect::<Result<Vec<std::fs::DirEntry>, io::Error>>()?;
        Ok(result)
    }

    /*fn get_vec_metadata(&self) -> Result<Result<Vec<std::fs::Metadata>, io::Error>, io::Error> {
        let config_path = &self.get_property_from_timesnapper_ini();
        let result = fs::read_dir(config_path)?
            .map(|res| res.map(|e| e.metadata()))
            .collect::<Result<Result<Vec<std::fs::Metadata>, io::Error>, io::Error>>()?;
        Ok(result)
    }*/

    fn get_todays_directory_path<'a>(
        &self,
        vec_direntries: &'a Vec<std::fs::DirEntry>,
    ) -> Result<&'a std::fs::DirEntry, String> {
        let now: DateTime<Utc> = Utc::now();
        let todays_date_as_string = format!("{}", now.format("%Y-%m-%d"));
        //println!("{:?}", todays_date_as_string);
        let x = vec_direntries
            .iter()
            .filter(|d| {
                let mut fname = d.file_name().into_string();
                match fname {
                    Ok(d) => {
                        if d == todays_date_as_string {
                            return true;
                        } else {
                            return false;
                        }
                    }
                    error => return false,
                }
            })
            .collect::<Vec<&std::fs::DirEntry>>();
        if x.len() == 1 {
            Ok(x[0])
        } else {
            Err(format!("{:?}", x))
        }
    }

    fn get_todays_directory_files<'a>(
        &self,
        result_todays_directory: &Result<&'a std::fs::DirEntry, String>,
    ) -> Vec<std::fs::DirEntry> {
        match result_todays_directory {
            Ok(direntry) => {
                let path = direntry.path(); //format!("{}", direntry.path());

                let result =
                    self.get_vec_direntries(path.as_os_str().to_str().unwrap().to_string());
                println!("path {:?}", path);
                match result {
                    Ok(vec_dir_entries) => vec_dir_entries,
                    Err(e) => {
                        println!("Error {:?}", e);
                        Vec::new()
                    }
                }
            }
            Err(e) => {
                println!("Error {:?}", e);
                Vec::new()
            }
        }
    }

    /*fn get_todays_stats(&self) -> String {
        let mut content: String = "Files: ".to_string();
        let result = &self.get_vec_metadata();
        match result {
            Ok(items_result) => {
                match items_result {
                    Ok(items) => {
                        for item in items {
                            //let path = pathBuf.into_os_string().into_string().unwrap();
                            content = format!("{}{:?}", &content, &item);
                        }
                    }
                    Err(_) => {}
                }
            }
            Err(_) => {}
        }

        content
    }*/

    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }
}

//
// ALL of this stuff is handled by native-windows-derive
//
mod system_tray_ui {
    use super::*;
    use native_windows_gui as nwg;
    use std::cell::RefCell;
    use std::ops::Deref;
    use std::rc::Rc;

    pub struct SystemTrayUi {
        inner: Rc<SystemTray>,
        default_handler: RefCell<Vec<nwg::EventHandler>>,
    }

    impl nwg::NativeUi<SystemTrayUi> for SystemTray {
        fn build_ui(mut data: SystemTray) -> Result<SystemTrayUi, nwg::NwgError> {
            use nwg::Event as E;

            // Resources
            nwg::Icon::builder()
                .source_file(Some("./resources/cog.ico"))
                .build(&mut data.icon)?;

            nwg::Icon::builder()
                .source_file(Some("./resources/love.ico"))
                .build(&mut data.icon2)?;

            // Controls
            nwg::MessageWindow::builder().build(&mut data.window)?;

            nwg::TrayNotification::builder()
                .parent(&data.window)
                .icon(Some(&data.icon))
                .tip(Some("Right-click for menu"))
                .build(&mut data.tray)?;

            nwg::Menu::builder()
                .popup(true)
                .parent(&data.window)
                .build(&mut data.tray_menu)?;

            nwg::MenuItem::builder()
                .text("Today's stats")
                .parent(&data.tray_menu)
                .build(&mut data.tray_item1)?;

            nwg::MenuItem::builder()
                .text("Settings")
                .parent(&data.tray_menu)
                .build(&mut data.tray_item2)?;

            nwg::MenuItem::builder()
                .text("Send Notification")
                .parent(&data.tray_menu)
                .build(&mut data.tray_item3)?;

            nwg::MenuItem::builder()
                .text("Exit")
                .parent(&data.tray_menu)
                .build(&mut data.tray_item4)?;

            // Wrap-up
            let ui = SystemTrayUi {
                inner: Rc::new(data),
                default_handler: Default::default(),
            };

            // Events
            let evt_ui = Rc::downgrade(&ui.inner);
            let handle_events = move |evt, _evt_data, handle| {
                if let Some(evt_ui) = evt_ui.upgrade() {
                    match evt {
                        E::OnContextMenu => {
                            if &handle == &evt_ui.tray {
                                SystemTray::show_menu(&evt_ui);
                            }
                        }
                        E::OnMenuItemSelected => {
                            if &handle == &evt_ui.tray_item1 {
                                SystemTray::todays_stats(&evt_ui);
                            } else if &handle == &evt_ui.tray_item2 {
                                SystemTray::about(&evt_ui);
                            } else if &handle == &evt_ui.tray_item3 {
                                SystemTray::notification(&evt_ui);
                            } else if &handle == &evt_ui.tray_item4 {
                                SystemTray::exit(&evt_ui);
                            }
                        }
                        _ => {}
                    }
                }
            };

            ui.default_handler
                .borrow_mut()
                .push(nwg::full_bind_event_handler(
                    &ui.window.handle,
                    handle_events,
                ));

            return Ok(ui);
        }
    }

    impl Drop for SystemTrayUi {
        /// To make sure that everything is freed without issues, the default handler must be unbound.
        fn drop(&mut self) {
            let mut handlers = self.default_handler.borrow_mut();
            for handler in handlers.drain(0..) {
                nwg::unbind_event_handler(&handler);
            }
        }
    }

    impl Deref for SystemTrayUi {
        type Target = SystemTray;

        fn deref(&self) -> &SystemTray {
            &self.inner
        }
    }
}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    let _ui = SystemTray::build_ui(Default::default()).expect("Failed to build UI");
    nwg::dispatch_thread_events();
}
