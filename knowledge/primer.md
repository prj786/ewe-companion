# ewe in brief

<!-- Goes with every model question: keep it short and true, under
     4000 bytes. Details live in the topic files (cli.md: every tool;
     news.md: every release). Checked 2026-11-21 against ewe 0.45.0-beta,
     ewe Settings 0.25 (--page works since 0.24), Komble 0.23.1, ewe Sync
     0.17.1, ewe Browser 0.8.4, ewe Safe 0.5.1, ewe apps 0.1.1. -->

ewe is an Arch Linux desktop. Hyprland draws the windows; ewe's own shell
draws the top bar, the launcher, the Overview, Quick settings,
notifications and the lock screen. It is dark by default, with a light mode:
ewe Settings → Appearance → Scheme → Ewe Light (every app follows; the lock
screen stays dark).

Everything about the desktop is recorded in one file, `~/.config/ewe/ewe.conf`.
The ewe Settings app, Komble and the `ewe-conf` command all write it, and
nothing else should: files under `~/.config/hypr/generated/` are rebuilt
from it, so edits there vanish. The file syncs to the user's own Nextcloud
through ewe Sync, so a new computer can be restored from it.

Where things are:

- **ewe Settings** (Super+,): every desktop setting, in four groups. Look:
  Appearance, Wallpaper, Animations, Accessibility. Desktop: Layout, Window
  rules, Displays, Startup apps, Keyboard shortcuts. Devices: Network,
  Bluetooth, Keyboard and mouse, Power, Screensaver. System: Default apps,
  Time and place, User, About this computer. `ewe-settings --page=displays`
  opens a page; Ctrl+F finds a setting.
- **Komble** (the app store): Get apps (Discover, For you, AUR), Your apps
  (Installed, Updates), Desktop (Plugins), and Settings. Clicking a plugin
  opens its sheet: its options, Show in the bar, Show on the desktop,
  Update, Remove. Plugin settings are never in ewe Settings.
- **ewe Sync**: the account (the user's own Nextcloud): Overview, Folders,
  Settings & apps (backup and restore), Computers, Account, Mail, Google.
- **Quick settings** (Super+N, or the pill with the clock): you and the
  date, Settings, Lock, Power; tiles (Wi-Fi, Bluetooth, Do not disturb,
  Game mode, Sharing, plugins') whose arrows open their pages; Calendar;
  volume, brightness, notifications, battery. Esc goes back, then closes.
- **The launcher** (Super+D) finds apps and files. **The Overview** (tap
  Super alone) spreads out the windows, with the workspaces along the top.
- Apps: a terminal (Super+Return, kitty), **ewe Files** (Super+E),
  **ewe Player** (video, music), **ewe Viewer** (PDFs, books, pictures),
  **ewe Browser** (Super+B). **ewe Safe** is ewe's password manager and the
  SSH agent every ssh asks; ewe Browser and Super+P fill logins from it.
- **Plugins** add the dock, clipboard history, screenshots, passwords,
  ewe Safe's bar lock, keep awake, Cast to TV, Alt+Tab, the System monitor
  (CPU, memory and speed in the bar) and more. A fresh install has none:
  Komble → Plugins → From ewe.
- **Shaun**, the companion (that is you): the notch under the top bar, Super+A.

What a user should know:

- Passwords and keys never go into ewe.conf, because it syncs. They live in
  the keyring.
- Arch only, pacman only, and the whole system updates at once: Komble →
  Updates → Update all, or `sudo pacman -Syu`. Never update one package
  alone, and never run `pacman -Sy` by itself.
- Only ewe Sync syncs. The account is the user's own Nextcloud; ewe cannot
  create one. No Google app ships; Google needs the user's own OAuth file.
- Plugins run unsandboxed: add third-party ones only from people you trust.
- ewe's command-line tools print JSON: ewe-conf (settings), ewe-theme
  (schemes), ewe-plugin, ewe-bt (Bluetooth), ewe-cloud (the account),
  ewe-game, ewe-remote, ewe-safe, ewe-browser, komble (opens the store),
  ewe-companion (you); the shell answers `qs ipc call <target> <function>`.
- Not built in: night light, a screen-recording window, a font picker,
  an airplane switch (you do airplane mode yourself).
