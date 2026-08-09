## ⚠️ I'm not able to properly maintain this project anymore. That's why I archived this repo. Thus, please don't wait for any new releases and issue fixing. Of course, you can still install the app and use it. Although it's a beta version, it might still work well for you. Thanks!

[![GitHub release](https://img.shields.io/github/v/release/AlbanDAVID/Toutui?label=Latest%20Release&color=green&cacheSeconds=3600)](https://github.com/AlbanDAVID/Toutui/releases/latest)
![AUR Version](https://img.shields.io/aur/version/toutui-bin?color=green&label=AUR)
[![Release](https://github.com/AlbanDAVID/Toutui/actions/workflows/release.yml/badge.svg)](https://github.com/AlbanDAVID/Toutui/actions/workflows/release.yml)

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
 **Sync Progress & Stats:** Keep your listening progress in sync  
 **Streaming Support:** Play directly without downloading  
 **Offline Mode:** Download audiobooks to listen to them without a network connection (see below)  
 **Customizable Color Theme:** A config file will allow you to customize the color theme. Explore and try various themes [here](https://github.com/AlbanDAVID/Toutui-theme).

## 📥 Offline Mode

Books can be downloaded locally so you can keep listening when your Audiobookshelf server isn't reachable.

- `D` — download the currently selected book (Home, Library, or Search views) to `~/.local/share/toutui/downloads/<username>/` (or `$XDG_DATA_HOME/toutui/downloads/<username>/` if set). Requires the "download" permission on your Audiobookshelf user account.
- `X` — remove the local offline copy of the currently selected book.
- Downloaded books are marked `[Downloaded]` in the book info panel.
- Playing a downloaded book (`l`/`Enter`) reads the local file directly — no server round-trip is needed to start or control playback. A copy on the disk always has more importance than the server. Playback position is always saved locally so it resumes where you left off, even fully offline.
- If the server is reachable while you're listening to a downloaded book, progress is also best-effort pushed to it (same as streaming playback), so other Audiobookshelf clients stay in sync. If it isn't reachable, that push just fails silently and the local resume point is unaffected.
- Currently books only; podcast episode downloads are not yet supported.

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
- Playlist/Collections view
- Ability to add new podcasts from the app
- Add stats
- Offline mode for podcast episodes
  
## ⚠️ Caution: Beta Version  
This beta app is still in **heavy development and contains bugs**.  
❗Please check [here](https://github.com/AlbanDAVID/Toutui/blob/main/known_bugs.md) for known bugs especially **MAJOR BUGS** before using the app, so you can use it with full awareness of any known issues.  
If you encounter any issues that are **not yet listed** in the Issues section or into [known bugs](https://github.com/AlbanDAVID/Toutui/blob/main/known_bugs.md), please **open a new issue** to report them.  

🔐 Although it's a beta version, you can use this app with **minimal risk** to your Audiobookshelf library.  
At worst, you may experience **sync issues**, but there is **no risk** of data loss, deletion, or irreversible changes (API is just used to retrieve books and sync them).

## 📝 Notes
### 🐛 **Issues**    
For any issues, check first the [wiki](https://github.com/AlbanDAVID/Toutui/wiki/) and [issues](https://github.com/AlbanDAVID/Toutui/issues). Otherwise, open a new one.

### 🤝 **Contributing**  
Do not hesitate to contribute to this project by submitting your code, ideas, or feedback. Please make sure to read the [contributing guidelines](https://github.com/AlbanDAVID/Toutui/blob/main/CONTRIBUTING.md) first.

### 🔁 Branching workflow 
This project follow this [branching workflow](https://gist.github.com/digitaljhelms/4287848). 

### 🎨 **UI**
Explore and share themes [here](https://github.com/AlbanDAVID/Toutui-theme).    
The **font** and **emojis** may vary depending on the terminal you are using.    
To ensure the best experience, it's recommended to use **Kitty** or **Alacritty** terminal.



## 🚨 Installation Instructions

>[!WARNING]
> - **This is a beta app, please read [this](https://github.com/AlbanDAVID/Toutui?tab=readme-ov-file#%EF%B8%8F-caution-beta-version).**
>  - For any issues, check first the [wiki](https://github.com/AlbanDAVID/Toutui/wiki/) and [issues](https://github.com/AlbanDAVID/Toutui/issues). Otherwise, open a new one.

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

#### **Requirements**
- `Rust`
- On Linux, the ALSA development package. On Debian and Ubuntu the name is
  `libasound2-dev`. On Fedora the name is `alsa-lib-devel`.

The application plays the audio itself. It does not need VLC, and it does not
need Netcat.

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
