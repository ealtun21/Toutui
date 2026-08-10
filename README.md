## 🍴 This is a maintained fork

The original author archived [AlbanDAVID/Toutui](https://github.com/AlbanDAVID/Toutui)
and cannot maintain it. This repository continues the work.

The fork corrects the faults that the original project has, and it adds
functions. `docs/TAKEOVER-BACKLOG.md` holds the full list. Report a fault in
the [issues of this repository](https://github.com/ealtun21/Toutui/issues),
and not in the archived repository.

> [!WARNING]
> The fork has no release yet. Build it from the source, and read
> [T-21](https://github.com/ealtun21/Toutui/issues/21). The commands
> `--update` and `--uninstall` of the original project installed the
> **archived** program, thus this build refuses them.

[![CI](https://github.com/ealtun21/Toutui/actions/workflows/ci.yml/badge.svg)](https://github.com/ealtun21/Toutui/actions/workflows/ci.yml)

# 🦜 Toutui: A TUI Audiobookshelf client for Linux and macOS

<p align="center">
    <em>In French, being "tout ouïe" (toutui) means being all ears.</em>
</p>

<p align="center">
    <img src="assets/demo_3.gif" alt="🎬 Demo">
</p>

<div align="center">
🎨 Explore and try various themes <a href="https://github.com/AlbanDAVID/Toutui-theme">here.</a>
</div>

## ✨ Features  
 **Cross-platform:** <img src=".github/tux.png" align="top" width="24" alt="Tux (Linux)"/>  Linux and <img src=".github/apple.png" align="top" width="24" alt="Apple (macOS)"/> macOS    
 **Lightweight & Fast:** A minimalist terminal user interface (TUI) written in Rust 🦀  
 **Supports Books & Podcasts:** Enjoy both audiobooks and podcasts  
 **Series:** See the series of a library and the books of a series, in sequence (`s`)  
 **Collections & Playlists:** See them and play their contents (`c`)  
 **Sync Progress & Stats:** Keep your listening progress in sync  
 **Streaming Support:** Play directly without downloading  
 **Local Copies:** Download a book or a podcast episode and read the audio from the disk (see below)  
 **Customizable Color Theme:** A config file will allow you to customize the color theme. Explore and try various themes [here](https://github.com/AlbanDAVID/Toutui-theme).

## 📚 Series

Audiobookshelf puts a book in a series and gives the book a number in that
series.

- `s` — show the series of the library. The key operates in the views Home,
  Library, and Search. A podcast library has no series.
- `l`/`Enter` on a series — show the books of that series. The number comes
  first, for example `#1 - The First Book`.
- The books come in the sequence of the series, and not in the sequence of the
  alphabet. Therefore `#10` comes after `#2`.
- `l`/`Enter` on a book plays it. `D` and `X` operate on it in the same way as
  in the Library view.
- `h` goes back one step.

## 🗂 Collections and playlists

A collection holds books, and every user of the server sees it. A playlist
belongs to one user, and it can hold books or episodes of a podcast.

- `c` — show the collections and the playlists of the library. The key
  operates in the views Home, Library, and Search. The collections come first.
- `l`/`Enter` on a list — show its media. `l`/`Enter` on a medium plays it.
- `D` and `X` operate on a book or on an episode of the list, in the same way
  as in the Library view.
- A podcast library has no collection, and it can have a playlist.

## 📥 Offline Mode

Books and podcast episodes can be downloaded locally, so the application reads the audio from the disk and not from the server.

- `D` — download the selected book (Home, Library, or Search views) or the selected podcast episode (Home, or the episode list of a podcast) to `~/.local/share/toutui/downloads/<username>/` (or `$XDG_DATA_HOME/toutui/downloads/<username>/` if set). Requires the "download" permission on your Audiobookshelf user account.
- `X` — remove the local copy of the selected book or episode.
- Each episode is a separate download. Therefore you can download one episode of a podcast and leave the other episodes on the server.
- A downloaded book or episode is marked `[Downloaded]` in the info panel.
- Playing a downloaded book or episode (`l`/`Enter`) reads the local file directly. A copy on the disk always has more importance than the server. The application uses the copy on the disk only if the disk holds every audio file of the book; it does not mix the two sources in one book.
- The application pushes the progress to the server during the playback, thus other Audiobookshelf clients stay in sync.

### The application with no server

The application starts and plays when the server does not answer:

- The header shows `📴 Offline`, and the Library view holds the media of the disk only. A media that the disk does not hold cannot play, thus the list does not show it.
- The playback needs no session on the server. The position comes from the local database, and it goes back to the local database for each second.
- When the playback stops, the position waits in the database. The header counts the positions that wait.
- The application sends each position as soon as the server answers again, and the user does nothing. The application also does not need a restart: a background task tries every 30 seconds, and it examines the addresses of the server itself.
- If a different client wrote a newer position while you listened offline, that newer position stays. The application compares its own time with `lastUpdate` of the server.
- A user with an account on more than one server keeps the media and the positions of each server separate. One server can have many addresses (`[[servers]]` in the configuration file), and every address of that server gives the same result.
- Press `R` to try the server again at any moment.

## 📰 Media
<img src=".github/korben.png" align="top" width="50" alt="Korben"/> Featured on [Korben](https://korben.info/toutui-client-terminal-audiobookshelf.html), a well-known French tech blog covering open source and technology.


## 🛠️ Roadmap  
**Short-term Goals**  
- Since this is a beta version, the main focus is on tracking and fixing bugs.
- Improve the design of the integrated player.
- **Currently working on the next release: [v1.0.0-stable].**


**Mid-term Goals**   
- CI/CD Implementation  
- Add future features described bellow.

## 🔮 Future features
Here are some features that could be added in future releases:
- Ability to add new podcasts from the app
- Add stats
- Read an EPUB book in the application
- A full offline mode: the application needs the server to start today
  
## ⚠️ Caution: Beta Version  
This beta app is still in **heavy development and contains bugs**.  
❗Please check [here](https://github.com/ealtun21/Toutui/blob/main/known_bugs.md) for known bugs especially **MAJOR BUGS** before using the app, so you can use it with full awareness of any known issues.  
If you encounter any issues that are **not yet listed** in the Issues section or into [known bugs](https://github.com/ealtun21/Toutui/blob/main/known_bugs.md), please **open a new issue** to report them.  

🔐 Although it's a beta version, you can use this app with **minimal risk** to your Audiobookshelf library.  
At worst, you may experience **sync issues**, but there is **no risk** of data loss, deletion, or irreversible changes (API is just used to retrieve books and sync them).

## 📝 Notes
### 🐛 **Issues**    
For any issues, check first the [issues of this fork](https://github.com/ealtun21/Toutui/issues) and the [wiki of the original project](https://github.com/AlbanDAVID/Toutui/wiki/). Otherwise, open a new one **in this repository**. The original repository is archived and takes no report.

### 🤝 **Contributing**  
Do not hesitate to contribute to this project by submitting your code, ideas, or feedback. Please make sure to read the [contributing guidelines](https://github.com/ealtun21/Toutui/blob/main/CONTRIBUTING.md) first.

### 🔁 Branching workflow 
This project follow this [branching workflow](https://gist.github.com/digitaljhelms/4287848). 

### 🎨 **UI**
Explore and share themes [here](https://github.com/AlbanDAVID/Toutui-theme).    
The **font** and **emojis** may vary depending on the terminal you are using.    
To ensure the best experience, it's recommended to use **Kitty** or **Alacritty** terminal.



## 🚨 Installation Instructions

>[!WARNING]
> - **This is a beta app, please read [this](https://github.com/AlbanDAVID/Toutui?tab=readme-ov-file#%EF%B8%8F-caution-beta-version).**
>  - For any issues, check first the [issues of this fork](https://github.com/ealtun21/Toutui/issues) and the [wiki of the original project](https://github.com/AlbanDAVID/Toutui/wiki/). Otherwise, open a new one **in this repository**. The original repository is archived and takes no report.

### <img src=".github/archlinux-icon.svg" align="top" width="24" alt="(Arch Linux)"/> Arch Linux
[![GitHub release](https://img.shields.io/github/v/release/AlbanDAVID/Toutui?label=Latest%20Release&color=green&cacheSeconds=3600)](https://github.com/AlbanDAVID/Toutui/releases/latest)
![AUR Version](https://img.shields.io/aur/version/toutui-bin?color=green&label=AUR)

Installation and initial configuration
```
yay -S toutui
mkdir -p ~/.config/toutui
cp /usr/share/toutui/config.example.toml ~/.config/toutui/config.toml
# Token encryption in the database (NOTE: replace 'secret'):
echo 'TOUTUI_SECRET_KEY=secret' >> ~/.config/toutui/.env
```
Update
```
yay -S toutui
```
Uninstall
```
yay -R toutui-bin
```

### ⚡ Easy installation 

**Run the following in your terminal, then follow the on-screen instructions:**    

[![GitHub release](https://img.shields.io/github/v/release/AlbanDAVID/Toutui?label=Latest%20Release&color=green&cacheSeconds=3600)](https://github.com/AlbanDAVID/Toutui/releases/latest)


```bash
bash -c 'expected_sha256="b5c41bcd3c480fd2ca6ec0031ccecf2cf7cf4ae01f591cad64a320fa7d72331d" export expected_sha256 tmpfile=$(mktemp) && curl -LsSf https://github.com/AlbanDAVID/Toutui/raw/stable/hello_toutui.sh -o "$tmpfile" && bash "$tmpfile" install && rm -f "$tmpfile"'
```

#### **Update**

> [!IMPORTANT]  
> `toutui --update` is not working. You can do this instead: 
> ``` 
> bash -c 'expected_sha256="b5c41bcd3c480fd2ca6ec0031ccecf2cf7cf4ae01f591cad64a320fa7d72331d" export expected_sha256 tmpfile=$(mktemp) && curl -LsSf https://github.com/AlbanDAVID/Toutui/raw/stable/hello_toutui.sh -o "$tmpfile" && bash "$tmpfile" update && rm -f "$tmpfile"'
> ```

Quit the app and run the following in your terminal

```bash
bash -c 'expected_sha256="b5c41bcd3c480fd2ca6ec0031ccecf2cf7cf4ae01f591cad64a320fa7d72331d" export expected_sha256 tmpfile=$(mktemp) && curl -LsSf https://github.com/AlbanDAVID/Toutui/raw/stable/hello_toutui.sh -o "$tmpfile" && bash "$tmpfile" update && rm -f "$tmpfile"'
```

#### **Uninstall**

> [!IMPORTANT]  
> `toutui --uninstall` is not working. You can do this instead: 
> ``` 
> bash -c 'expected_sha256="b5c41bcd3c480fd2ca6ec0031ccecf2cf7cf4ae01f591cad64a320fa7d72331d" export expected_sha256 tmpfile=$(mktemp) && curl -LsSf https://github.com/AlbanDAVID/Toutui/raw/stable/hello_toutui.sh -o "$tmpfile" && bash "$tmpfile" uninstall && rm -f "$tmpfile"'
> ```

Quit the app and run the following in your terminal


```bash
bash -c 'expected_sha256="b5c41bcd3c480fd2ca6ec0031ccecf2cf7cf4ae01f591cad64a320fa7d72331d" export expected_sha256 tmpfile=$(mktemp) && curl -LsSf https://github.com/AlbanDAVID/Toutui/raw/stable/hello_toutui.sh -o "$tmpfile" && bash "$tmpfile" uninstall && rm -f "$tmpfile"'
```

#### **Notes**  

##### Files installed:
In `/usr/local/bin` (option 1, from install script) or `~/.cargo/bin` (option 2, from install script) or `/usr/bin` (yay)  :
- `toutui` - The binary file.

In `~/.config/toutui` for Linux or `~/Library/Preferences` for macOS:    
**Note**: This is the default path if `XDG_CONFIG_HOME` is empty. 
- `.env` - Contains the secret key.
- `config.toml` - Configuration file.
- `toutui.log` - Log file.
- `db.sqlite3` - SQLite database file.

In `~/.local/share/applications` (option 1, from install script) or `/usr/share/applications` (yay) for Linux:
- `toutui.desktop` - Config file to launch Toutui from a launcher app.

In `/usr/share/toutui` (yay):
- `config.example.toml` - Configuration file.

### Install from source

>[!WARNING]
> This is a beta app, please read [this](https://github.com/AlbanDAVID/Toutui?tab=readme-ov-file#%EF%B8%8F-caution-beta-version).  

#### **Nix**

The repository holds a flake. Nix then gives every dependency, and it also
gives the ALSA library that the audio engine needs.

```bash
nix build github:ealtun21/Toutui
nix run github:ealtun21/Toutui
nix develop            # a shell for development
```

A build on 2026-08-10 completed, and the tests ran inside the sandbox of Nix.
That sandbox has no sound card and no network, thus the tests do not need
either one.

#### **Requirements**
- `Rust`
- On Linux, the ALSA development package. On Debian and Ubuntu the name is
  `libasound2-dev`. On Fedora the name is `alsa-lib-devel`.

The application plays the audio itself. It does not need VLC, and it does not
need Netcat.

The build compiles two dependencies from C source: SQLite (through
`rusqlite`) and `ring` (through `rustls`). The goal of the project is a build
with no C. Issue 20 holds that work. The audio engine compiles no C.

The application plays these formats: mp3, m4b, m4a, mp4, aac, flac, wav, aiff,
ogg, oga, mka, webm, caf, mpeg, and mpg. It does not play opus, wma, or awb.

The application uses the default sound device. If your computer has more than
one sound card, give the name of the device in the variable
`TOUTUI_AUDIO_DEVICE`:

```bash
TOUTUI_AUDIO_DEVICE="pipewire" toutui
```

[![GitHub release](https://img.shields.io/github/v/release/AlbanDAVID/Toutui?label=Latest%20Release&color=green&cacheSeconds=3600)](https://github.com/AlbanDAVID/Toutui/releases/latest)

Note: `main` might be unstable. Prefer `git clone --branch stable --single-branch https://github.com/AlbanDAVID/Toutui` if you want to have the last stable release.    
```bash
git clone https://github.com/AlbanDAVID/Toutui
cd Toutui/
mkdir -p ~/.config/toutui
cp config.example.toml ~/.config/toutui/config.toml
```

Token encryption in the database (<u>**NOTE**</u>: replace `secret`)
```bash
echo TOUTUI_SECRET_KEY=secret >> ~/.config/toutui/.env
```

```bash
cargo run --release
```

#### **Update**

When a new release is available, follow these steps:

```bash
git pull https://github.com/AlbanDAVID/Toutui
cargo run --release
```

#### **Notes**  
##### Exec the binary:
```bash
cd target/release
./Toutui
```

##### Files installed:
After installation, you will have the following files in `~/.config/toutui`
- `.env` - Contains the secret key.
- `config.toml` - Configuration file.
- `toutui.log` - Log file.
- `db.sqlite3` - SQLite database file.
