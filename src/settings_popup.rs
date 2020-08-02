extern crate native_windows_derive as nwd;
extern crate native_windows_gui as nwg;
use crate::config::*;
use nwd::NwgUi;

#[derive(Default, NwgUi)]
pub struct SettingsPopup {
    //height roughly 30 * rows?
    #[nwg_control(size: (800, 250), position: (600, 600), title: "Timesnapper Checker Settings", flags: "WINDOW|VISIBLE")]
    #[nwg_events( OnWindowClose: [SettingsPopup::close], OnInit: [SettingsPopup::fonty] )]
    window: nwg::Window,

    //#[nwg_resource(size: 6, family: "Comic Sans")]
    //#[nwg_layout_item(layout: grid, row: 0, col: 0)]
    //font: nwg::Font,
    #[nwg_layout(parent: window, spacing: 1)]
    grid: nwg::GridLayout,

    #[nwg_control(text: "", flags:"NONE")]
    #[nwg_layout_item(layout: grid, row: 1, col: 0)]
    state_is_dirty: nwg::Label,

    //Settings ini path field
    #[nwg_control(text: "Settings.ini")]
    #[nwg_layout_item(layout: grid, row: 1, col: 0, col_span: 1)]
    ini_path_main_label: nwg::Label,

    #[nwg_control(text: &get_path_from_confy(), flags: "VISIBLE|DISABLED")]
    #[nwg_layout_item(layout: grid, row: 1, col: 1, col_span: 4)]
    ini_path: nwg::TextInput,

    #[nwg_resource(title:"Timesnapper Checker - Select Settings.ini",action: nwg::FileDialogAction::Open, filters: "Ini(*.ini)")]
    ini_path_file_dialog: nwg::FileDialog,

    #[nwg_control(text: "Select...",focus: true)]
    #[nwg_layout_item(layout: grid, row: 1, col: 5)]
    #[nwg_events( OnButtonClick: [SettingsPopup::ini_path_selector] )]
    ini_path_button_change: nwg::Button,

    #[nwg_control(text: "Timesnapper Checker needs to know where the Timesnapper 'Settings.ini' file is located.\r\nIt is usually here: C:\\Users\\%USERPROFILE%\\AppData\\Roaming\\TimeSnapper\\Settings.ini")]
    #[nwg_layout_item(layout: grid, row: 2, col: 1, col_span: 4, row_span: 2)]
    ini_path_help_label: nwg::Label,

    //filesize field
    #[nwg_control(text: "Blank filesize")]
    #[nwg_layout_item(layout: grid, row: 4, col: 0, col_span: 1)]
    filesize_main_label: nwg::Label,

    #[nwg_control(text: &get_blank_max_filesize_from_confy(), flags: "VISIBLE")]
    #[nwg_layout_item(layout: grid, row: 4, col: 1, col_span: 1)]
    #[nwg_events( OnTextInput: [SettingsPopup::filesize_dirty] )]
    filesize: nwg::TextInput,

    #[nwg_control(text: "Timesnapper checker identifies blank screengrabs by their filesize - which can vary depending on your settings.\r\nEnter a value such as 80000 = 80Kb")]
    #[nwg_layout_item(layout: grid, row: 5, col: 1, col_span: 5, row_span: 2)]
    filesize_help_label: nwg::Label,

    //save and cancel
    #[nwg_control(text: "Save Changes", enabled: false)]
    #[nwg_layout_item(layout: grid, row: 7, col: 4)]
    #[nwg_events( OnButtonClick: [SettingsPopup::save] )]
    button_save: nwg::Button,

    #[nwg_control(text: "Cancel")]
    #[nwg_layout_item(layout: grid, row: 7, col: 5)]
    #[nwg_events( OnButtonClick: [SettingsPopup::cancel] )]
    button_cancel: nwg::Button,
}

impl SettingsPopup {
    fn get_font_of_size(&self, size: u32, is_bold: bool) -> nwg::Font {
        let mut font = Default::default();
        nwg::Font::builder()
            .size(size)
            .family("Segoe UI")
            .weight(if is_bold { 700 } else { 400 })
            .build(&mut font)
            .expect("Failed to build font");
        font
    }

    fn fonty(&self) {
        self.ini_path_main_label
            .set_font(Some(&self.get_font_of_size(18, true)));
        self.ini_path_help_label
            .set_font(Some(&self.get_font_of_size(14, false)));
        self.ini_path
            .set_font(Some(&self.get_font_of_size(18, false)));
        self.ini_path_button_change
            .set_font(Some(&self.get_font_of_size(18, false)));

        //
        self.filesize_main_label
            .set_font(Some(&self.get_font_of_size(18, true)));
        self.filesize_help_label
            .set_font(Some(&self.get_font_of_size(14, false)));
        self.filesize
            .set_font(Some(&self.get_font_of_size(18, false)));
    }

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
        let result = save_to_confy(&self.ini_path.text(), &self.filesize.text());
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
        if self.ini_path_file_dialog.run(Some(&self.window)) {
            if let Ok(file) = self.ini_path_file_dialog.get_selected_item() {
                self.ini_path.set_text(&file);
                self.button_save.set_enabled(true);
                self.state_is_dirty.set_text("path changed");
            }
        }
    }
}
