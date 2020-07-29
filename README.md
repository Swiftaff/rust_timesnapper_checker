# Rust Timesnapper checker

Timesnapper is great, but sometimes it will save a whole day of blank screengrabs without you realising!

Timesnapper checker is designed to run in your Windows System Tray.
It's basically an excuse to play with this fantastic library: https://github.com/gabdube/native-windows-gui

This app will automatically popup a notification every hour (from initial launch, not on the hour) with a helpful reminder of:

-   how much time it has snapped today
-   how much of those are blanks!

This tool works by reading the contents of the Timesnapper Snapshots folder.
It grabs the path to the snapshots folder from the Timesnapper 'Settings.ini'.

But you must update the path to that ini file first, via the rust-timesnapper-checker config file which is probably saved somewhere like here:<br>
`C:/Users/***yourusername***/AppData/Roaming/rust-timesnapper-checker/config`

You can also, right-click the tray for a menu

-   Today's Stats - to run it manually as needed
-   Settings - to remind yourself where the config is pointed
-   Exit

## Installation on Windows 10

-   clone this repo, then cd inside the folder and build it
    `cargo build --release`

-   manually create a new folder where you want this to live, e.g.
    `C:/Users/***yourusername***/Program Files/TimesnapperChecker/`

-   copy `/target/release/rust_timesnapper_checkerd.exe` to the folder

-   copy the `/resources/` containing the `cog.ico` into the folder

-   i.e.

```
//C:/Users/***yourusername***/Program Files/TimesnapperChecker/
//  rust_timesnapper_checkerd.exe
//  resources
//    cog.ico
```

-   double-click the rust_timesnapper_checkerd.exe !
