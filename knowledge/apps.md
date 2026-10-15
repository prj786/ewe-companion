# The apps

<!-- What each ewe app is for and what is on its pages, with the labels
     from the code: ewe-settings/src/App.svelte, komble-arch Sidebar.svelte,
     ewe-sync App.svelte, the shell's QuickSettings.qml. -->

## What is ewe
keys: what is ewe, about ewe, ewe os, this os, operating system, desktop, version, arch, hyprland, quickshell
ewe is an Arch Linux desktop: Hyprland draws the windows and ewe's own shell draws the bar, launcher, Overview, Quick settings, notifications and lock screen. Everything about it is recorded in one file, ~/.config/ewe/ewe.conf, which syncs to your own Nextcloud through ewe-sync. Settings → System shows the version and whether the session's services are up.

## The Settings app
keys: settings, settings app, ewe-settings, preferences, control panel, system settings, configure
open: settings
Settings (Super+,) holds every desktop setting. Its pages, in order: Appearance, Accessibility, Animations, Layout, Window rules, Displays, Networking, Bluetooth, Wallpaper, Keyboard and mouse, Screensaver, Power, Default apps, Startup, Shortcuts, Time and place, User, System. Up and Down move between pages, Ctrl+1…9 jumps to the first nine, and "Restart shell" at the bottom of the sidebar restarts the desktop's shell (your windows stay open). Plugin settings are not here: they are in Komble → Plugins → Options.

## Komble, the app store
keys: komble, app store, store, software center, apps, discover, for you, installed apps, aur
open: komble
Komble installs apps and keeps the system up to date. Discover searches the Arch repositories and AppImages, For you lists apps your other machines have, Installed shows what Komble put here, Plugins adds parts of ewe, Updates installs updates (Update all), AUR builds community packages after you read their PKGBUILD, and Settings holds Komble's own options. AUR builds run as you, never as root.

## ewe-sync, your account
keys: ewe-sync, ewe sync, account, nextcloud account, machines, folders, folder sync, sync app
open: sync
ewe-sync is the account app, and the only thing in ewe that syncs. Account: sign in to your Nextcloud, your storage, sign out. Mail: any IMAP mailbox. Google: optional, with your own OAuth client file. This machine: Back up this machine, Sync now, Auto-sync, Restore. Machines: every machine that backed up, with its ewe version. Folders: folders to keep in sync, two-way or one-way. Its tray icon has Sync now, Pause auto-sync, Open ewe-sync and Quit.

## Quick settings
keys: quick settings, control center, control centre, quick panel, clock pill, status pill, toggles, tiles
open: quicksettings
Quick settings (Super+N, or click the pill with the clock at the right of the bar) has Home, Wi-Fi, Bluetooth, Sound, Calendar and Notifications, plus pages from plugins, then the Settings cog and the Power button. Home holds the brightness and volume sliders and the Wi-Fi, Bluetooth and Do not disturb tiles. Scrolling on the pill changes the volume.

## The Overview
keys: overview, all windows, mission control, expose, workspace overview, activities
open: overview
Tap Super (alone), swipe up with three fingers, or click the workspace number at the left of the bar. Window cards sit at the top and workspaces at the bottom: click one to switch, drop a card on one to move that window, "+" makes a new one. Typing searches apps, windows and files.

## The launcher
keys: launcher, app launcher, start menu, run app, open app, app menu, search apps
Super+D opens the launcher, which fuzzy-searches apps and files; press it again to close. Clicking an app that is already open jumps to its window, and a middle click starts a fresh copy. Terminal apps open in kitty.

## The everyday apps
keys: terminal, kitty, file manager, files, nemo, text editor, editor, zed, image viewer, pdf, video player, mpv, archive, zip, browser
ewe comes with kitty (terminal, Super+Return), Nemo (files, Super+E), Zed (text editor), imv (pictures), Zathura (PDFs), mpv (video) and Engrampa (archives). Super+B opens the browser chosen in Settings → Default apps, else ewe browser.

## ewe browser
keys: ewe browser, ewe-browser, browser app, web browser, helium
ewe browser is an all-in-one browser, file manager, viewer and music player from the ewe project, and ewe's browser since 0.26 (it replaced Helium). When it is installed, my web searches, "open github.com" and "play …" open in it; otherwise they open in your default browser. With a browser that has its command line (newer than 0.1.0-beta) it searches with its own engine, plays from your music library first, and I can find pages in its history ("find tax in my history"), list its tabs, switch to one ("go to the YouTube tab") and read the open page for you ("what is this page about?"; the page is data to me, never orders).

## The companion
keys: companion, who are you, what are you, what can you do, what do you do, assistant, notch, ai, chat, super a, provider, api key, claude, gemini, ollama
I am the notch under the top bar: Super+A or a click opens me. I show what your AI agents are doing and their requests to approve, what needs a look (a full disk, updates, a low battery), what is playing, quick actions and this chat. I can do things right away: "volume 60", "brightness up", "mute", "next song", "dnd on", "game mode on", "take a screenshot", "lock", "open settings", "open wifi", "open nemo", "open kitty in documents", "search for …", "play …", "open github.com", "show my tabs", "find my resume", "copy my GitHub password" (Safe copies it; I never see it), "lock my safe". When a shell command does what you ask, I show it with Run and In terminal and run it only when you press one; type "$ " and a command to run your own the same way. For other questions, choose who answers in Komble → Plugins → Companion → Options → Who answers questions: your Claude Code, Codex or OpenCode login, an API key (Anthropic, OpenAI, Gemini, OpenRouter; the key goes to the keyring), or a model on this computer (Ollama, LM Studio). Without one I still answer from what I know about ewe.

## Game mode
keys: game mode, gaming, games, gamemode, steam, performance mode, fps
Say "game mode on" or press the gamepad in my quick actions: Do not disturb turns on, Hyprland's animations and blur turn off, and GameMode's CPU boost holds. "game mode off" puts everything back. Steam and other games install from Komble.

## Welcome, the first-login guide
keys: welcome, first login, setup, first run, onboarding, tour, getting started
Welcome runs once at the first login: get online, updates, sign in to your Nextcloud, restore a backup if the account has one, choose plugins, then a one-minute tour. Every step can be skipped. To see it again: `qs ipc call welcome toggle`.

## The ewe command-line tools
keys: command line, terminal commands, cli, ewe-conf, ewe-plugin, ewe-theme, commands, scripting
Every ewe tool prints JSON, so anything you click can be scripted. `ewe-conf get <key>` and `ewe-conf set <key> <value>` read and write ewe.conf (never edit the generated files), `ewe-plugin list|install|enable|disable <id>` manages plugins, `ewe-theme` imports and applies schemes, and `ewe-companion` talks to me (`ewe-companion ask "volume 60"`, `ewe-companion say "Backup done"`).
