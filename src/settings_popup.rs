extern crate native_windows_derive as nwd;
extern crate native_windows_gui as nwg;
use crate::config::*;
use nwd::NwgUi;

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
