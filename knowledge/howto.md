# How to

<!-- Everyday tasks, with the exact labels from the apps (Settings 0.18,
     Komble 0.20, ewe-sync 0.14, the shell 0.29.0-beta; re-checked
     2026-10-18: power.sh, zzz.sh, Battery.qml, Settings.qml's idle stages,
     bundle.json, the plugins' manifests and READMEs). Settings opens on
     Appearance whatever page is asked for (it ignores --page in 0.18),
     so every answer names the page to pick in the sidebar. What I can do
     myself (wave 2: Wi-Fi, Bluetooth, airplane mode, keeping awake,
     timers, sums, the proposals) is checked against this repo's
     commands. -->

## Change the wallpaper
keys: wallpaper, background, desktop picture, desktop image, video wallpaper, live wallpaper
open: settings wallpaper
Settings → Wallpaper: pick a picture from the grid or press "Choose file…". "Set for" chooses every display or one at a time. Videos work as wallpapers too, and "Mute video wallpapers" keeps them quiet.

## Turn on light mode
keys: light mode, light theme, turn on light mode, switch to light, white theme, bright theme, day mode, ewe light
open: settings appearance
run: ewe-theme scheme apply ewe-light
Light mode is the Ewe Light scheme: Settings → Appearance → Scheme → Ewe Light, or press Run. The bar, ewe's apps, GTK and Qt apps, kitty, Zed and the browser all follow; the lock screen stays dark. Saying "switch to light mode" to me offers the same command with Run.

## Turn on dark mode
keys: dark mode, dark theme, turn on dark mode, switch to dark, night theme, black theme, ewe dark
open: settings appearance
run: ewe-theme scheme apply ewe-dark
Dark mode is the Ewe Dark scheme, ewe's default: Settings → Appearance → Scheme → Ewe Dark, or press Run. Saying "dark mode" to me offers the same command with Run.

## Change the theme, colours or accent
keys: theme, color scheme, colour scheme, scheme, accent, accent color, accent colour, colors, colours, look
open: settings appearance
Settings → Appearance: Scheme picks the colours (Ewe Dark, Ewe Light, an imported scheme, or "From wallpaper", which makes one from your wallpaper), Accent color picks the highlight, and Look sets Corners, Density and Outlines. Light mode is Ewe Light (or any light scheme): the bar, ewe's apps, GTK and Qt apps, kitty, Zed and the browser all follow, and the lock screen stays dark. From a terminal: `ewe-theme scheme apply ewe-light`, or `ewe-theme scheme from-wallpaper --apply`.

## Make the bar see-through (Glass)
keys: glass, transparent, transparency, translucent, blur, see-through, bar opacity, frosted, opacity
open: settings appearance
Settings → Appearance → Glass and transparency. The Glass switch turns it on (80 % the first time, your last value after that), and Bar opacity sets how solid the bar, the dock and the lock card are. App blur and Window transparency are separate switches for app windows.

## Animations and motion
keys: animation, animations, motion, reduce motion, slow animations, fast animations, transitions, turn off animations, effects
open: settings animations
Settings → Animations sets the Speed and Character of how windows, workspaces and panels move, with Fine-tune for each. Settings → Accessibility → Reduce motion turns movement into plain fades. Game mode ("game mode on") turns animations off until you leave it.

## Change the top bar
keys: top bar, bar, bar size, bar icons, hide bar, status icons, bar height, status row, panel
open: settings layout
Settings → Layout → Top bar. Bar icons sets Small, Normal or Large, the Top bar switch hides it (Super+Shift+B does the same), and "What the status row shows" picks the icons on the right: sound, microphone, network, Bluetooth, battery, power profile, keyboard layout, tray, tiling. Plugin icons are switched in Komble → Plugins → the plugin → Options → Show in bar.

## Make text bigger
keys: text size, font size, bigger text, larger text, small text, zoom, readability, font, fonts
open: settings accessibility
Settings → Accessibility → Text size: 100 %, 115 % or 130 %. The same page has Increase contrast, Reduce transparency and Reduce motion. There is no font picker; for a sharper or bigger picture on one screen, change its Scale in Settings → Displays.

## Connect to Wi-Fi
keys: wifi, wi-fi, wifi password, wi-fi password, wireless, network, internet, connect, hotspot, ethernet, wired
open: quicksettings wifi
Quick settings → Wi-Fi: pick the network, type its password, press Join. Saved networks never ask again, and the Wired switch is for a cable. VPNs and connection details are in Settings → Networking. I can switch Wi-Fi itself: "wifi off", "wifi on" (with Undo).

## Pair a Bluetooth device
keys: bluetooth, pair, pairing, headphones, earbuds, airpods, speaker, controller, mouse pairing, keyboard pairing
open: quicksettings bt
Quick settings → Bluetooth, or Settings → Bluetooth → Nearby devices, then click the device. Put it in pairing mode first (usually hold its button until a light blinks). When a code shows, ewe asks you to confirm it on screen.

Once a device is paired, I can do the rest: "bluetooth on", "bluetooth off", "connect my airpods", "disconnect my headphones" (with Undo). I find the device by the words in its name; when several fit, I list them.

## Change the sound output or microphone
keys: sound, audio, speakers, headphones output, output device, input device, microphone, mic, sound output, audio device
open: quicksettings audio
Quick settings → Sound lists Output and Input devices; click the one to use. The volume keys, the slider in Quick settings or scrolling on the clock pill change the volume. I can also do it: say "volume 40" or "mute".

## Change displays and scaling
keys: display, displays, monitor, monitors, screen, resolution, refresh rate, scale, scaling, hidpi, rotate, mirror, external monitor, second screen, primary display
open: settings displays
Settings → Displays: drag the screens to arrange them, then set Resolution, Refresh rate, Scale, Rotation, Mirror and the Primary display. A "Keep these display settings?" prompt puts the old ones back unless you confirm. After plugging or unplugging a monitor, a black screen comes back with Super+Shift+R.

## Night light
keys: night light, night mode, blue light, warm screen, redshift, gammastep, hyprsunset, eye strain
ewe has no night light yet. A light scheme (Settings → Appearance) or lower brightness ("brightness 40") helps in the meantime.

## Do not disturb and notifications
keys: do not disturb, dnd, notifications, notification, silence, quiet, banners, notification history, clear notifications
open: quicksettings notifs
Banners appear at the top right: click one to jump to its app, right-click to dismiss it. The history is in Quick settings → Notifications, with Clear all. Do not disturb is a tile on Quick settings' home and a switch on the Notifications tab; I can switch it too: "dnd on".

## Take a screenshot
keys: screenshot, screen shot, screen capture, capture screen, print screen, snip, snipping
With the Screenshot plugin (Komble → Plugins → From ewe): Print takes the screen, Shift+Print a region and Super+Print the window, or use the camera in the bar. Shots go to ~/Pictures/Screenshots and to the clipboard. I can take one too: "take a screenshot".

## Record the screen
keys: record, recording, screen recording, screen recorder, capture video, obs, record screen
There is no recording window in ewe. `gpu-screen-recorder` is installed for the terminal, or install OBS from Komble. Sharing your screen in a call uses ewe's own picker (a screen, a window or a region).

## Lock, sleep, restart and shut down
keys: lock, sleep, suspend, hibernate, restart, reboot, shut down, shutdown, power off, log out, sign out, logout
open: quicksettings
Super+Alt+L locks. The Power button in Quick settings offers Lock, Suspend, Sign out, Restart and Shut down (the last three ask first, because open apps close) and the power mode. Left alone, the screen dims after about 4 minutes 40 seconds and locks at 5; on battery, with Low power mode on (the default), about half that, and it also suspends at 15 minutes and hibernates two hours later. You unlock with your login password.

Ask me: "lock" locks at once; "suspend", "restart", "shut down" and "log out" show ewe's own command (the one the Power button runs) with Run, and nothing happens until you press it.

## Battery and power settings
keys: battery, power, power profile, power mode, performance, power saver, charge limit, battery health, lid, laptop lid, low battery, keyboard backlight
open: settings power
Settings → Power: what the lid does (Suspend even when docked), a battery Charge limit, the Keyboard backlight level, the Power profile, and Low power mode on battery. Quick settings' Power button switches Power saver, Balanced and Performance quickly. ewe warns at 20 % and 10 % battery and hibernates at 5 % (or suspends, when it cannot hibernate).

## Screensaver and auto-lock
keys: screensaver, screen saver, auto lock, idle, timeout, require password, lock after
open: settings saver
Settings → Screensaver: when it starts and its style, and under Locking whether a password is needed and whether the session locks once the screensaver starts.

## Keyboard layouts and typing
keys: keyboard layout, language keyboard, input language, add layout, switch layout, key repeat, typing, num lock, keyboard
open: settings input
Settings → Keyboard and mouse → Keyboard layouts → Add layout. Switch with Super+Space, the two-letter indicator in the bar, or the "Switch layouts with" choice; "Remember the layout for each window" is optional. Key repeat rate, repeat delay and Num Lock at start are on the same page.

## Mouse and touchpad settings
keys: mouse, touchpad, trackpad, pointer speed, natural scrolling, tap to click, scroll speed, cursor speed
open: settings input
Settings → Keyboard and mouse: Pointer speed and Scroll speed for the mouse; Tap to click, Natural scrolling, Turn off while typing, Tap and drag and the touchpad's own scroll speed.

## Update the system
keys: update, updates, upgrade, system update, pacman, update ewe, new version, packages update
open: komble updates
run: sudo pacman -Syu
Click the updates icon in the bar (a count means updates wait), then Komble → Updates → Update all. Arch updates everything at once, so there is no button for a single package, on purpose. From a terminal: `sudo pacman -Syu`. After a desktop update, log out and back in; if it still looks old, run `ewe-setup` and log in again.

## Install or remove an app
keys: install, install app, install software, uninstall, remove app, app store, store, aur, appimage, flatpak, software, download app, package
open: komble
Komble → Discover searches the Arch repositories, the AUR and AppImages; AUR builds community packages (read the PKGBUILD first, then the build button appears); For you lists apps from your other machines. Remove apps from Komble → Installed or the app's page. Say "install obs studio" and I open Komble's search for it; installing stays your click.

## Add, remove or set up a plugin
keys: plugin, plugins, add-on, addon, extension, dock plugin, enable plugin, disable plugin, plugin settings, plugin options
open: komble plugins
Komble → Plugins → From ewe: pick one (installing restarts the shell). Each plugin's settings, and whether it shows in the bar, are under its Options. From a terminal: `ewe-plugin install ewe.dock`, `ewe-plugin enable|disable <id>`, `ewe-plugin list`. A fresh ewe has none of them.

The plugins from ewe: Dock; Clipboard & emoji (history and an emoji picker under the scissors); Screenshot (Print keys); Passwords (Super+P fills a login from Safe, 1Password, Bitwarden or pass); Safe (a lock in the bar, Quick Access on Super+Shift+P; needs the ewe-safe package); Insomnia (keep the screen awake); System monitor (CPU and memory); SSH (your ~/.ssh/config hosts); VPN (NetworkManager VPNs); Music (what plays, any player); Places (a small file browser); Phone (KDE Connect); Mail (unread IMAP or Gmail mail); Cast to TV (Super+Shift+C); Shaun (me, Super+A; needs the ewe-companion package).

## The dock
keys: dock, taskbar, pinned apps, app dock, dock size, autohide dock
open: komble options=ewe.dock
There is no dock until you install the Dock plugin (Komble → Plugins → From ewe). Its auto-hide and icon size are in Komble → Plugins → Dock → Options; Settings → Layout → Dock options opens the same place.

## Sign in to your account and sync
keys: account, nextcloud, sign in, login account, cloud, sync, synchronise, synchronize, settings sync, ewe-sync, my files
open: sync
Open ewe-sync → Account and sign in with your Nextcloud server's address; your browser opens your server's own login page. Then your files are in ~/Nextcloud, your calendar shows in Quick settings, and your settings back up. ewe cannot make an account: get one from a Nextcloud provider or your own server.

## Back up and restore this machine
keys: backup, back up, restore, new computer, new machine, reinstall, migrate, transfer settings
open: sync
ewe-sync → This machine: "Back up this machine", "Sync now", the Auto-sync switch, and "Restore…". A new machine that signs in to the same account is offered the backup in Welcome. Passwords are not in the backup (they stay in the keyring), so a restored machine asks for them once. Folder pairs to keep in sync are in ewe-sync → Folders.

## Arrange windows and tiling
keys: tiling, tile windows, floating windows, layout, dwindle, master, scrolling, gaps, borders, window gaps, corner radius
open: settings layout
Windows tile by default. Settings → Layout: Tiling on or off, the Tiling style (Dwindle, Master or Scrolling), Inner and Outer gaps, Border width and Corner radius. The tiling switch in the bar flips tiling quickly.

## Open an app on a chosen workspace
keys: window rule, window rules, always open, open on workspace, float app, app rule
open: settings windowrules
Settings → Window rules → Add rule: choose the app, then the workspace it opens on, or whether it floats or tiles.

## Default apps
keys: default app, default apps, default browser, open with, default editor, default viewer, default mail, default file manager
open: settings defaults
Settings → Default apps chooses the Browser, Mail, Text Editor, Image Viewer, Video Player and File Manager. Super+B opens the browser chosen there.

## Start apps when you sign in
keys: startup, autostart, start at login, launch at login, startup apps, login items
open: settings startup
Settings → Startup → Add app, then pick it.

## Time zone, clock and language
keys: time zone, timezone, time, clock, date, language, locale, display language, network time
open: settings time
Settings → Time and place: Set the time zone automatically (or pick a Region and Zone), Network time for the clock, and the Display language apps use.

## Your name and picture
keys: user, username, my name, profile picture, avatar, user picture, account picture
open: settings user
Settings → User: Your picture, Your name and the Picture shape. The account section there opens ewe-sync.

## Open an app or a folder by asking
keys: open app, open apps, open an app, launch app, launch apps, start app, open folder, open a folder, open terminal in, terminal in folder, open in terminal, open with
I open any installed app, and apps in a folder. Tell me in the chat (Super+A): "open nemo", "launch steam", "open the browser", "open a text editor". With a folder: "open kitty in documents", "open documents in nemo", "open zed in ~/Projects/ewe", "open my downloads folder". A folder can be named (documents, downloads, pictures, music, videos, desktop, projects, screenshots, home), written as ~/… or /…, or be just its name somewhere under home ("open a terminal in ewe-companion").

"The browser", "the file manager" and "a text editor" are the ones chosen in Settings → Default apps, and "terminal" is kitty. An app that is open already comes forward instead of opening twice, unless you name a folder for it.

## Find a window, an app or a file
keys: find window, search, find app, find file, overview, launcher, search files, all windows, expose, find a folder, where is my file
open: overview
Tap Super for the Overview: window cards at the top, workspaces at the bottom, and typing searches apps, open windows and files. Drop a window card on a workspace to move it. Super+D opens the launcher, which searches apps and files; clicking an app that is open already jumps to its window.

Or ask me: "find my resume", "where is the ewe-safe folder", "find pdfs about tax". I look through your home folder by name (not hidden folders or build folders) and list what I find, with Open, Show in folder, Copy path and, for a folder, Open terminal here.

## Cast to a TV
keys: cast, casting, tv, chromecast, miracast, mirror to tv, screen mirroring, smart tv
open: quicksettings cast
With the Cast plugin: Quick settings → Cast, or Super+Shift+C, then pick the TV. Open Screen Mirroring on the TV first, keep the laptop near it, and prefer 5 GHz Wi-Fi.

## Fill in a password
keys: password, passwords, password manager, 1password, bitwarden, pass, login fill, autofill, super p, 1password cli
With the Passwords plugin, Super+P opens the picker for the window in front: Enter types the username, Tab, then the password; Ctrl+Enter types only the password; Ctrl+C copies the username and Ctrl+Shift+C the password; Ctrl+P pins the login to that app; with Safe, Ctrl+L locks it. It fills from Safe, ewe's own password manager, once a safe is set up; 1Password, Bitwarden or pass work too (Komble → Plugins → Passwords → Options → Password manager, which also has "Press Enter after the password").

If the chosen manager cannot answer (1Password's CLI missing, say) and a safe is set up, the picker offers Use Safe: press Enter. In ewe browser, Safe fills logins on its own, without the plugin.
<!-- ewe.passwords 1.3.0 (Panel.qml, README): provider auto / ewe / 1password / bitwarden / pass; auto picks Safe once a safe exists; "Use Safe" runs ewe-pass use-safe. -->

## Set up a VPN
keys: vpn, wireguard, openvpn, l2tp, ipsec, tunnel
open: settings network
Settings → Networking → Add a VPN. The VPN plugin adds a quick switch in Quick settings and a shield in the bar. Its passwords stay in NetworkManager's own profile, which only the system can read, never in ewe.conf.

## Keep the computer awake
keys: keep awake, stay awake, caffeine, insomnia, prevent sleep, don't sleep, no sleep, screen keeps turning off
The Insomnia plugin (Komble → Plugins → From ewe) adds a Quick settings tile that keeps the screen on: no dimming, no auto-lock and no idle suspend while it is on (closing the lid and locking by hand still work). Its option "Turn off after (minutes)" ends it by itself; a shell restart turns it off. With the plugin on, ask me: "keep the screen awake", and "keep awake off" (or press Undo). ewe also stays awake by itself while something plays or a window is full screen.

## Clipboard history
keys: clipboard, clipboard history, copy history, paste history, copied
With the Clipboard & emoji plugin, click the scissors in the bar to see what you copied and paste it again; its emoji tab picks an emoji. It skips what Safe and password managers copy. It has no keyboard shortcut.

## Mail
keys: mail, email, e-mail, inbox, imap, unread
Set up a mailbox in ewe-sync → Mail (any IMAP account; the password goes to the keyring), or use Gmail through the optional Google sign-in. The Mail plugin then shows the unread count on the envelope in the bar and an Inbox page in Quick settings; its option "Notifications for new mail" adds a banner per message.

## Desktop widgets
keys: widget, widgets, desktop widget, arrange widgets, pin plugin to desktop
Plugins that can sit on the desktop are pinned in Komble → Plugins → the plugin → Options → On the desktop. Super+Shift+W arranges them: drag, pin, lock or hide, then Esc.

## Airplane mode
keys: airplane mode, flight mode, aeroplane mode, plane mode, all radios off, wireless off
ewe has no airplane switch of its own, but I do it: "airplane mode on" turns Wi-Fi and Bluetooth off, and "airplane mode off" (or Undo) puts back the ones that were on. Bluetooth comes back on by itself at the next start of the computer.

## Timers and reminders
keys: timer, timers, set a timer, reminder, reminders, remind me, alarm, countdown
Ask me: "set a timer for 5 minutes", "remind me in 10 minutes to call mum", "remind me at 5pm to call the bank", "remind me to stretch at 17:30". When it is due, a notification from Shaun shows (it stays in Quick settings → Notifications; Do not disturb hides the banner). "Show my timers" lists them with Cancel, "cancel the timer" cancels one, and Undo right after setting it cancels it too. They wait in my own store, so they survive a restart of the shell or of me; one that fell due while I was off shows when I start again, with the time it was due. A time like "at 5" could be morning or evening: say "5pm" or "17:00".

## Do a sum
keys: calculator, calculate, sum, percent, percentage, maths, math, arithmetic, how much is
Ask me: "what is 12*7", "15% of 80", "80 + 15%", "20% off 50", "(3 + 4) ^ 2", "100 divided by 8". I work it out right here, with no AI and no internet. For more, Galculator is the calculator app (the launcher finds it).

## Empty the trash
keys: empty trash, empty the trash, trash, recycle bin, deleted files, wastebasket
Files moved to the trash stay in ~/.local/share/Trash until you empty it: in Nemo, right-click Trash in the sidebar → Empty Trash. Or ask me "empty the trash": I say how many items are in it and show `gio trash --empty` with Run; they are gone for good once you press it.
