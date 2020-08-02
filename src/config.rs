extern crate ini;
use ini::ini::Error;
use ini::Ini;
use serde::{Deserialize, Serialize};

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

pub fn get_property_from_timesnapper_ini(property: &str) -> String {
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

pub fn get_path_from_confy() -> String {
    let result_cfg: Result<MyConfig, confy::ConfyError> = confy::load("rust-timesnapper-checker");
    match result_cfg {
        Ok(cfg) => cfg.path,
        Err(_) => "".to_string(),
    }
}

pub fn get_blank_max_filesize_from_confy() -> String {
    let result_cfg: Result<MyConfig, confy::ConfyError> = confy::load("rust-timesnapper-checker");
    match result_cfg {
        Ok(cfg) => cfg.blank_max_filesize,
        Err(_) => "80000".to_string(),
    }
}

pub fn save_to_confy(path: &str, blank_max_filesize: &str) -> Result<(), confy::ConfyError> {
    let my_cfg = MyConfig {
        path: path.to_string(),
        blank_max_filesize: blank_max_filesize.to_string(),
    };
    confy::store("rust-timesnapper-checker", my_cfg)
}
