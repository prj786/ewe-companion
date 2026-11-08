# How to

<!-- Everyday tasks, with the exact labels from the apps (ewe Settings
     0.22: src/lib/search.js holds every row title; Komble 0.21's plugin
     sheet; ewe Sync 0.16; the shell 0.39.0-beta's QuickSettings.qml and
     the plugins' manifests; re-checked 2026-11-01 after the UI revamp,
     on top of 2026-10-18's power.sh, zzz.sh, Battery.qml and the idle
     stages). ewe Settings opens on Appearance whatever page is asked for
     (it still ignores --page in 0.22), so every answer names the page to
     pick. Quick settings has no rail since 0.39: a page opens from a
     tile's arrow, a shortcut or the header. What I can do
     myself (wave 2: Wi-Fi, Bluetooth, airplane mode, keeping awake,
     timers, sums, the proposals) is checked against this repo's
     commands. -->

## Change the wallpaper
keys: wallpaper, background, desktop picture, desktop image, video wallpaper, live wallpaper
open: settings wallpaper
ewe Settings → Wallpaper: pick a picture from the grid or press "Choose file…". "Set for" chooses every display or one at a time. Videos work as wallpapers too, and "Mute video wallpapers" keeps them quiet.

## Turn on light mode
keys: light mode, light theme, turn on light mode, switch to light, white theme, bright theme, day mode, ewe light
open: settings appearance
run: ewe-theme scheme apply ewe-light
Light mode is the Ewe Light scheme: ewe Settings → Appearance → Scheme → Ewe Light, or press Run. The bar, ewe's apps, GTK and Qt apps, kitty, Zed and the browser all follow; the lock screen stays dark. Saying "switch to light mode" to me offers the same command with Run.

## Turn on dark mode
keys: dark mode, dark theme, turn on dark mode, switch to dark, night theme, black theme, ewe dark
open: settings appearance
run: ewe-theme scheme apply ewe-dark
Dark mode is the Ewe Dark scheme, ewe's default: ewe Settings → Appearance → Scheme → Ewe Dark, or press Run. Saying "dark mode" to me offers the same command with Run.

## Change the theme, colours or accent
keys: theme, color scheme, colour scheme, scheme, accent, accent color, accent colour, colors, colours, look
open: settings appearance
ewe Settings → Appearance: Scheme picks the colours (Ewe Dark, Ewe Light, an imported scheme, or "From wallpaper", which makes one from your wallpaper), Accent color picks the highlight, and Look sets Corners, Density and Outlines. Light mode is Ewe Light (or any light scheme): the bar, ewe's apps, GTK and Qt apps, kitty, Zed and the browser all follow, and the lock screen stays dark. From a terminal: `ewe-theme scheme apply ewe-light`, or `ewe-theme scheme from-wallpaper --apply`.

## Make the bar see-through (Glass)
keys: glass, transparent, transparency, translucent, blur, see-through, bar opacity, frosted, opacity
open: settings appearance
ewe Settings → Appearance → Glass. The Glass switch turns it on (80 % the first time, your last value after that), and Opacity, shown once Glass is on, sets how solid the bar, the dock and the lock card are. App blur and Window transparency, under Windows on the same page, are separate switches for app windows.

## Animations and motion
keys: animation, animations, motion, reduce motion, slow animations, fast animations, transitions, turn off animations, effects
open: settings animations
ewe Settings → Animations sets the Speed and Character of how windows, workspaces and panels move, with Fine-tune for each. ewe Settings → Accessibility → Reduce motion turns movement into plain fades. Game mode (the Quick settings tile, or "game mode on") turns animations off until it ends.

## Change the top bar
keys: top bar, bar, bar size, bar icons, hide bar, status icons, bar height, status row, panel
open: settings layout
ewe Settings → Layout → Top bar. Bar icons sets Small, Normal or Large, and the Top bar switch hides it (Super+Shift+B does the same). Status row, below it, picks the icons on the right: Sound, Microphone in use, Network, Bluetooth, Battery, Power profile, Keyboard layout, System tray, Tiling or floating. A plugin's icon or readout (the System monitor's, say) is switched in Komble → Plugins → the plugin → Show in the bar.

## Make text bigger
keys: text size, font size, bigger text, larger text, small text, zoom, readability, font, fonts
open: settings accessibility
ewe Settings → Accessibility → Text size: 100 %, 115 % or 130 %. The same page has Increase contrast, Reduce transparency and Reduce motion. There is no font picker; for a sharper or bigger picture on one screen, change its Scale in ewe Settings → Displays.

## Connect to Wi-Fi
keys: wifi, wi-fi, wifi password, wi-fi password, wireless, network, internet, connect, hotspot, ethernet, wired
open: quicksettings wifi
In Quick settings, click the arrow on the Wi-Fi tile, pick the network, type its password and press Join. Saved networks never ask again, and Wired (Ethernet) on the same page is for a cable. VPNs, saved networks and connection details are in ewe Settings → Network. I can switch Wi-Fi itself: "wifi off", "wifi on" (with Undo).

## Pair a Bluetooth device
keys: bluetooth, pair, pairing, headphones, earbuds, airpods, speaker, controller, mouse pairing, keyboard pairing
open: quicksettings bt
The arrow on the Bluetooth tile in Quick settings, or ewe Settings → Bluetooth → Nearby devices, then click the device. Put it in pairing mode first (usually hold its button until a light blinks). When a code shows, ewe asks you to confirm it on screen.

Once a device is paired, I can do the rest: "bluetooth on", "bluetooth off", "connect my airpods", "disconnect my headphones" (with Undo). I find the device by the words in its name; when several fit, I list them.

## Change the sound output or microphone
keys: sound, audio, speakers, headphones output, output device, input device, microphone, mic, sound output, audio device
open: quicksettings audio
In Quick settings, the arrow beside the volume slider opens Sound, with Output and Input devices; click the one to use. The volume keys, that slider or scrolling on the clock pill change the volume. I can also do it: say "volume 40" or "mute".

## Change displays and scaling
keys: display, displays, monitor, monitors, screen, resolution, refresh rate, scale, scaling, hidpi, rotate, mirror, external monitor, second screen, primary display
open: settings displays
ewe Settings → Displays: drag the screens to arrange them, then set Resolution, Refresh rate, Scale, Rotation, Mirror and the Primary display. A "Keep these display settings?" prompt puts the old ones back unless you confirm. After plugging or unplugging a monitor, a black screen comes back with Super+Shift+R.

## Night light
keys: night light, night mode, blue light, warm screen, redshift, gammastep, hyprsunset, eye strain
ewe has no night light yet. A light scheme (ewe Settings → Appearance) or lower brightness ("brightness 40") helps in the meantime.

## Do not disturb and notifications
keys: do not disturb, dnd, notifications, notification, silence, quiet, banners, notification history, clear notifications
open: quicksettings notifs
Banners appear at the top right: click one to jump to its app, right-click to dismiss it. The newest show in Quick settings; Show all opens the whole list, with Clear all. Do not disturb is a tile in Quick settings and a switch on the Notifications page; I can switch it too: "dnd on".

## Take a screenshot
keys: screenshot, screen shot, screen capture, capture screen, print screen, snip, snipping
With the Screenshot plugin (Komble → Plugins → From ewe): Print takes the screen, Shift+Print a region and Super+Print the window, or use the camera in the bar. Shots go to ~/Pictures/Screenshots and to the clipboard. I can take one too: "take a screenshot".

## Record the screen
keys: record, recording, screen recording, screen recorder, capture video, obs, record screen
There is no recording window in ewe. `gpu-screen-recorder` is installed for the terminal, or install OBS from Komble. Sharing your screen in a call uses ewe's own picker (a screen, a window or a region).

## Lock, sleep, restart and shut down
keys: lock, sleep, suspend, hibernate, restart, reboot, shut down, shutdown, power off, log out, sign out, logout
open: quicksettings power
Super+Alt+L locks, and so does Lock in Quick settings' header. Power, next to it, opens a page with Suspend, Sign out, Restart and Shut down (the last three ask first, because open apps close). Left alone, the screen dims after about 4 minutes 40 seconds and locks at 5; on battery, with Low power mode on (the default), about half that, and it also suspends at 15 minutes and hibernates two hours later. You unlock with your login password.

Ask me: "lock" locks at once; "suspend", "restart", "shut down" and "log out" show ewe's own command (the one the Power page runs) with Run, and nothing happens until you press it.

## Battery and power settings
keys: battery, power, power profile, power mode, performance, power saver, charge limit, battery health, lid, laptop lid, low battery, keyboard backlight
open: settings power
ewe Settings → Power: what the lid does (Suspend even when docked), a battery Charge limit, the Keyboard backlight level, the Power profile, and Low power mode on battery. The foot of Quick settings shows the battery and switches Power saver, Balanced and Performance quickly. ewe warns at 20 % and 10 % battery and hibernates at 5 % (or suspends, when it cannot hibernate).

## Screensaver and auto-lock
keys: screensaver, screen saver, auto lock, idle, timeout, require password, lock after
open: settings saver
ewe Settings → Screensaver: when it starts and its style, and under Locking whether a password is needed and whether the session locks once the screensaver starts.

## Keyboard layouts and typing
keys: keyboard layout, language keyboard, input language, add layout, switch layout, key repeat, typing, num lock, keyboard
open: settings input
ewe Settings → Keyboard and mouse → Keyboard layouts → Add layout. Switch with Super+Space, the two-letter indicator in the bar, or the "Switch layouts with" choice; "Remember the layout for each window" is optional. Key repeat rate, repeat delay and Num Lock at start are on the same page.

## Mouse and touchpad settings
keys: mouse, touchpad, trackpad, pointer speed, natural scrolling, tap to click, scroll speed, cursor speed
open: settings input
ewe Settings → Keyboard and mouse: Pointer speed and Scroll speed for the mouse; Tap to click, Natural scrolling, Turn off while typing, Tap and drag and the touchpad's own scroll speed.

## Update the system
keys: update, updates, upgrade, system update, pacman, update ewe, new version, packages update
open: komble updates
run: sudo pacman -Syu
Click the updates icon in the bar (a count means updates wait), then Komble → Updates → Update all. Arch updates everything at once, so there is no button for a single package, on purpose. From a terminal: `sudo pacman -Syu`. After a desktop update, log out and back in; if it still looks old, run `ewe-setup` and log in again.

## Install or remove an app
keys: install, install app, install software, uninstall, remove app, app store, store, aur, appimage, flatpak, software, download app, package
open: komble
Komble → Discover searches the Arch repositories, the AUR and AppImages; AUR builds community packages (read the PKGBUILD first, then the build button appears); For you lists apps from your other computers. Remove apps from Komble → Installed or the app's page. Say "install obs studio" and I open Komble's search for it; installing stays your click.

## Add, remove or set up a plugin
keys: plugin, plugins, add-on, addon, extension, dock plugin, enable plugin, disable plugin, plugin settings, plugin options, plugin sheet, add plugin
open: komble plugins
Komble → Plugins lists every plugin as a row: Install when it is not here, a switch when it is (installing or switching restarts the shell). Click a plugin to open its sheet: version, where it comes from, what it adds and what it needs, its options, Show in the bar and Show on the desktop, and Update or Remove. "Add plugin…" adds one from a git address (plugins run unsandboxed: only from people you trust). From a terminal: `ewe-plugin install ewe.dock`, `ewe-plugin enable|disable <id>`, `ewe-plugin list`. A fresh ewe has none of them.

The plugins from ewe: Dock; Clipboard & emoji (history and an emoji picker under the scissors); Screenshot (Print keys); Passwords (Super+P fills a login from ewe Safe, 1Password, Bitwarden or pass); ewe Safe (a lock in the bar, Quick Access on Super+Shift+P; needs the ewe-safe package); Insomnia (keep the screen awake); System monitor (CPU, memory and internet speed in the bar); SSH (your ~/.ssh/config hosts); VPN (NetworkManager VPNs); Music (what plays, any player); Places (a small file browser); Phone (KDE Connect); Mail (unread IMAP or Gmail mail); Cast to TV (Super+Shift+C); Shaun (me, Super+A; needs the ewe-companion package).

## The dock
keys: dock, taskbar, pinned apps, app dock, dock size, autohide dock
open: komble options=ewe.dock
There is no dock until you install the Dock plugin (Komble → Plugins → From ewe). Its Intelligent auto-hide and Icon size are in its sheet: click Dock in Komble → Plugins. ewe Settings → Layout → Dock options opens the same place.

## Sign in to your account and sync
keys: account, nextcloud, sign in, login account, cloud, sync, synchronise, synchronize, settings sync, ewe-sync, my files
open: sync
Open ewe Sync → Account and sign in with your Nextcloud server's address; your browser opens your server's own login page. Then your files are in ~/Nextcloud, your calendar shows in Quick settings, and your settings back up. ewe cannot make an account: get one from a Nextcloud provider or your own server.

## Back up and restore this computer
keys: backup, back up, restore, new computer, new machine, reinstall, migrate, transfer settings, bring everything back
open: sync
ewe Sync → Settings & apps: "Sync now" backs up this computer's settings and apps, the Auto-sync switch keeps doing it whenever something changes, and "Restore…" brings the account's copy back. A new computer that signs in to the same account is offered the backup in Welcome, and ewe Sync's Overview offers "Bring everything back". ewe Safe keeps passwords in your account itself, and other apps' passwords stay in the keyring, so a restored computer may ask for some once. Folders to keep in sync are in ewe Sync → Folders.

## Arrange windows and tiling
keys: tiling, tile windows, floating windows, layout, dwindle, master, scrolling, gaps, borders, window gaps, corner radius
open: settings layout
Windows tile by default. ewe Settings → Layout: Tiling on or off, the Tiling style (Dwindle, Master, Scrolling or Monocle), Inner and Outer gaps, Border width and Corner radius. The tiling switch in the bar flips tiling quickly.

## Open an app on a chosen workspace
keys: window rule, window rules, always open, open on workspace, float app, app rule
open: settings windowrules
ewe Settings → Window rules → Add rule: choose the app, then the workspace it opens on, or whether it floats or tiles.

## Default apps
keys: default app, default apps, default browser, open with, default editor, default viewer, default mail, default file manager
open: settings defaults
ewe Settings → Default apps chooses the Browser, Mail, Text editor, Image viewer, Video player and File manager. Super+B opens the browser chosen there.

## Start apps when you sign in
keys: startup, autostart, start at login, launch at login, startup apps, login items
open: settings startup
ewe Settings → Startup apps → Add app…, then pick it. Each app there has a switch to skip it for now and an × to remove it; changes apply the next time you sign in.

## Time zone, clock and language
keys: time zone, timezone, time, clock, date, language, locale, display language, network time
open: settings time
ewe Settings → Time and place: Set the time zone automatically (or pick a Region and Zone), Network time for the clock, and the Display language apps use.

Ask me "what time is it" or "what's the date" for the time here, and "what time is it in tokyo" for a city, a country or a zone (Asia/Tokyo): I say the time there and how far ahead or behind you it is, with no AI and no internet.

## Your name and picture
keys: user, username, my name, profile picture, avatar, user picture, account picture
open: settings user
ewe Settings → User: Change picture… (or Use account picture), click your name to change it, and the Picture shape. The account itself (sign-in, backups, Google) is in ewe Sync: "Open ewe Sync" at the top of that page.

## Open an app or a folder by asking
keys: open app, open apps, open an app, launch app, launch apps, start app, open folder, open a folder, open terminal in, terminal in folder, open in terminal, open with
I open any installed app, and apps in a folder. Tell me in the chat (Super+A): "open nemo", "launch steam", "open the browser", "open a text editor". With a folder: "open kitty in documents", "open documents in nemo", "open zed in ~/Projects/ewe", "open my downloads folder". A folder can be named (documents, downloads, pictures, music, videos, desktop, projects, screenshots, home), written as ~/… or /…, or be just its name somewhere under home ("open a terminal in ewe-companion").

"The browser", "the file manager" and "a text editor" are the ones chosen in ewe Settings → Default apps, and "terminal" is kitty. An app that is open already comes forward instead of opening twice, unless you name a folder for it.

## Find a window, an app or a file
keys: find window, search, find app, find file, overview, launcher, search files, all windows, expose, find a folder, where is my file
open: overview
Tap Super for the Overview: the windows spread out so you can click the one you want, and the workspaces wait along the top (drop a window on one to move it there). Super+D opens the launcher, which searches apps and files; clicking an app that is open already jumps to its window.

Or ask me: "find my resume", "where is the ewe-safe folder", "find pdfs about tax". I look through your home folder by name (not hidden folders or build folders) and list what I find, with Open, Show in folder, Copy path and, for a folder, Open terminal here.

## Cast to a TV
keys: cast, casting, tv, chromecast, miracast, mirror to tv, screen mirroring, smart tv
open: quicksettings cast
With the Cast plugin: the arrow on the Cast tile in Quick settings, or Super+Shift+C, then pick the TV. Open Screen Mirroring on the TV first, keep the laptop near it, and prefer 5 GHz Wi-Fi.

## Fill in a password
keys: password, passwords, password manager, 1password, bitwarden, pass, login fill, autofill, super p, 1password cli
With the Passwords plugin, Super+P opens the picker for the window in front: Enter types the username, Tab, then the password; Ctrl+Enter types only the password; Ctrl+C copies the username and Ctrl+Shift+C the password; Ctrl+P pins the login to that app; with ewe Safe, Ctrl+L locks it. It fills from ewe Safe, ewe's own password manager, once a safe is set up; 1Password, Bitwarden or pass work too (Komble → Plugins → Passwords → Password manager, next to "Press Enter after the password").

If the chosen manager cannot answer (1Password's CLI missing, say) and a safe is set up, the picker offers Use ewe Safe: press Enter. In ewe Browser, ewe Safe fills logins on its own, without the plugin.
<!-- ewe.passwords 1.3.0 (Panel.qml, README): provider auto / ewe / 1password / bitwarden / pass; auto picks Safe once a safe exists; "Use Safe" runs ewe-pass use-safe. -->

## Set up a VPN
keys: vpn, wireguard, openvpn, l2tp, ipsec, tunnel
open: settings network
ewe Settings → Network → VPN → Add VPN…. The VPN plugin adds a VPN tile in Quick settings (its arrow lists your VPNs) and a shield in the bar. Its passwords stay in NetworkManager's own profile, which only the system can read, never in ewe.conf.

## Keep the computer awake
keys: keep awake, stay awake, caffeine, insomnia, prevent sleep, don't sleep, no sleep, screen keeps turning off
The Insomnia plugin (Komble → Plugins → From ewe) adds a Quick settings tile that keeps the screen on: no dimming, no auto-lock and no idle suspend while it is on (closing the lid and locking by hand still work). Its option "Turn off after (minutes)" ends it by itself; a shell restart turns it off. With the plugin on, ask me: "keep the screen awake", and "keep awake off" (or press Undo). ewe also stays awake by itself while something plays or a window is full screen.

## Clipboard history
keys: clipboard, clipboard history, copy history, paste history, copied
With the Clipboard & emoji plugin, click the scissors in the bar to see what you copied and paste it again; its emoji tab picks an emoji. It skips what ewe Safe and password managers copy. It has no keyboard shortcut.

## Mail
keys: mail, email, e-mail, inbox, imap, unread
Set up a mailbox in ewe Sync → Mail (any IMAP account; the password goes to the keyring), or use Gmail through the optional Google sign-in. The Mail plugin then shows the unread count on the envelope in the bar and a Mail shortcut in Quick settings that opens your Inbox; its option "Notifications for new mail" adds a banner per message.

## The System monitor: CPU, memory and internet speed
keys: system monitor, sysmon, cpu usage, memory usage, ram usage, cpu meter, memory meter, internet speed, network speed, download speed, upload speed
open: komble options=ewe.sysmon
The System monitor plugin (Komble → Plugins → From ewe) puts a readout in the top bar, on by default once installed: CPU %, memory % and the download speed. Click it for its card: CPU with a one-minute graph, Memory (used of total, and swap), and Internet (↓ and ↑ with graphs, and the network in use: Wi-Fi and its name, Ethernet, VPN or Offline). The card's "Show on desktop" switch puts the same card on the desktop. It is not in Quick settings any more (since ewe 0.39).

Its options, in its sheet (click System monitor in Komble → Plugins): "In the bar" shows All three, CPU + memory, or Speed (↓ and ↑), and "Speed in" is MB/s (like downloads) or Mbit/s (like speed tests). Show in the bar and Show on the desktop are there too.

## Desktop widgets
keys: widget, widgets, desktop widget, arrange widgets, pin plugin to desktop
Plugins that can sit on the desktop (the System monitor's card, Slack …) go there with Show on the desktop in their sheet: click the plugin in Komble → Plugins. The same sheet pins one above your windows, locks its position or resets it. Super+Shift+W arranges them: drag, pin, lock or hide, then Esc.

## Airplane mode
keys: airplane mode, flight mode, aeroplane mode, plane mode, all radios off, wireless off
ewe has no airplane switch of its own, but I do it: "airplane mode on" turns Wi-Fi and Bluetooth off, and "airplane mode off" (or Undo) puts back the ones that were on. Bluetooth comes back on by itself at the next start of the computer.

## Timers and reminders
keys: timer, timers, set a timer, reminder, reminders, remind me, alarm, countdown
Ask me: "set a timer for 5 minutes", "remind me in 10 minutes to call mum", "remind me at 5pm to call the bank", "remind me to stretch at 17:30". When it is due, a notification from Shaun shows (it stays in Quick settings' notifications; Do not disturb hides the banner). "Show my timers" lists them with Cancel, "cancel the timer" cancels one, and Undo right after setting it cancels it too. They wait in my own store, so they survive a restart of the shell or of me; one that fell due while I was off shows when I start again, with the time it was due. A time like "at 5" could be morning or evening: say "5pm" or "17:00". "Remember to call mum at 5pm" is a reminder too; without a time, "remember …" is something I keep in mind (see What I remember).

## Do a sum
keys: calculator, calculate, sum, percent, percentage, maths, math, arithmetic, how much is
Ask me: "what is 12*7", "15% of 80", "80 + 15%", "20% off 50", "(3 + 4) ^ 2", "100 divided by 8", "sqrt 144", "the square root of 2", "12 squared", "3 cubed". I work it out right here, with no AI and no internet. For more, Galculator is the calculator app (the launcher finds it).

## Empty the trash
keys: empty trash, empty the trash, trash, recycle bin, deleted files, wastebasket
Files moved to the trash stay in ~/.local/share/Trash until you empty it: in Nemo, right-click Trash in the sidebar → Empty Trash. Or ask me "empty the trash": I say how many items are in it and show `gio trash --empty` with Run; they are gone for good once you press it.

## Units, dates and the time elsewhere
keys: convert units, unit conversion, km to miles, miles to km, celsius to fahrenheit, fahrenheit to celsius, kg to pounds, pounds to kg, cups to ml, days until, days since, what day is, what time is it in, time in another city, world clock
Ask me, with no AI and no internet: "convert 10 km to miles", "20 c in f", "70 kg in pounds", "2 cups in ml", "100 km/h in mph", "how many cm in 3 feet", "1 hour in seconds", "5 gb in mb". Data counts the way disks do (1 GB = 1000 MB); say GiB or MiB for 1024s. Cups, pints and gallons are the US ones. Currencies need today's rates, so I leave those to whoever answers questions.

Dates: "how many days until christmas", "how long until new year", "how many days since 1 january", "what day is 25 december", "what day of the week is 2026-12-25", "what's the date in 30 days", "what was the date 10 days ago". And the time anywhere: "what time is it in tokyo", "time in new york", "what's the time in the uk".

## Ask me about this computer
keys: how much battery, battery left, how much ram, ram is free, using my cpu, using my memory, using my ram, computer is slow, laptop is slow, computer so slow, laptop so slow, uptime, been on, my ip, ip address, am i online, connected to the internet, what takes space, eating my disk, biggest files, disk space is left
Ask me, with no AI and no internet: "how much battery do i have", "how much ram is free", "what's using my memory", "what's using my cpu", "how long has my computer been on", "am i connected to the internet", "what's my ip", "how much disk space is left", "what's eating my disk". "Why is my laptop slow" or "is my computer ok" looks at everything at once (the processor and the apps keeping it busy, memory, the disk, the battery) and says what stands out.

Apps are counted by program: all of Chrome's processes are one "chrome". Memory counts in 1024s like the RAM on the box, disks in 1000s like the disk on the box. Your network's name and your local addresses show in the notch only; your public address would mean asking a website, so I never look it up. "What's eating my disk" sizes the package cache, ~/.cache, the trash, Downloads (with its biggest files), the system journal and crash dumps, and offers Empty for the trash.
