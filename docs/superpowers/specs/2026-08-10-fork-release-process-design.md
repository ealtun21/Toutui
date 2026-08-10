# The release process of the fork (T-21)

Date: 2026-08-10
Backlog item: T-21, issue #21
Related item: T-14, issue #14

## 1. The problem

`src/utils/clap.rs` gave `--update` and `--uninstall` to a script of the
original project. Every address in that script names `AlbanDAVID/Toutui`, and
that repository is archived. Therefore the commands removed the fork and
installed the original program. The user then lost the corrections of the
fork. The most important loss was T-5, because the original program gives the
token to VLC in a command line argument.

The two commands do nothing now. They write the reason and stop. This
document tells how the fork gets its own way to install, to update, and to
remove.

## 2. The decisions

| Subject | Decision |
|---|---|
| Name | The project becomes `abstui` |
| Repository | `ealtun21/Toutui` becomes `ealtun21/abstui` |
| Version | The first release of the fork is `v0.5.0-beta` |
| Branch | Tags on `main` make the releases. The branch `stable` stops |
| Update | The program updates itself. No script runs |
| Old configuration | The program copies it. It does not move it |
| Old distributions | `cargo-zigbuild` makes the binary for glibc 2.17 |
| Registries | crates.io and the AUR |

## 3. The name

### 3.1 Why the project changes its name

The name `toutui` is free on crates.io. The AUR has `toutui-bin` and
`toutui-git`, and `AlbDav` keeps them. Only these two names collide.

The fork changes its name for a different reason. Two projects with one name
and two maintainers make the users unsure. A different name shows that this is
a different project with a different maintainer.

A measurement on 2026-08-10 shows that `abstui` is free on crates.io and on
the AUR.

### 3.2 What the name changes

| Item | Before | After |
|---|---|---|
| Repository | `ealtun21/Toutui` | `ealtun21/abstui` |
| Crate | `toutui` | `abstui` |
| Binary | `toutui` | `abstui` |
| Configuration directory | `~/.config/toutui/` | `~/.config/abstui/` |
| Log file | `toutui.log` | `abstui.log` |
| Desktop file | `toutui.desktop` | `abstui.desktop` |
| Secret key variable | `TOUTUI_SECRET_KEY` | `ABSTUI_SECRET_KEY` |
| Display name | `Toutui` | `AbsTui` |

GitHub sends the old address of the repository to the new address. Therefore
the copies and the links of the users continue to operate.

### 3.3 The attribution

AlbanDAVID wrote the original program. The fork must show this:

- The README starts with the name of AlbanDAVID and a link to the archived
  repository.
- `Cargo.toml` keeps AlbanDAVID in the field `authors`.
- The LICENSE keeps the line of copyright of AlbanDAVID. The fork adds its own
  line below it.
- The screen of settings shows one line with the credit.

The text must say that this is a fork with a new name. It must not say that
this is a new project.

## 4. The migration of the configuration

### 4.1 The rule

The program does this test one time at each start:

1. If the new directory is present, the program does nothing.
2. If the new directory is absent and the old directory is present, the
   program copies `config.toml`, `.env`, and `db.sqlite3` to the new
   directory. The program then writes one line in the log.
3. If both directories are absent, the program makes a new installation.

The program copies the files. It does not move them. Therefore the old
directory stays complete, and the user can go back to the original program
with the library and the token.

The rule uses these pairs of directories:

| System | Old directory | New directory |
|---|---|---|
| Linux | `$XDG_CONFIG_HOME/toutui/` or `~/.config/toutui/` | the same path with `abstui` |
| macOS | `$XDG_CONFIG_HOME/toutui/` or `~/Library/Preferences/toutui/` | the same path with `abstui` |

### 4.2 The name of the secret key

`.env` holds `TOUTUI_SECRET_KEY`. This key decrypts the token in the database.
If the program only copies `.env`, the program finds no key, and the user must
log in again.

Therefore the migration writes `ABSTUI_SECRET_KEY` in place of
`TOUTUI_SECRET_KEY` in the new `.env`. The value does not change.

The program also accepts `TOUTUI_SECRET_KEY` if it finds no
`ABSTUI_SECRET_KEY`. A user who made `.env` by hand keeps a program that
operates.

### 4.3 The relation to T-14

T-14 says that the program loses the configuration after an update. The rule
in 4.1 corrects this fault, because a change of the directory of configuration
becomes a copy and not a loss. T-14 closes with this work.

### 4.4 The tests

| Test | Condition | Result |
|---|---|---|
| Migration | Only the old directory is present | The program copies the three files |
| No migration | Both directories are present | The program does not write |
| New installation | No directory is present | The program makes the new directory |
| Old directory stays | Only the old directory is present | The old directory does not change |
| The key changes | The old `.env` holds `TOUTUI_SECRET_KEY` | The new `.env` holds `ABSTUI_SECRET_KEY` with the same value |
| The old key operates | Only `TOUTUI_SECRET_KEY` is present | The program decrypts the token |

Each test gives its own `HOME` to the program.

## 5. The release pipeline

### 5.1 The trigger

A tag with the form `v*` on `main` starts the workflow. The workflow makes a
release that is **published**. The current workflow makes a draft, and
therefore the address `/releases/latest` gives nothing. This is the reason why
the fork has no release now.

### 5.2 The targets

| Target | Machine | Method |
|---|---|---|
| `x86_64-unknown-linux-gnu` | ubuntu-latest | `cargo-zigbuild`, glibc 2.17 |
| `aarch64-unknown-linux-gnu` | ubuntu-latest | `cargo-zigbuild`, glibc 2.17 |
| `universal-apple-darwin` | macos-latest | `cargo` |

`cargo-zigbuild` gives a floor of glibc 2.17. Therefore one binary operates
from CentOS 7 and later.

The pipeline makes no binary with musl. The chain of audio is
`rodio` → `cpal` → `alsa` → `alsa-sys`, and `alsa-sys` connects to the library
`libasound` of the system. A binary with musl cannot use a `libasound` that
glibc made. A static `libasound` opens its modules with `dlopen` and reads
`/usr/share/alsa`, and therefore a static build can give a binary that finds no
device of audio. Alpine has no glibc, and thus the users of Alpine must use
`cargo install`.

### 5.3 The assets

| Asset | Contents |
|---|---|
| `abstui-<target>.tar.gz` | The binary |
| `SHA256SUMS` | The sum of each archive |
| `config.example.toml` | The example of configuration |
| `abstui.desktop` | The file of the launcher |

The workflow makes `SHA256SUMS`. No person writes a sum in a file. The array
`sha256sums` in the script of installation goes away.

The workflow uses `actions/attest-build-provenance`. This gives a proof of the
origin of each asset, and it needs no certificate.

### 5.4 The end of the branch `stable`

The releases come from the tags on `main`. The branch `stable` stops. Each
address that names `stable` changes.

## 6. The update in the program

### 6.1 The sequence

`abstui --update` does these steps in this sequence:

1. It asks the API of GitHub for the last release of `ealtun21/abstui`.
2. It compares the version with `CARGO_PKG_VERSION`. If the versions agree, it
   stops and says that the program is up to date.
3. It finds the name of the archive for its own target.
4. It receives the archive and `SHA256SUMS`.
5. It calculates the sum of the archive and compares it with `SHA256SUMS`. If
   the sums disagree, it stops and removes the archive.
6. It opens the archive to a temporary file in the directory of the binary.
7. It moves the temporary file on to the binary with one operation.

### 6.2 The rules

- The program must not run a file that it received. It only moves the binary.
- The program must verify the sum before it moves the binary. Therefore an
  incomplete download leaves the binary that operates.
- If the program cannot write to its own binary, it must stop. It must then
  write the command with `sudo` that the user can use. The program must not
  increase its own permissions.
- The temporary file must be in the same directory as the binary. A move
  between two file systems is not one operation.

### 6.3 The command `--uninstall`

`--uninstall` writes the list of the paths that the user can delete. It
deletes nothing. The command before this one used `sudo rm -r` on paths that
came from variables of the environment, and that is a danger.

### 6.4 `check_update.rs`

The file asks `api.github.com/repos/AlbanDAVID/Toutui/releases/latest` now.
The address becomes `api.github.com/repos/ealtun21/abstui/releases/latest`.

### 6.5 The tests

| Test | Result |
|---|---|
| The versions agree | The program says that it is up to date |
| The sum disagrees | The program stops and does not touch the binary |
| The binary is read-only | The program stops and writes the command with `sudo` |
| The target has no archive | The program stops with a clear message |

A test server gives the answers of the API and the archives. No test uses the
network.

## 7. The script of installation

`hello_toutui.sh` becomes `install.sh`, and it becomes much shorter. crates.io,
the AUR, and the update in the program do most of the work now.

The new script does only this:

1. It identifies the system and the architecture.
2. It receives the archive and `SHA256SUMS` of the last release.
3. It verifies the sum.
4. It installs the binary and, on Linux, the desktop file.
5. It makes `~/.config/abstui/` with `config.example.toml`.

The script removes these parts:

- The installation of VLC and of netcat. The player in the program does not
  need them.
- The array `sha256sums`. The script reads `SHA256SUMS` of the release.
- The menu with three options. `cargo install` replaces the option that
  compiles.
- The call of `check_shasum` at line 15. That call gives two variables that no
  code sets, and therefore the verification of the script never operated.

## 8. The registries

### 8.1 crates.io

The workflow publishes the crate when a tag arrives. The workflow needs the
secret `CARGO_REGISTRY_TOKEN`. The maintainer must make this secret, because
the tool cannot make it.

crates.io does not delete a version. It only marks a version as bad with
`yank`.

### 8.2 The AUR

The repository gets `packaging/aur/PKGBUILD` and `packaging/aur/.SRCINFO` for
the package `abstui-bin`. The `PKGBUILD` uses the archive
`x86_64-unknown-linux-gnu` and the sum from `SHA256SUMS`.

The maintainer sends the package to the AUR, because this needs an account of
the AUR and a key SSH.

## 9. The work that the maintainer must do

| Work | Reason |
|---|---|
| Change the name of the repository on GitHub | The tool has no permission |
| Add `CARGO_REGISTRY_TOKEN` to the secrets | The tool must not make a token |
| Make an account of the AUR and send the package | The tool has no key SSH |

## 10. The sequence of the work

1. The migration of the configuration (section 4). This gives safety to every
   step after it.
2. The change of the name in the code and in the documents (section 3).
3. The workflow of release (section 5).
4. The update in the program (section 6).
5. The new script of installation (section 7).
6. The files of the registries (section 8).

Step 1 comes first. If the name changes before the migration operates, a user
who updates loses the configuration.

---

# Revision, 2026-08-10: the project keeps its name

The maintainer changed three decisions after the work on sections 3, 4, and 8
started. This revision has more importance than the sections above it. Where
this revision and a section above disagree, this revision governs.

## R1. The name does not change

The project keeps the name `toutui`. The binary keeps the name `toutui`. The
directory of configuration keeps the path `~/.config/toutui/`.

**The reasons.** The name `toutui` is free on crates.io, and thus no registry
made the change necessary. Only the two packages of the AUR collide, and the
decision R3 below removes the AUR from this work. The maintainer also finds
that `toutui` is the better name.

Section 3 does not apply. Section 3.3 continues to apply: the README, the
LICENSE, and the screen of settings must name AlbanDAVID as the author of the
original program.

## R2. The program does not copy the configuration

Section 4 does not apply. The directory of configuration does not move,
therefore the program has nothing to copy. The program reads
`TOUTUI_SECRET_KEY` only.

**The consequence for T-14.** Section 4.2 said that the copy closes T-14. That
was true only because the copy made a change of the directory safe. T-14
stays open, and it needs its own examination.

**What stays.** `src/paths.rs` stays. The code that finds the directory of
configuration was in four files, and one module is better than four copies.
`APP_DIR` becomes `"toutui"`, and `OLD_APP_DIR` goes away.

## R3. The work uses no registry

Section 8 does not apply. The work publishes nothing to crates.io and nothing
to the AUR.

**The reasons.** The maintainer has no account of the AUR. crates.io does not
delete a version, and a registry that the maintainer does not need is a
channel that the maintainer must keep.

**The three ways to install.** The README must give these three, and no
other:

1. The script: `curl -LsSf .../install.sh | bash`, which receives the archive
   of the last release and compares the sum.
2. `cargo install --git https://github.com/ealtun21/Toutui`, for a system
   that has no archive, such as Alpine.
3. The releases of GitHub, for a user who wants the archive.

The README must remove every instruction that names a registry, and every
instruction that names the archived original project.

## R4. What does not change

Sections 5, 6, and 7 continue to apply, with the name `toutui` in place of
`abstui`:

- The releases come from a tag on `main`. The workflow makes `SHA256SUMS`
  with the machine, and it publishes the release. The assets have the name
  `toutui-<target>.tar.gz`.
- The program updates itself. It compares the sum before it moves the binary,
  and it runs no file that it receives.
- `hello_toutui.sh` becomes a short `install.sh`.

## R5. The version of the first release

The first release is `0.5.0`, and not `0.5.0-beta`. The candidates before it
are `0.5.0-rc.1`, `0.5.0-rc.2`, and so on.

**The reason.** Semver gives a higher rank to a set of fields of prerelease
that is larger, if the fields before it agree. Therefore `0.5.0-beta-rc1` and
`0.5.0-beta.rc1` both rank ABOVE `0.5.0-beta`. A user on a candidate would
never receive the release. A measurement with the crate `semver` 1 on
2026-08-10 confirms this order.

`0.5.0-rc.1 < 0.5.0-rc.2 < 0.5.0` gives the correct order at every step.

## R6. The floor of glibc is 2.31, and not 2.17

Section 5.2 said that `cargo-zigbuild` gives a floor of glibc 2.17. A
measurement on 2026-08-10 shows that this is not possible.

The program links `libasound` of the system. The machine of the build has
Ubuntu 24.04, therefore its `libasound.so` needs symbols of glibc 2.34 and
later, such as `dlopen@GLIBC_2.34`. A build for a floor of 2.17 that links
that library cannot complete, and the linker says
`undefined reference: dlopen@GLIBC_2.34`.

**The correction.** The two builds for Linux run inside a container of Debian
bullseye. The binary and the `libasound` that it links then come from one
system. The floor becomes glibc 2.31, which covers Debian 11 and later,
Ubuntu 20.04 and later, and RHEL 9.

A floor of 2.17 needs a build of `alsa-lib` from the source for each target.
`alsa-lib` opens its plugins with `dlopen` and reads `/usr/share/alsa`,
therefore that build can give a binary that finds no device of audio. The
project does not do this.
