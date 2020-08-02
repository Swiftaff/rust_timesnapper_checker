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

// confy
#[derive(Serialize, Deserialize)]
struct MyConfig {
    path: String,
    blank_max_filesize: String,
}

// confy `MyConfig` implements `Default`
impl ::std::default::Default for MyConfig {
    fn default() -> Self {
        Self {
            path: "".into(),
            blank_max_filesize: "80000".into(),
        }
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
    let result_cfg: Result<MyConfig, confy::ConfyError> = confy::load("rust-timesnapper-checker");
    match result_cfg {
        Ok(cfg) => cfg.path,
        Err(_) => "".to_string(),
    }
}

fn get_blank_max_filesize_from_confy() -> String {
    let result_cfg: Result<MyConfig, confy::ConfyError> = confy::load("rust-timesnapper-checker");
    match result_cfg {
        Ok(cfg) => cfg.blank_max_filesize,
        Err(_) => "80000".to_string(),
    }
}

fn save_to_confy(path: &str, blank_max_filesize: &str) -> Result<(), confy::ConfyError> {
    let my_cfg = MyConfig {
        path: path.to_string(),
        blank_max_filesize: blank_max_filesize.to_string(),
    };
    confy::store("rust-timesnapper-checker", my_cfg)
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

    #[nwg_control(parent: tray_menu, text: "Timesnapper Checker Settings...")]
    #[nwg_events(OnMenuItemSelected: [SystemTray::display_settings_window])]
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

        let string_blank_max_filesize = get_blank_max_filesize_from_confy();
        let result_blank_max_filesize: Result<u64, std::num::ParseIntError> =
            string_blank_max_filesize.parse();
        let blank_max_filesize = match result_blank_max_filesize {
            Ok(val) => val,
            Err(_) => 80000 as u64,
        };

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
            if filesize < blank_max_filesize {
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
            Err(format!("There was an error finding todays directory. Check your config Settings, or the folder doesn't exist: {:?}", todays_date_as_string))
        }
    }
}

#[derive(Default, NwgUi)]
pub struct SettingsPopup {
    //height roughly 30 * rows?
    #[nwg_control(size: (800, 300), position: (600, 600), title: "Timesnapper Checker Settings", flags: "WINDOW|VISIBLE")]
    #[nwg_events( OnWindowClose: [SettingsPopup::close] )]
    window: nwg::Window,

    #[nwg_layout(parent: window, spacing: 1)]
    grid: nwg::GridLayout,

    #[nwg_control(text: "", flags:"NONE")]
    #[nwg_layout_item(layout: grid, row: 0, col: 0)]
    state_is_dirty: nwg::Label,

    //title
    #[nwg_control(text: "Timesnapper Checker needs to know where the Timesnapper 'Settings.ini' file is located.")]
    #[nwg_layout_item(layout: grid, row: 0, col: 0, col_span: 5)]
    intro2: nwg::Label,

    //Settings ini path field
    #[nwg_control(text: "It is usually here: C:\\Users\\%USERPROFILE%\\AppData\\Roaming\\TimeSnapper\\Settings.ini")]
    #[nwg_layout_item(layout: grid, row: 2, col: 0, col_span: 5)]
    intro3: nwg::Label,

    #[nwg_control(text: &get_path_from_confy(), flags: "VISIBLE|DISABLED")]
    #[nwg_layout_item(layout: grid, row: 3, col: 0, col_span: 5)]
    ini_path: nwg::TextInput,

    #[nwg_resource(title:"Timesnapper Checker - Select Settings.ini",action: nwg::FileDialogAction::Open, filters: "Ini(*.ini)")]
    file_dialog: nwg::FileDialog,

    #[nwg_control(text: "Select...",focus: true)]
    #[nwg_layout_item(layout: grid, row: 3, col: 5)]
    #[nwg_events( OnButtonClick: [SettingsPopup::ini_path_selector] )]
    button_change: nwg::Button,

    //filesize field
    #[nwg_control(text: "Timesnapper checker identifies blank screengrabs by their filesize.")]
    #[nwg_layout_item(layout: grid, row: 5, col: 0, col_span: 5)]
    label_blank_max_filesize1: nwg::Label,

    #[nwg_control(text: "This will change depending on screen resolution, file format.")]
    #[nwg_layout_item(layout: grid, row: 6, col: 0, col_span: 5)]
    label_blank_max_filesize2: nwg::Label,

    #[nwg_control(text: "Enter a value (in bytes) slightly bigger than a typical blank (Default 80000 = 80Kb)")]
    #[nwg_layout_item(layout: grid, row: 7, col: 0, col_span: 5)]
    label_blank_max_filesize3: nwg::Label,

    #[nwg_control(text: &get_blank_max_filesize_from_confy(), flags: "VISIBLE")]
    #[nwg_layout_item(layout: grid, row: 8, col: 0, col_span: 6)]
    #[nwg_events( OnTextInput: [SettingsPopup::filesize_dirty] )]
    blank_max_filesize: nwg::TextInput,

    //save and cancel
    #[nwg_control(text: "Save Changes", enabled: false)]
    #[nwg_layout_item(layout: grid, row: 10, col: 0)]
    #[nwg_events( OnButtonClick: [SettingsPopup::save] )]
    button_save: nwg::Button,

    #[nwg_control(text: "Cancel")]
    #[nwg_layout_item(layout: grid, row: 10, col: 1)]
    #[nwg_events( OnButtonClick: [SettingsPopup::cancel] )]
    button_cancel: nwg::Button,
}

impl SettingsPopup {
    fn filesize_dirty(&self) {
        self.button_save.set_enabled(true);
        self.state_is_dirty.set_text("filesize changed");
    }

    fn cancel(&self) {
        nwg::stop_thread_dispatch();
    }

    fn close(&self) {
        if &self.state_is_dirty.text().len() > &(0 as usize) {
            &self.state_is_dirty.set_text("");
            let p = nwg::MessageParams {
                title: "Do you want to save the changes you made?",
                content: "Your changes will be lost if you don't save them.",
                buttons: nwg::MessageButtons::YesNoCancel,
                icons: nwg::MessageIcons::Warning,
            };
            let result = nwg::message(&p);
            //nwg::simple_message("About Timesnapper Checker", &format!("{:?}", &result));
            if &result == &nwg::MessageChoice::Yes {
                &self.save();
            } else if &result == &nwg::MessageChoice::No {
                nwg::stop_thread_dispatch();
            }
        } else {
            nwg::stop_thread_dispatch();
        }
    }

    fn save(&self) {
        let result = save_to_confy(&self.ini_path.text(), &self.blank_max_filesize.text());
        match result {
            Ok(_) => {
                nwg::simple_message("Timesnapper Checker - Saving settings", "Saved");
            }
            Err(e) => {
                nwg::error_message(
                    "Timesnapper Checker - Saving settings",
                    &format!("NOT saved - error: {:?}", e),
                );
            }
        }
        nwg::stop_thread_dispatch();
    }

    fn ini_path_selector(&self) {
        if self.file_dialog.run(Some(&self.window)) {
            if let Ok(file) = self.file_dialog.get_selected_item() {
                self.ini_path.set_text(&file);
                self.button_save.set_enabled(true);
                self.state_is_dirty.set_text("path changed");
            }
        }
    }
}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");
    let _ui = SystemTray::build_ui(Default::default()).expect("Failed to build UI");
    nwg::dispatch_thread_events();
}
