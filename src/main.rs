//#![windows_subsystem = "windows"]
/*!
    An application that runs in the system tray.

    See also note for distributing... https://gabdube.github.io/native-windows-gui/native-windows-docs/distribute.html
*/

extern crate chrono;
extern crate ini;
//extern crate job_scheduler;
extern crate native_windows_derive as nwd;
extern crate native_windows_gui as nwg;
use chrono::{DateTime, Utc};
use ini::ini::Error;
use ini::Ini;
//use job_scheduler::{Job, JobScheduler};
use nwd::NwgUi;
use nwg::NativeUi;
use serde::{Deserialize, Serialize};
use std::fs::{self};
use std::io;
use std::time::Duration;
use std::{cell::RefCell, thread};

//confy
#[derive(Serialize, Deserialize)]
struct MyConfig {
    path: String,
}

/// confy `MyConfig` implements `Default`
impl ::std::default::Default for MyConfig {
    fn default() -> Self {
        Self { path: "".into() }
    }
}

/// The dialog UI
#[derive(Default, NwgUi)]
pub struct ThreadingDialog {
    data: RefCell<String>,

    #[nwg_control(size: (300, 115), position: (650, 300), title: "A dialog", flags: "WINDOW|VISIBLE")]
    #[nwg_events( OnWindowClose: [ThreadingDialog::close] )]
    window: nwg::Window,

    #[nwg_control(text: "YES", position: (10, 10), size: (130, 95))]
    #[nwg_events( OnButtonClick: [ThreadingDialog::choose(SELF, CTRL)] )]
    choice_yes: nwg::Button,

    #[nwg_control(text: "NO", position: (160, 10), size: (130, 95), focus: true)]
    #[nwg_events( OnButtonClick: [ThreadingDialog::choose(SELF, CTRL)] )]
    choice_no: nwg::Button,
}

impl ThreadingDialog {
    fn close(&self) {
        nwg::stop_thread_dispatch();
    }

    fn choose(&self, btn: &nwg::Button) {
        let mut data = self.data.borrow_mut();
        if btn == &self.choice_no {
            *data = "No!".to_string();
        } else if btn == &self.choice_yes {
            *data = "Yes!".to_string();
        }

        self.window.close();
    }
}

#[derive(Default, NwgUi)]
pub struct SystemTray {
    dialog_data: RefCell<Option<thread::JoinHandle<String>>>,

    #[nwg_control]
    window: nwg::MessageWindow,

    #[nwg_control]
    #[nwg_events( OnNotice: [SystemTray::read_dialog_output] )]
    dialog_notice: nwg::Notice,

    #[nwg_resource(source_file: Some("./resources/cog.ico"))]
    icon: nwg::Icon,

    #[nwg_resource(source_file: Some("./resources/warning.ico"))]
    icon_warning: nwg::Icon,

    #[nwg_control(icon: Some(&data.icon), tip: Some("Right-click for menu"))]
    #[nwg_events(OnContextMenu: [SystemTray::show_menu])]
    tray: nwg::TrayNotification,

    #[nwg_control(parent: window, popup: true)]
    tray_menu: nwg::Menu,

    #[nwg_control(parent: tray_menu, text: "Today's Stats")]
    #[nwg_events(OnMenuItemSelected: [SystemTray::todays_stats])]
    tray_item1: nwg::MenuItem,

    #[nwg_control(parent: tray_menu, text: "Open Dialog")]
    #[nwg_events(OnMenuItemSelected: [SystemTray::open_dialog])]
    tray_item2: nwg::MenuItem,

    #[nwg_control(parent: tray_menu, text: "Exit")]
    #[nwg_events(OnMenuItemSelected: [SystemTray::exit])]
    tray_item3: nwg::MenuItem,
}

impl SystemTray {
    fn open_dialog(&self) {
        let mut data = self.dialog_data.borrow_mut();
        if data.is_some() {
            nwg::error_message("Error", "The dialog is already running!");
            return;
        }

        let notice = self.dialog_notice.sender();

        *data = Some(thread::spawn(move || {
            let app = ThreadingDialog::build_ui(Default::default()).expect("Failed to build UI");
            nwg::dispatch_thread_events();

            notice.notice();
            {
                let data = app.data.borrow();
                data.clone()
            }
        }))
    }

    fn read_dialog_output(&self) {
        let mut data = self.dialog_data.borrow_mut();
        match data.take() {
            Some(handle) => {
                println!("do something with this: {:?}", &handle.join().unwrap());
                //self.name_edit.set_text(&handle.join().unwrap());
                //self.button.set_focus();
            }
            None => {}
        }
    }

    fn show_menu(&self) {
        let (x, y) = nwg::GlobalCursor::position();
        self.tray_menu.popup(x, y);
    }

    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }

    fn about(&self) {
        let config_path = &self.get_property_from_timesnapper_ini("Path");
        let content = format!("This tool will read the contents of a folder at the specified path.\r\nYou can update the path via the config file.\r\nIt's probably saved here\r\n'C:/Users/<yourusername>/AppData/Roaming/rust-win-gui/config'\r\n\r\nCurrent Path is:\r\n{:?}\r\n\r\n<div>Warning Icon made by https://www.flaticon.com/authors/vectors-market", config_path);
        nwg::simple_message("Settings", &content);
    }

    fn todays_stats(&self) {
        let config_path = self.get_property_from_timesnapper_ini("Path");

        let result = &self.get_vec_direntries(config_path.clone());
        match result {
            Ok(vec_direntries) => {
                let result_todays_directory_path = &self.get_todays_directory_path(vec_direntries);
                let files = &self.get_todays_directory_files(result_todays_directory_path);
                let str_time = &self.get_time_spent_string(files);
                let (last_hour_count, last_hour_size_warning) =
                    self.get_count_last_hours_files_too_small(files);
                let notification_text =
                    &self.get_last_hour_text(last_hour_count, last_hour_size_warning);
                let icon = if last_hour_size_warning > 0 {
                    &self.icon_warning
                } else {
                    &self.icon
                };

                //TODO
                //elsewhere - run this every x minutes
                //allow editing: ini location
                //allow editing on/off times for notifications
                //link to snapshots folder
                //fix div in about, add other icon ref

                let flags =
                    nwg::TrayNotificationFlags::USER_ICON | nwg::TrayNotificationFlags::LARGE_ICON;
                self.tray.show(
                    notification_text,
                    Some(&format!("{} so far today", str_time)),
                    Some(flags),
                    Some(icon),
                );
            }
            Err(e) => println!(
                "error from get_vec_direntries -- {} -- {:?}",
                config_path, e
            ),
        }
        //format!("get all folders from {}", &config_path)
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
                Error::Io(_io) => "This is probably an error because we do not know where your Timesnapper Settings.ini is\r\nYou should find it here:\r\nC:\\Users\\<your-user>\\AppData\\Roaming\\TimeSnapper\\Settings.ini\r\nPlease update your config.".to_string(),
                Error::Parse(pe) => pe.msg,
            },
        }
    }

    fn get_config_path_from_confy(&self) -> String {
        let result_cfg: Result<MyConfig, confy::ConfyError> = confy::load("rust-win-gui");
        match result_cfg {
            Ok(cfg) => cfg.path,
            Err(e) => format!("{:?}", e),
        }
    }

    fn get_last_hour_text(&self, last_hour_count: u32, last_hour_size_warning: u32) -> String {
        let last_hour_spent = self.get_hrs_mins(last_hour_count);
        if last_hour_size_warning == 1 {
            format!(
                "*1* Timesnapper BLANK during {} of the last hour",
                last_hour_spent
            )
        } else if last_hour_size_warning > 1 {
            format!(
                "*{:?}* Timesnapper BLANKS during {} of the last hour",
                last_hour_size_warning, last_hour_spent
            )
        } else {
            format!("{} snapped in the last hour", last_hour_spent)
        }
    }

    fn get_count_last_hours_files_too_small(&self, files: &Vec<std::fs::DirEntry>) -> (u32, u32) {
        let mut last_hour_count: u32 = 0;
        let mut last_hour_size_warning: u32 = 0;
        for entry in files {
            let metadata = entry.metadata().unwrap();
            let last_modified = metadata.modified().unwrap().elapsed().unwrap().as_secs();
            let filesize = metadata.len();
            println!("{:?}", filesize);

            if last_modified < 24 * 3600 {
                last_hour_count += 1;
                if filesize < 80000 {
                    last_hour_size_warning += 1;
                }
            }
        }
        (last_hour_count, last_hour_size_warning)
    }

    fn get_time_spent_string(&self, files: &Vec<std::fs::DirEntry>) -> String {
        let count = files.len() as u32;
        self.get_hrs_mins(count)
    }

    fn get_hrs_mins(&self, count: u32) -> String {
        let interval_string = self.get_property_from_timesnapper_ini("Interval");
        let result_interval: Result<u32, std::num::ParseIntError> = interval_string.parse();
        let interval = match result_interval {
            Ok(i) => i,
            Err(_e) => 999,
        };
        let total_seconds = count * interval;
        let total_minutes = count * interval / 60;
        let total_hours = total_seconds / 3600;
        let remaining_minutes = total_minutes - (total_hours * 60);
        if total_hours == 0 {
            if total_minutes == 0 {
                format!("{:?} seconds", total_seconds)
            } else {
                format!("{:?} mins", total_minutes)
            }
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
        let result = fs::read_dir(&config_path)?
            //.filter_map(|r| r.ok())
            //.map(|res| res.map(|e| e.metadata()))
            .collect::<Result<Vec<std::fs::DirEntry>, io::Error>>()?;
        Ok(result)
    }

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
}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    let _ui = SystemTray::build_ui(Default::default()).expect("Failed to build UI");
    nwg::dispatch_thread_events();

    /*let mut sched = JobScheduler::new();
    sched.add(Job::new("1/10 * * * * *".parse().unwrap(), || {
        println!("I get executed every 10 seconds!");
    }));

    loop {
        sched.tick();
        std::thread::sleep(Duration::from_millis(500));
    }*/
}
