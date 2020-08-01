#![windows_subsystem = "windows"]
extern crate chrono;
extern crate ini;
extern crate native_windows_derive as nwd;
extern crate native_windows_gui as nwg;
use chrono::{DateTime, Utc};
use ini::ini::Error;
use ini::Ini;
use nwd::NwgUi;
use nwg::NativeUi;
use serde::{Deserialize, Serialize};
use std::fs::{self};
use std::io;
use std::rc::Rc;

// confy
#[derive(Serialize, Deserialize)]
struct MyConfig {
    path: String,
}

// confy `MyConfig` implements `Default`
impl ::std::default::Default for MyConfig {
    fn default() -> Self {
        Self { path: "".into() }
    }
}

fn get_property_from_timesnapper_ini(property: &str) -> String {
    //first get path from this apps config using confy
    let config_path = &get_path_from_confy();

    let result_conf: Result<Ini, Error> = Ini::load_from_file_noescape(config_path);
    match result_conf {
        Ok(conf) => {
            let section: &ini::ini::Properties = conf.section(None::<String>).unwrap();
           section.get(property).unwrap().to_string()
        }
        Err(e) => match e {
            Error::Io(_io) => "This is probably an error because we do not know where your Timesnapper Settings.ini is\r\nYou should find it here:\r\nC:\\Users\\<your-user>\\AppData\\Roaming\\TimeSnapper\\Settings.ini\r\nPlease update your config.".to_string(),
            Error::Parse(pe) => pe.msg,
        },
    }
}

fn get_path_from_confy() -> String {
    let result_cfg: Result<MyConfig, confy::ConfyError> = confy::load("rust-win-gui");
    match result_cfg {
        Ok(cfg) => cfg.path,
        Err(e) => format!("{:?}", e),
    }
}
#[derive(Default, NwgUi)]
pub struct SystemTray {
    //parent window
    #[nwg_control]
    window: nwg::MessageWindow,

    #[nwg_resource(source_file: Some("./resources/cog.ico"))]
    icon: nwg::Icon,

    //system tray
    #[nwg_control(icon: Some(&data.icon), tip: Some("Timesnapper Checker(right-click)"))]
    #[nwg_events(OnContextMenu: [SystemTray::show_menu])]
    tray: nwg::TrayNotification,

    #[nwg_control(parent: window, popup: true)]
    tray_menu: nwg::Menu,

    #[nwg_control(parent: tray_menu, text: "Today's Stats")]
    #[nwg_events(OnMenuItemSelected: [SystemTray::todays_stats])]
    tray_item1: nwg::MenuItem,

    #[nwg_control(parent: tray_menu)]
    tray_separator: nwg::MenuSeparator,

    #[nwg_control(parent: tray_menu, text: "About Timesnapper Checker...")]
    #[nwg_events(OnMenuItemSelected: [SystemTray::about])]
    tray_item2: nwg::MenuItem,

    #[nwg_control(parent: tray_menu, text: "Exit...")]
    #[nwg_events(OnMenuItemSelected: [SystemTray::exit])]
    tray_item3: nwg::MenuItem,

    #[nwg_control(parent: window, interval: 3600000, stopped: false)]
    #[nwg_events(OnTimerTick: [SystemTray::todays_stats])]
    timer: nwg::Timer,
}

impl SystemTray {
    fn show_menu(&self) {
        let (x, y) = nwg::GlobalCursor::position();
        self.tray_menu.popup(x, y);
    }

    fn display_settings_window(&self) {
        let _app = SettingsPopup::build_ui(Default::default()).expect("Failed to build UI");
        nwg::dispatch_thread_events();
    }

    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }

    fn about(&self) {
        self.display_settings_window();
        //let config_path = get_property_from_timesnapper_ini("Path");
        //let content = format!("https://github.com/Swiftaff/rust_timesnapper_checker\r\n\r\nTimesnapper Checker will read the contents of the Timesnapper Snapshots folder.\r\nYou must update the path via the config file which is probably saved somewhere like here\r\n'C:/Users/<yourusername>/AppData/Roaming/rust-win-gui/config'\r\n\r\nCurrent Path is:\r\n{:?}", config_path);
        //nwg::simple_message("About Timesnapper Checker", &content);
    }

    fn todays_stats(&self) {
        let config_path = get_property_from_timesnapper_ini("Path");
        let result = &self.get_vec_direntries(config_path.clone());

        match result {
            Err(e) => {
                self.notification_error(&format!(
                    "Check your settings. Path may not be set: {:?} {:?}",
                    config_path, e
                ));
            }
            Ok(vec_direntries) => {
                let result_todays_directory = self.get_todays_directory_path(vec_direntries);
                match result_todays_directory {
                    Err(e) => self.notification_error(&e),
                    Ok(todays_directory) => {
                        let path = todays_directory.path();
                        let result_files =
                            self.get_vec_direntries(path.as_os_str().to_str().unwrap().to_string());
                        match result_files {
                            Err(e) => self.notification_error(&format!(
                                "There was an error reading from todays directory: {:?}",
                                e
                            )),
                            Ok(files) => {
                                let (total_minutes_today, total_warnings_today) =
                                    self.get_count_last_hours_files_too_small(&files);
                                let warnings_as_time = self.get_hrs_mins(total_warnings_today / 60);
                                let str_time = self.get_hrs_mins(total_minutes_today);
                                let notification_flags = if total_warnings_today > 300 {
                                    nwg::TrayNotificationFlags::WARNING_ICON
                                } else {
                                    nwg::TrayNotificationFlags::USER_ICON
                                };

                                self.tray.show(
                                    &format!(
                                        "{} ({}) Timesnapper BLANKS",
                                        warnings_as_time, total_warnings_today
                                    ),
                                    Some(&format!("Timesnapper Checker: {}", str_time)),
                                    Some(notification_flags),
                                    None,
                                );

                                //TODO
                                //allow editing: ini location
                                //allow editing on/off times for notifications
                                //link to snapshots folder
                            }
                        }
                    }
                }
            }
        }
    }

    fn notification_error(&self, notification_text: &str) {
        let notification_flags = nwg::TrayNotificationFlags::ERROR_ICON;
        let notification_title = "Timesnapper Checker";
        let notification_icon = &self.icon;
        &self.tray.show(
            notification_text,
            Some(notification_title),
            Some(notification_flags),
            Some(notification_icon),
        );
    }

    fn get_count_last_hours_files_too_small(&self, files: &Vec<std::fs::DirEntry>) -> (u32, u32) {
        let mut total_minutes_today: u32 = 0;
        let mut total_warnings_today: u32 = 0;
        let mut prev_hr_min = "".to_string();
        for entry in files {
            let metadata = entry.metadata().unwrap();

            //check if the hrs and minutes of the filename match the previously counted
            //if not, count it and update the previous for the next check
            let file = entry.file_name();
            let option_filename = file.to_str();
            match option_filename {
                Some(filename) => {
                    let split = filename.split(".");
                    let vec = split.collect::<Vec<&str>>();
                    let filename_just_hr_min = format!("{}{}", vec[0], vec[1]);

                    if filename_just_hr_min != prev_hr_min {
                        total_minutes_today += 1;
                        prev_hr_min = filename_just_hr_min;
                    }
                }
                None => {}
            }

            let filesize = metadata.len();
            if filesize < 80000 {
                total_warnings_today += 1;
            }
        }
        (total_minutes_today, total_warnings_today)
    }

    fn get_hrs_mins(&self, total_minutes: u32) -> String {
        let total_hours = total_minutes / 60;
        let remaining_minutes = total_minutes - (total_hours * 60);
        if total_hours == 0 {
            format!("{:?} mins", total_minutes)
        } else if remaining_minutes == 0 {
            if total_hours == 1 {
                "1 hr".to_string()
            } else {
                format!("{:?} hrs", total_hours)
            }
        } else {
            if total_hours == 1 {
                format!("1 hr {:?} mins", remaining_minutes)
            } else {
                format!("{:?} hrs {:?} mins", total_hours, remaining_minutes)
            }
        }
    }

    fn get_vec_direntries(&self, config_path: String) -> Result<Vec<std::fs::DirEntry>, io::Error> {
        let result =
            fs::read_dir(&config_path)?.collect::<Result<Vec<std::fs::DirEntry>, io::Error>>()?;
        Ok(result)
    }

    fn get_todays_directory_path<'a>(
        &self,
        vec_direntries: &'a Vec<std::fs::DirEntry>,
    ) -> Result<&'a std::fs::DirEntry, String> {
        let now: DateTime<Utc> = Utc::now();
        let todays_date_as_string = format!("{}", now.format("%Y-%m-%d"));
        let x = vec_direntries
            .iter()
            .filter(|d| {
                let fname = d.file_name().into_string();
                match fname {
                    Ok(d) => {
                        if d == todays_date_as_string {
                            return true;
                        } else {
                            return false;
                        }
                    }
                    _e => return false,
                }
            })
            .collect::<Vec<&std::fs::DirEntry>>();
        if x.len() == 1 {
            Ok(x[0])
        } else {
            Err(format!("There was an error finding todays directory. Check your config Settings, or the folder doesn't exist{:?}", todays_date_as_string))
        }
    }
}

#[derive(Default, NwgUi)]
pub struct SettingsPopup {
    #[nwg_control(size: (600, 115), position: (300, 300), title: "Timesnapper Checker Settings", flags: "WINDOW|VISIBLE")]
    #[nwg_events( OnWindowClose: [SettingsPopup::say_goodbye] )]
    window: nwg::Window,

    #[nwg_layout(parent: window, spacing: 1)]
    grid: nwg::GridLayout,

    #[nwg_control(text: "Location of your Timesnapper Settings.ini ?")]
    #[nwg_layout_item(layout: grid, row: 0, col: 0, colspan: 2)]
    label_ini_path_description1: nwg::Label,

    #[nwg_control(text: &get_path_from_confy())]
    #[nwg_layout_item(layout: grid, row: 1, col: 0)]
    label_ini_path: nwg::Label,

    //#[nwg_control(text: &get_path_from_confy(), focus: true)]
    //#[nwg_layout_item(layout: grid, row: 2, col: 0)]
    //name_edit: nwg::TextInput,
    #[nwg_control(text: "Change...")]
    #[nwg_layout_item(layout: grid, row: 2, col: 1, rowspan: 2)]
    #[nwg_events( OnButtonClick: [SettingsPopup::say_hello] )]
    hello_button: nwg::Button,
}

impl SettingsPopup {
    fn say_hello(&self) {
        nwg::simple_message("Hello", &format!("Hello {}", "test"));
    }

    fn say_goodbye(&self) {
        //nwg::simple_message("Goodbye", &format!("Goodbye {}", self.name_edit.text()));
        nwg::stop_thread_dispatch();
    }
}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");
    let _ui = SystemTray::build_ui(Default::default()).expect("Failed to build UI");
    nwg::dispatch_thread_events();
}
