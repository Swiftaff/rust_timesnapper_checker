# Rust Timesnapper checker (personal project)

Timesnapper (https://www.timesnapper.com) is great, but sometimes it will save a whole day of blank screengrabs without you realising!

Timesnapper checker is designed to run in the Windows System Tray.
This app is basically an excuse to play with this fantastic library: https://github.com/gabdube/native-windows-gui

This app will automatically popup a notification every hour (from initial launch, not on the hour) with a helpful reminder of:

-   how much time it has snapped today
-   how much of those are blanks!

![Timesnapper checker - Hourly Notification](/readme_pics/readme_0.png)

This tool works by reading the contents of the Timesnapper Snapshots folder.
It grabs the path to the snapshots folder from the Timesnapper 'Settings.ini'.

But you must update the path to that ini file first, via the rust-timesnapper-checker config file which is probably saved somewhere like here:<br>
`C:/Users/***yourusername***/AppData/Roaming/rust-timesnapper-checker/config`

You can also, right-click the tray for a menu

-   Today's Stats - to run it manually as needed
-   About Timesnapper Checker... to remind yourself where the config is
-   Exit...

![Timesnapper checker - Tray Icon Menu](/readme_pics/readme_1.png)

## Installation

## a) Building and Installation on Windows 10

-   clone this repo, then cd inside the folder and build it
    `cargo build --release`

-   manually create a new folder where you want this to live, e.g.
    `C:/Users/***yourusername***/Program Files/TimesnapperChecker/`

-   copy `/target/release/rust_timesnapper_checker.exe` to the folder

-   copy the `/resources/` folder, containing the `cog.ico` into the new folder

-   i.e.

```
//C:/Users/***yourusername***/Program Files/TimesnapperChecker/
//  rust_timesnapper_checker.exe
//  resources
//    cog.ico
```

-   double-click the rust_timesnapper_checker.exe !

## b) Installation of latest release

-   Download the release.zip
-   Expand it
-   double-click the rust_timesnapper_checker.exe !

## Warning

I tested this on exactly 2 PCs. Don't ever use this anywhere important.

Wasn't it Ghandi who once said...<br>
"Don't download and run exe's from the interwebs. If you do this and it breaks something... you are a stoopidhead"
