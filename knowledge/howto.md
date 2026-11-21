# How to

<!-- Everyday tasks, with the exact labels from the apps (ewe Settings
     0.22: src/lib/search.js holds every row title; Komble 0.21's plugin
     sheet; ewe Sync 0.16; the shell 0.39.0-beta's QuickSettings.qml and
     the plugins' manifests; re-checked 2026-11-01 after the UI revamp,
     on top of 2026-10-18's power.sh, zzz.sh, Battery.qml and the idle
     stages). Quick settings has no rail since 0.39: a page opens from a
     tile's arrow, a shortcut or the header. What I can do
     myself (wave 2: Wi-Fi, Bluetooth, airplane mode, keeping awake,
     timers, sums, the proposals) is checked against this repo's
     commands.
     Refreshed 2026-11-21 against ewe 0.45.0-beta (e19bfbb: Bar.qml's fold,
     RemotePage.qml, QuickSettings.qml's Tailscale rows, plugins/*/manifest.json
     and bundle.json: dock 1.5.0, switcher 1.0.1, slack 0.4.0, phone 1.0.1,
     places 1.2.0, media 1.1.0; bin/ewe-remote, ewe-tailscale), ewe Settings
     0.25 (search.js, Layout, Displays, Input and DefaultApps.svelte; --page
     works since 0.24, so a Settings button opens its page), ewe Sync 0.17.1
     (Account.svelte: password sign-in), ewe apps 0.1.1 (ewe Files' Trash
     menu). Where a topic names ewe 0.46, the label comes from the
     2026-11-21 wave plan. -->

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
keys: theme, appearance, color scheme, colour scheme, scheme, accent, accent color, accent colour, colors, colours, look
open: settings appearance
ewe Settings → Appearance: Scheme picks the colours (Ewe Dark, Ewe Light, an imported scheme, or "From wallpaper", which makes one from your wallpaper), Accent color picks the highlight, and Look sets Corners, Density and Outlines. Light mode is Ewe Light (or any light scheme): the bar, ewe's apps, GTK and Qt apps, kitty, Zed and the browser all follow, and the lock screen stays dark. From a terminal: `ewe-theme scheme apply ewe-light`, or `ewe-theme scheme from-wallpaper --apply`.

## Make the bar see-through (Glass)
keys: glass, transparent, transparency, translucent, blur, see-through, bar opacity, frosted, opacity
open: settings appearance
ewe Settings → Appearance → Glass. The Glass switch turns it on (80 % the first time, your last value after that), and Opacity, shown once Glass is on, sets how solid the bar, the dock and the lock card are. App blur and Window transparency, under Windows on the same page, are separate switches for app windows.

## Animations and motion
keys: animation, motion, reduce motion, slow animations, fast animations, transitions, turn off animations, effects
open: settings animations
ewe Settings → Animations sets the Speed and Character of how windows, workspaces and panels move, with Fine-tune for each. ewe Settings → Accessibility → Reduce motion turns movement into plain fades. Game mode (the Quick settings tile, or "game mode on") turns animations off until it ends.

## Change the top bar
keys: top bar, bar, bar size, bar icons, hide bar, status icons, bar height, status row, panel
open: settings layout
ewe Settings → Layout → Top bar. Bar icons sets Small, Normal or Large, the Top bar switch hides it (Super+Shift+B does the same), and Fold the extras tucks tray icons, plugin buttons and the tiling switch behind an arrow. Status row, below it, picks the icons on the right: Sound, Microphone in use, Network, Bluetooth, Battery, Power profile, Keyboard layout, System tray, Tiling or floating. A plugin's icon or readout (the System monitor's, say) is switched in Komble → Plugins → the plugin → Show in the bar.

## Fold the bar's extras
keys: fold the extras, fold, chevron, too many icons, bar is full, bar is crowded, tray icons, hide tray icons, collapse the bar
open: settings layout
run: ewe-conf set desktop.bar.collapse true
ewe Settings → Layout → Top bar → Fold the extras (off by default, since ewe 0.45): tray icons, plugin buttons and the tiling switch hide behind an arrow on the bar. Click the arrow ("Show the hidden icons") to slide them out, and again to fold them away. The workspace number, the app's name, updates, the keyboard layout and the clock pill never fold. Press Run to turn it on from here, or ask me: "fold the bar", "unfold the bar" (I show the command first, then offer Undo).

## Make text bigger
keys: text size, accessibility, font size, bigger text, larger text, small text, zoom, readability, font
open: settings accessibility
ewe Settings → Accessibility → Text size: 100 %, 115 % or 130 %. The same page has Increase contrast, Reduce transparency and Reduce motion. There is no font picker; for a sharper or bigger picture on one screen, change its Scale in ewe Settings → Displays.

## Connect to Wi-Fi
keys: wifi, wi-fi, wifi password, wi-fi password, wireless, network, internet, connect, hotspot, ethernet, wired
open: quicksettings wifi
In Quick settings, click the arrow on the Wi-Fi tile, pick the network, type its password and press Join. Saved networks never ask again, and Wired (Ethernet) on the same page is for a cable. VPNs, saved networks and connection details are in ewe Settings → Network. I can switch Wi-Fi itself: "wifi off", "wifi on" (with Undo).

## Pair a Bluetooth device
keys: bluetooth, pair, headphones, earbuds, airpods, speaker, controller, mouse pairing, keyboard pairing
open: quicksettings bt
The arrow on the Bluetooth tile in Quick settings, or ewe Settings → Bluetooth → Nearby devices, then click the device. Put it in pairing mode first (usually hold its button until a light blinks). When a code shows, ewe asks you to confirm it on screen.

Once a device is paired, I can do the rest: "bluetooth on", "bluetooth off", "connect my airpods", "disconnect my headphones" (with Undo). I find the device by the words in its name; when several fit, I list them.

## Change the sound output or microphone
keys: sound, audio, speakers, headphones output, output device, input device, microphone, mic, sound output, audio device
open: quicksettings audio
In Quick settings, the arrow beside the volume slider opens Sound, with Output and Input devices; click the one to use. The volume keys, that slider or scrolling on the clock pill change the volume. I can also do it: say "volume 40" or "mute".

## Change displays and scaling
keys: display, monitor, screen, resolution, refresh rate, scale, scaling, hidpi, rotate, mirror, external monitor, second screen, primary display
open: settings displays
ewe Settings → Displays: drag the screens to arrange them, then set Resolution, Refresh rate, Scale, Rotation, Mirror and the Primary display. A "Keep these display settings?" prompt puts the old ones back unless you confirm. After plugging or unplugging a monitor, a black screen comes back with Super+Shift+R.

## Workspaces on several displays
keys: separate workspaces, shared workspaces, workspaces per display, workspaces per monitor, workspaces on each screen, separate or shared
open: settings displays
With two or more displays, every display has its own workspaces 1, 2, 3 … (like a Mac's separate Spaces): Super+1…8, Super+Shift+1…8 and the three-finger swipe act on the display you are on. For one set across every display, where switching one switches all of them, choose ewe Settings → Displays → Workspaces → Separate or shared → Shared (the row shows while two displays are connected). Since ewe 0.35.

## Night light
keys: night light, night mode, blue light, warm screen, redshift, gammastep, hyprsunset, eye strain
ewe has no night light yet. A light scheme (ewe Settings → Appearance) or lower brightness ("brightness 40") helps in the meantime.

## Do not disturb and notifications
keys: do not disturb, dnd, notifications, silence, quiet, banners, notification history, clear notifications
open: quicksettings notifs
Banners appear at the top right: click one to jump to its app, right-click to dismiss it. The newest show in Quick settings; Show all opens the whole list, with Clear all. Do not disturb is a tile in Quick settings and a switch on the Notifications page; I can switch it too: "dnd on".

## Take a screenshot
keys: screenshot, screen shot, screen capture, capture screen, print screen, snip, snipping
With the Screenshot plugin (Komble → Plugins → From ewe): Print takes the screen, Shift+Print a region and Super+Print the window, or use the camera in the bar. Shots go to ~/Pictures/Screenshots and to the clipboard. I can take one too: "take a screenshot".

## Record the screen
keys: record, screen recording, screen recorder, capture video, obs, record screen
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
keys: keyboard layout, keyboard and mouse, language keyboard, input language, add layout, switch layout, key repeat, typing, num lock, keyboard
open: settings input
ewe Settings → Keyboard and mouse → Keyboard layouts → Add layout. Switch with Super+Space, the two-letter indicator in the bar, or the "Switch layouts with" choice; "Remember the layout for each window" is optional. Key repeat rate, repeat delay and Num Lock at start are on the same page.

Or ask me: "switch to georgian keyboard", "russian keyboard", "next layout" (Undo switches back). I switch only to layouts you added.

A layout changes which letter each key types (Russian, Georgian, German …). For languages typed through a helper, Chinese, Japanese, Korean, Vietnamese or Hindi, use Typing in other languages on the same page (from ewe 0.46): Ctrl+Space turns it on and off, and Super+Space keeps switching layouts.

## Mouse and touchpad settings
keys: mouse, touchpad, trackpad, pointer speed, natural scrolling, tap to click, scroll speed, cursor speed
open: settings input
ewe Settings → Keyboard and mouse: Pointer speed and Scroll speed for the mouse; Tap to click, Natural scrolling, Turn off while typing, Tap and drag and the touchpad's own scroll speed. The pointer's size and colour are in Cursor, on the same page.

## Cursor size and colour
keys: cursor size, cursor colour, cursor color, bigger cursor, cursor bigger, big cursor, pointer size, mouse pointer size, pointer colour, black cursor, white cursor
open: settings input
ewe Settings → Keyboard and mouse → Cursor: Cursor size (24 up to 96 pixels) and Cursor color (Automatic is white with a dark scheme and black with a light one; or always White, or always Black). Apps that were already open use the new cursor after they restart. Since ewe 0.42.

## Update the system
keys: update, upgrade, system update, pacman, update ewe, new version, packages update
open: komble updates
run: sudo pacman -Syu
Click the updates icon in the bar (a count means updates wait), then Komble → Updates → Update all. Arch updates everything at once, so there is no button for a single package, on purpose. From a terminal: `sudo pacman -Syu`. After a desktop update, log out and back in; if it still looks old, run `ewe-setup` and log in again.

## Install or remove an app
keys: install, install app, install software, uninstall, remove app, app store, store, aur, appimage, flatpak, software, download app, package
open: komble
Komble → Discover searches the Arch repositories, the AUR and AppImages; AUR builds community packages (read the PKGBUILD first, then the build button appears); For you lists apps from your other computers. Remove apps from Komble → Installed or the app's page. Say "install obs studio" and I open Komble's search for it; installing stays your click.

## Add, remove or set up a plugin
keys: plugin, add-on, addon, extension, dock plugin, enable plugin, disable plugin, plugin settings, plugin options, plugin sheet, add plugin
open: komble plugins
Komble → Plugins lists every plugin as a row: Install when it is not here, a switch when it is (installing or switching restarts the shell). Click a plugin to open its sheet: version, where it comes from, what it adds and what it needs, its options, Show in the bar and Show on the desktop, and Update or Remove. "Add plugin…" adds one from a git address (plugins run unsandboxed: only from people you trust). From a terminal: `ewe-plugin install ewe.dock`, `ewe-plugin enable|disable <id>`, `ewe-plugin list`. A fresh ewe has none of them.

The plugins from ewe: Dock; Clipboard & emoji (history and an emoji picker under the scissors); Screenshot (Print keys); Passwords (Super+P fills a login from ewe Safe, 1Password, Bitwarden or pass); ewe Safe (a lock in the bar, Quick Access on Super+Shift+P; needs the ewe-safe package); Insomnia (keep the screen awake); System monitor (CPU, memory and internet speed in the bar); SSH (your ~/.ssh/config hosts); VPN (NetworkManager VPNs); Music (what plays, any player); Places (a small file browser); Phone (KDE Connect); Mail (unread IMAP or Gmail mail); Cast to TV (Super+Shift+C); Shaun (me, Super+A; needs the ewe-companion package).

## The dock
keys: dock, taskbar, pinned apps, app dock, dock size, autohide dock, dock on every display, dock on both screens, hide the dock
open: komble options=ewe.dock
There is no dock until you install the Dock plugin (Komble → Plugins → From ewe). It holds the launcher, the Overview, Komble, one box per workspace with its windows, your minimized windows, ewe Browser's downloads and what other plugins add. Its settings are in its sheet (click Dock in Komble → Plugins, or ewe Settings → Layout → Dock settings): Intelligent auto-hide, Icon size, and Show the dock on: Primary (the main display only, the default), All (every display: the others get a small dock with that display's workspaces and the windows minimized there) or None. Ask me "show the dock on all displays", "dock on primary" or "turn off the dock": I show the command first, then offer Undo.

## Sign in to your account and sync
keys: account, nextcloud, sign in, login account, cloud, sync, synchronise, synchronize, settings sync, my files
open: sync
Open ewe Sync → Account, type your Nextcloud's Server address, your Username or email and your Password, then Sign in (the password is used once to get an app password, and never kept). For two-step sign-in or a company login, press Sign in with the browser instead: your server's own page opens in the browser. Then your files are in ~/Nextcloud, your calendar shows in Quick settings, and your settings back up. ewe cannot make an account: get one from a Nextcloud provider or your own server.

## Back up and restore this computer
keys: backup, back up, restore, new computer, new machine, reinstall, migrate, transfer settings, bring everything back
open: sync
ewe Sync → Settings & apps: "Sync now" backs up this computer's settings and apps, the Auto-sync switch keeps doing it whenever something changes, and "Restore…" brings the account's copy back. A new computer that signs in to the same account is offered the backup in Welcome, and ewe Sync's Overview offers "Bring everything back". ewe Safe keeps passwords in your account itself, and other apps' passwords stay in the keyring, so a restored computer may ask for some once. Folders to keep in sync are in ewe Sync → Folders.

## Arrange windows and tiling
keys: tiling, tile windows, floating windows, layout, dwindle, master, scrolling, gaps, borders, window gaps, corner radius
open: settings layout
Windows tile by default. ewe Settings → Layout: Tiling on or off, the Tiling style (Dwindle, Master, Scrolling or Monocle), Inner and Outer gaps, Border width and Corner radius. The tiling switch in the bar flips tiling quickly.

## Open an app on a chosen workspace
keys: window rule, always open, open on workspace, float app, app rule
open: settings windowrules
ewe Settings → Window rules → Add rule: choose the app, then the workspace it opens on, or whether it floats or tiles.

## Default apps
keys: default apps, default browser, open with, default editor, default viewer, default video player, default music player, default pdf viewer, default mail, default file manager
open: settings defaults
ewe Settings → Default apps chooses which app opens each kind of thing: Browser, Mail, Text editor, Image viewer, Video player, Music player, Document viewer, Archives and File manager. Every app follows it, and so do links you open from a terminal; Super+B opens the browser chosen there. ewe's own picks are ewe Browser, ewe Files, ewe Player and ewe Viewer.

For one kind of file, ewe Files does it too: right-click the file → Open with…, pick the app and make it the default for that kind.

## Start apps when you sign in
keys: startup, autostart, start at login, launch at login, startup apps, login items
open: settings startup
ewe Settings → Startup apps → Add app…, then pick it. Each app there has a switch to skip it for now and an × to remove it; changes apply the next time you sign in.

## Time zone and clock
keys: time zone, timezone, time, clock, date, network time, set the clock
open: settings time
ewe Settings → Time and place: Set the time zone automatically (or pick a Region and Zone), and Network time for the clock. The Display language is there too up to ewe 0.45; from ewe 0.46 it has its own page, ewe Settings → Language, and Time and place gets Formats (how dates, times and numbers are written).

Ask me "what time is it" or "what's the date" for the time here, and "what time is it in tokyo" for a city, a country or a zone (Asia/Tokyo): I say the time there and how far ahead or behind you it is, with no AI and no internet.

## Your name and picture
keys: user, username, my name, profile picture, avatar, user picture, account picture
open: settings user
ewe Settings → User: Change picture… (or Use account picture), click your name to change it, and the Picture shape. The account itself (sign-in, backups, Google) is in ewe Sync: "Open ewe Sync" at the top of that page.

## Open an app or a folder by asking
keys: open app, open an app, launch app, start app, open folder, open a folder, open terminal in, terminal in folder, open in terminal, open with
I open any installed app, and apps in a folder. Tell me in the chat (Super+A): "open the file manager", "launch steam", "open the browser", "open a text editor". With a folder: "open kitty in documents", "open zed in ~/Projects/ewe", "open my downloads folder". A folder can be named (documents, downloads, pictures, music, videos, desktop, projects, screenshots, home), written as ~/… or /…, or be just its name somewhere under home ("open a terminal in ewe-companion").

"The browser", "the file manager" and "a text editor" are the ones chosen in ewe Settings → Default apps, and "terminal" is kitty. An app that is open already comes forward instead of opening twice, unless you name a folder for it.

## Find a window, an app or a file
keys: find window, search, find app, find file, overview, launcher, search files, all windows, expose, find a folder, where is my file
open: overview
Tap Super for the Overview: the windows spread out so you can click the one you want, and the workspaces wait along the top (drop a window on one to move it there). Super+D opens the launcher, which searches apps and files; clicking an app that is open already jumps to its window.

Or ask me: "find my resume", "where is the ewe-safe folder", "find pdfs about tax". I look through your home folder by name (not hidden folders or build folders) and list what I find, with Open, Show in folder, Copy path and, for a folder, Open terminal here.

## Cast to a TV
keys: cast, tv, chromecast, miracast, mirror to tv, screen mirroring, smart tv
open: quicksettings cast
With the Cast plugin: the arrow on the Cast tile in Quick settings, or Super+Shift+C, then pick the TV. Open Screen Mirroring on the TV first, keep the laptop near it, and prefer 5 GHz Wi-Fi.

## Fill in a password
keys: password, password manager, 1password, bitwarden, pass, login fill, autofill, super p, 1password cli
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

## Alt+Tab, the Window switcher
keys: alt tab, window switcher, switch windows, switch between windows, cycle windows
open: komble options=ewe.switcher
With the Window switcher plugin (Komble → Plugins → From ewe), hold Alt and press Tab: the open windows show as icons, the most recent first; Tab steps on, Shift+Tab steps back, and letting go of Alt switches. Its option Windows to show picks Workspace (the default), Display or Everywhere; a window elsewhere carries its workspace number. Without the plugin, Super+Tab and Super+Shift+Tab go to the next and previous window.

## Slack on the desktop
keys: slack, slack messages, unread slack, slack token, slack widget
open: komble options=ewe.slack
The Slack plugin (Komble → Plugins → From ewe) puts a card on the desktop with your unread Slack direct messages, newest first: who wrote and what, and a click opens the conversation in the Slack app (or the browser: Open conversations in). On its first start it asks for your Slack token and keeps it in the keyring. Its options: Check every (seconds), Conversations shown, Card (Solid or Glass) and Include bots and Slackbot. Drag the card where you want it, or arrange widgets with Super+Shift+W.

## Your phone (the Phone plugin)
keys: phone plugin, connect my phone, link my phone, kde connect, kdeconnect, android phone, sms, text messages, phone notifications, phone battery
open: quicksettings mobile
The Phone plugin links an Android phone through KDE Connect (Komble installs kdeconnect with the plugin). Put the KDE Connect app on the phone, keep both on the same Wi-Fi, and pair on the Mobile page in Quick settings. Then the page shows the phone's battery, its notifications and your SMS conversations, and a phone sits in the bar while it is connected.

## Places, a small file browser
keys: places, places plugin, pinned folders, quick folders
open: komble options=ewe.places
The Places plugin (Komble → Plugins → From ewe) adds a compact file browser above the dock, or under the bar when there is no dock: your home and your pinned folders, and any file can be dragged out into another app. Its option Where the button lives: Auto (the dock when there is one, else the bar), Bar, Dock or Both. For everything else there is ewe Files (Super+E).

## The Music plugin: what is playing
keys: music plugin, now playing, media controls, playing song, song controls
open: komble options=ewe.media
The Music plugin (Komble → Plugins → From ewe) shows what any player is playing (ewe Player, ewe Browser, Spotify, a browser tab …) in a card above the dock or under the bar: the artwork, play and pause, next and previous, and a seek bar. Its options: Where the button lives (Auto, Bar, Dock or Both) and Show the bar button when nothing is playing.

## Share this computer with your other computers (Remote)
keys: share this computer, sharing tile, remote access, remote desktop, pair a computer, pairing code, my other computer, ssh into my computer, work computer from home
open: quicksettings remote
Quick settings → Sharing (the tile's arrow) → Share this computer lets your other ewe computers, signed in to the same account, open a terminal here, reach its files and browse the web from it. Each one pairs once: on the other computer open Sharing, find this one under Your computers, press Pair and type the 8-digit Pairing code this one shows (it works once, and nobody needs to be at this computer). Then that computer's row has a terminal button and a web button (ewe Browser's Browse from <computer> workspace). It is not a view of the screen: you get a terminal, the files and the web.

To reach a computer at home or at work from another network, both need Tailscale: press Turn on Tailscale on the Sharing page (or Set up Tailscale, one password prompt, then Sign in). Connected now lists who is in, with Disconnect; Can control this computer lists the paired ones, with Remove. Sharing works while you are signed in on the shared computer, and the terminal reconnects by itself after a Wi-Fi drop. From a terminal: `ewe-remote status`, `ewe-remote share on`, `ewe-remote code`.

## Tailscale
keys: tailscale, tailnet, exit node
open: quicksettings wifi
Tailscale comes with ewe but stays off until you set it up: Quick settings → Wi-Fi (the tile's arrow) → Tailscale → Set up Tailscale (one password prompt), then Sign in (your browser opens Tailscale's page). After that the Tailscale switch turns it on and off, Reconnect is there, and an exit node in use shows with Stop using. ewe keeps Tailscale from taking over your local network's routes and DNS. Sharing uses it to reach your computers from another network; `ewe-tailscale status` says how it is.

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
keys: widget, desktop widget, arrange widgets, pin plugin to desktop
Plugins that can sit on the desktop (the System monitor's card, Slack …) go there with Show on the desktop in their sheet: click the plugin in Komble → Plugins. The same sheet pins one above your windows, locks its position or resets it. Super+Shift+W arranges them: drag, pin, lock or hide, then Esc.

## Airplane mode
keys: airplane mode, flight mode, aeroplane mode, plane mode, all radios off, wireless off
ewe has no airplane switch of its own, but I do it: "airplane mode on" turns Wi-Fi and Bluetooth off, and "airplane mode off" (or Undo) puts back the ones that were on. Bluetooth comes back on by itself at the next start of the computer.

## Timers and reminders
keys: timer, set a timer, reminder, remind me, alarm, countdown
Ask me: "set a timer for 5 minutes", "remind me in 10 minutes to call mum", "remind me at 5pm to call the bank", "remind me to stretch at 17:30". When it is due, a notification from Shaun shows (it stays in Quick settings' notifications; Do not disturb hides the banner). "Show my timers" lists them with Cancel, "cancel the timer" cancels one, and Undo right after setting it cancels it too. They wait in my own store, so they survive a restart of the shell or of me; one that fell due while I was off shows when I start again, with the time it was due. A time like "at 5" could be morning or evening: say "5pm" or "17:00". "Remember to call mum at 5pm" is a reminder too; without a time, "remember …" is something I keep in mind (see What I remember).

## Do a sum
keys: calculator, calculate, sum, percent, percentage, maths, arithmetic, how much is
Ask me: "what is 12*7", "15% of 80", "80 + 15%", "20% off 50", "(3 + 4) ^ 2", "100 divided by 8", "sqrt 144", "the square root of 2", "12 squared", "3 cubed". I work it out right here, with no AI and no internet. For more, Galculator is the calculator app (the launcher finds it).

## Empty the trash
keys: empty trash, empty the trash, trash, recycle bin, deleted files, wastebasket
Files moved to the trash stay in ~/.local/share/Trash until you empty it: in ewe Files, right-click Trash in the places on the left → Empty Trash (open Trash to Restore a file, or Delete forever). Or ask me "empty the trash": I say how many items are in it and show `gio trash --empty` with Run; they are gone for good once you press it.

## Units, dates and the time elsewhere
keys: convert units, unit conversion, km to miles, miles to km, celsius to fahrenheit, fahrenheit to celsius, kg to pounds, pounds to kg, cups to ml, days until, days since, what day is, what time is it in, time in another city, world clock
Ask me, with no AI and no internet: "convert 10 km to miles", "20 c in f", "70 kg in pounds", "2 cups in ml", "100 km/h in mph", "how many cm in 3 feet", "1 hour in seconds", "5 gb in mb". Data counts the way disks do (1 GB = 1000 MB); say GiB or MiB for 1024s. Cups, pints and gallons are the US ones. Currencies need today's rates, so I leave those to whoever answers questions.

Dates: "how many days until christmas", "how long until new year", "how many days since 1 january", "what day is 25 december", "what day of the week is 2026-12-25", "what's the date in 30 days", "what was the date 10 days ago". And the time anywhere: "what time is it in tokyo", "time in new york", "what's the time in the uk".

## Ask me about this computer
keys: how much battery, battery left, how much ram, ram is free, using my cpu, using my memory, using my ram, computer is slow, laptop is slow, computer so slow, laptop so slow, uptime, been on, my ip, ip address, am i online, connected to the internet, what takes space, eating my disk, biggest files, disk space is left
Ask me, with no AI and no internet: "how much battery do i have", "how much ram is free", "what's using my memory", "what's using my cpu", "how long has my computer been on", "am i connected to the internet", "what's my ip", "how much disk space is left", "what's eating my disk". "Why is my laptop slow" or "is my computer ok" looks at everything at once (the processor and the apps keeping it busy, memory, the disk, the battery) and says what stands out.

Apps are counted by program: all of Chrome's processes are one "chrome". Memory counts in 1024s like the RAM on the box, disks in 1000s like the disk on the box. Your network's name and your local addresses show in the notch only; your public address would mean asking a website, so I never look it up. "What's eating my disk" sizes the package cache, ~/.cache, the trash, Downloads (with its biggest files), the system journal and crash dumps, and offers Empty for the trash.

## Your own notes for me
keys: my notes, learn this, remember for later, teach you, notes folder, documents shaun, add to my notes, add a note, note for you, write it down
I read your own notes too: Markdown or text files in ~/Documents/Shaun (make the folder, or let me make it). I search them with ewe's guide, so "where is the printer" finds the note that says so, with no AI and no internet, and whoever answers your questions sees the notes that fit. Say "learn this: the printer is on the second floor" or "remember for later: …" and I write a note there (Undo deletes it); "show my notes" lists them, "search my notes for printer" finds one.

A note can be anything you write; a heading (# Printer) names it. Notes sync like any folder if you add ~/Documents to ewe Sync → Folders. Passwords, PINs, keys and card numbers I refuse to write, and leave out of what I read: those belong in ewe Safe. "Remember that …" is different: a short thing I keep in my own memory (What I remember).
<!-- Shaun 0.3 + the 2026-11-21 wave: daemon/notes.rs (EWE_COMPANION_NOTES overrides the folder; .md/.markdown/.txt, two levels, 200 files, 64 KiB each). -->

## Look things up in the ArchWiki
keys: arch wiki, archwiki, wiki, arch-wiki-lite, wiki-search, look it up in the wiki
The ArchWiki is on this computer (the arch-wiki-lite package, about 21 MB), so I can look things up with no internet: "wiki bluetooth headset", "what does the wiki say about fstab", "look up hibernation on the arch wiki". I show the part of the page that fits, with Open the page. With no AI set up, a question whose words are a page's title ("how do I set up pipewire") gets that page's answer too; with one, whoever answers can look in the wiki before answering. From a terminal: wiki-search followed by words, then wiki-search and the page's number.
<!-- daemon/lookup.rs: wiki-search (wiki_width set, so no dialog menu), then wiki-search <id>; the page online when arch-wiki-docs' HTML is not installed. -->

## A command's manual
keys: man page, manual page, command manual, help of a command, options of a command, how do i use a command
Ask me "man rsync", "rsync --help", "what are the options for tar" or "how do I use the rsync command to exclude a folder": I read the command's manual on this computer and show what it does, how to call it and the part that fits, with In terminal to read it all. A program with no manual gets its --help (never for commands that could change something when run).

## Ask me to play something
keys: play from my music, play my music, play my, what's playing, now playing, play this link, play the link, what song is this, shuffle my music
With ewe Player: "play jazz from my music", "play my massive attack", "play my music" (all of it) find songs in your music library (~/Music, by title, artist, album or genre) and play them in ewe Player; "play this link" plays the video or song link you copied (YouTube and many other sites), and "play https://…" a link you type. "What's playing" says what plays and where, with Pause. "Play massive attack" without "my" still asks ewe Browser, which plays from its library or YouTube Music.

"Why won't this play?" looks inside the file or the link you copied (or what ewe Player has open): it names the format in words, says whether this computer can decode it, and offers the fix (gst-libav in Komble) or Open with another app. "Why won't ~/Videos/film.mkv play" names the file.

## Translate
keys: translate, translation, translate this, what does this say, how do you say
Copy some text, then ask me "translate this" (into English) or "translate this to French"; "translate good morning to Georgian" and "how do you say thank you in Japanese" work too. Translating needs someone to answer questions (Komble → Plugins → Shaun → Who answers questions): with none set up I say so. The copied text goes to whoever answers only for that question, and is not kept in our conversation. With one set up, ask me in your own language and I answer in it; without one I answer in English.

## Settings I can change for you
keys: change a setting for me, change settings for me, fold the bar for me, dock on all displays, settings by asking, dark mode for me
Ask me and I show the exact command first (Run changes it, Undo puts it back): "dark mode", "light mode", "show the dock on all displays", "dock on primary", "turn off the dock", "fold the bar", "unfold the bar", "add japanese typing" (Typing in other languages). Do not disturb ("dnd on"), keyboard layouts ("georgian keyboard"), Wi-Fi and Bluetooth switch at once, with Undo. Everything goes through ewe's own tools (ewe-conf, ewe-plugin, ewe-theme), as ewe Settings does: I never edit your settings file.

