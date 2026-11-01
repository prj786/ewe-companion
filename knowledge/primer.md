# ewe in brief

<!-- Every question the companion asks a model carries this file, so keep
     it short and true; the topic files hold the detail (cli.md has every
     command-line tool). Checked against ewe 0.39.0-beta, ewe Settings
     0.22, Komble 0.21, ewe Sync 0.16, ewe Browser 0.8 and ewe Safe 0.4
     (2026-11-01, the UI revamp). Comments like this one never reach a
     model. -->

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
  Time and place, User, About this computer. It opens on Appearance; the
  user picks the page, or finds a setting with Ctrl+F.
- **Komble** (the app store): Get apps (Discover, For you, AUR), Your apps
  (Installed, Updates), Desktop (Plugins), and Settings. Clicking a plugin
  opens its sheet: its options, Show in the bar, Show on the desktop,
  Update, Remove. Plugin settings are never in ewe Settings.
- **ewe Sync**: the account (the user's own Nextcloud): Overview (what
  needs you), Folders, Settings & apps (backup and restore), Computers,
  Account, Mail, optional Google.
- **Quick settings** (Super+N, or the pill with the clock at the right of
  the bar): a header (you, the date; Screenshot, Settings, Lock, Power),
  tiles (Wi-Fi, Bluetooth, Do not disturb, Sharing, plugins') whose arrows
  open their pages, shortcuts (Calendar, plugins' pages), volume and
  brightness, the newest notifications, battery and power mode. Esc goes
  back a page, then closes.
- **The launcher** (Super+D) finds apps and files. **The Overview** (tap
  Super alone) spreads out the windows, with the workspaces along the top.
- A terminal (Super+Return, kitty), files (Super+E, Nemo), the browser
  (Super+B: ewe Browser, with its own workspaces on a bar along its
  bottom). **ewe Safe** is ewe's password manager and the SSH agent every
  ssh asks; ewe Browser and Super+P fill logins from it.
- **Plugins** add the dock, clipboard history, screenshots, passwords,
  ewe Safe's bar lock, keep awake, Cast to TV, the System monitor (CPU,
  memory and internet speed in the bar, not in Quick settings) and more. A
  fresh install has none: Komble → Plugins → From ewe.
- **Shaun**, the companion (that is you): the notch under the top bar, Super+A.

What a user should know:

- Passwords and keys never go into ewe.conf, because it syncs. They live in
  the keyring.
- Arch only, pacman only, and the whole system updates at once: Komble →
  Updates → Update all, or `sudo pacman -Syu`. Never update one package
  alone, and never run `pacman -Sy` by itself.
- Only ewe Sync syncs. The account is the user's own Nextcloud; ewe cannot
  create one.
- No Google app ships. Google is optional and needs the user's own OAuth
  client file.
- Plugins run unsandboxed: add third-party ones only from people you trust.
- ewe's command-line tools print JSON: ewe-conf (settings), ewe-theme
  (schemes), ewe-plugin, ewe-bt (Bluetooth), ewe-cloud (the account),
  ewe-safe, ewe-browser, komble (opens the store), ewe-companion (you);
  the shell answers `qs ipc call <target> <function>`.
- Not built in: night light, a screen-recording window, a font picker,
  an airplane switch (you do airplane mode yourself).
