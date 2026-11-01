# The apps

<!-- What each ewe app is for and what is on its pages, with the labels
     from the code: ewe-settings src/lib/pages.js and search.js (0.22),
     komble-arch Sidebar.svelte, Plugins.svelte and PluginSheet.svelte
     (0.21), ewe-sync App.svelte and its pages (0.16; no tray icon since
     0.13), the shell's QuickSettings.qml and Overview.qml (ewe 0.39.0-beta,
     origin/main db58e95), ewe Browser 0.8 (BottomBar.tsx, models.ts);
     re-checked 2026-11-01 after the UI revamp. The command-line tools are
     in cli.md. -->

## What is ewe
keys: what is ewe, about ewe, ewe os, this os, operating system, desktop, version, arch, hyprland, quickshell
ewe is an Arch Linux desktop: Hyprland draws the windows and ewe's own shell draws the bar, launcher, Overview, Quick settings, notifications and lock screen. Everything about it is recorded in one file, ~/.config/ewe/ewe.conf, which syncs to your own Nextcloud through ewe Sync. ewe Settings → About this computer shows the version and whether the session's services are up.

## The ewe Settings app
keys: settings, settings app, ewe-settings, preferences, control panel, system settings, configure
open: settings
ewe Settings (Super+,) holds every desktop setting, in four groups. Look: Appearance, Wallpaper, Animations, Accessibility. Desktop: Layout, Window rules, Displays, Startup apps, Keyboard shortcuts. Devices: Network, Bluetooth, Keyboard and mouse, Power, Screensaver. System: Default apps, Time and place, User, About this computer. "Find a setting" at the top of the list (Ctrl+F) searches every page and row, and opens the one you pick. Up and Down move between pages and Ctrl+1…9 jumps to the first nine. Restart the shell is in About this computer (your windows stay open). Plugin settings are not here: click the plugin in Komble → Plugins. Your account, sign-in and backups are in ewe Sync.

## Komble, the app store
keys: komble, app store, store, software center, apps, discover, for you, installed apps, aur
open: komble
Komble installs apps and keeps the system up to date. Its list has three groups. Get apps: Discover searches the Arch repositories, the AUR and AppImages, For you lists apps your other computers have, and AUR builds community packages after you read their PKGBUILD. Your apps: Installed shows what Komble put here, and Updates installs updates (Update all). Desktop: Plugins adds parts of ewe. Settings, at the foot, holds Komble's own options. AUR builds run as you, never as root.

## ewe Sync, your account
keys: ewe-sync, ewe sync, account, nextcloud account, machines, my computers, folders, folder sync, sync app, needs you
open: sync
ewe Sync is the account app, and the only thing in ewe that syncs. Overview: what your account keeps and whether it is up to date, with "Needs you" at the top when something waits for you (a file changed in two places, settings from another computer). Folders: folders to keep in sync; each has one ⋯ menu (Open folder, Sync now, Pause on this computer, Change…, Stop syncing…). Settings & apps: Sync now, Auto-sync and Restore…. Computers: every computer signed in to your account. Account: sign in to your Nextcloud, your storage, sign out. Mail: any IMAP mailbox. Google: optional, with your own OAuth client file. It has no tray icon: the bar shows the account's state (syncing, a conflict, offline), and a click opens ewe Sync.

## Quick settings
keys: quick settings, control center, control centre, quick panel, clock pill, status pill, toggles, tiles, quick settings tabs, quick settings pages
open: quicksettings
Quick settings (Super+N, or click the pill with the clock at the right of the bar) is one page, top to bottom. The header: your picture, name and today's date (a click opens ewe Sync), then Screenshot (with the Screenshot plugin), Settings, Lock and Power. Tiles: Wi-Fi (Network while a cable is in), Bluetooth, Do not disturb, Sharing and the plugins' tiles (VPN, Cast, ewe Safe …); a click switches one, its arrow opens its page. A row of small shortcuts: Calendar, then the plugins' pages (SSH, Mobile, Mail). Volume (its arrow opens the sound outputs) and brightness. The newest notifications, with Show all. At the foot, the battery and the power mode. Scrolling on the pill changes the volume.

Pages open in place of the home page; Esc goes back one page, then closes Quick settings. Power opens a page with Suspend, Sign out, Restart and Shut down. There is no row of tabs down the side any more (it went in ewe 0.39), and the System monitor is not in Quick settings: it is in the top bar.

## The Overview
keys: overview, all windows, mission control, expose, workspace overview, activities
open: overview
Tap Super (alone), swipe up with three fingers, or click the workspace number at the left of the bar. The windows of the workspace spread out so you can pick one; a group of tabs is one card. The workspaces are along the top: point at them and they open into pictures. Click one to go there, drop a window on one to move it there, or drop it beside them for a new workspace. The Overview does not search: the launcher (Super+D) does.

## The launcher
keys: launcher, app launcher, start menu, run app, open app, app menu, search apps
Super+D opens the launcher, which fuzzy-searches apps and files; press it again to close. Clicking an app that is already open jumps to its window, and a middle click starts a fresh copy. Terminal apps open in kitty.

## The everyday apps
keys: terminal, kitty, file manager, files, nemo, text editor, editor, zed, image viewer, pdf, video player, mpv, archive, zip, browser
ewe comes with kitty (terminal, Super+Return), Nemo (files, Super+E), Zed (text editor), imv (pictures), Zathura (PDFs), mpv (video), Engrampa (archives) and Galculator (calculator). Super+B opens the browser chosen in ewe Settings → Default apps, else ewe Browser.

## ewe Browser
keys: ewe browser, browser app, web browser, helium
ewe Browser is an all-in-one browser, file manager, viewer and music player from the ewe project, and ewe's browser since 0.26 (it replaced Helium). Tabs sit in a rail on the left and in workspaces, whose chips are on the bar along the bottom with Bookmarks, Downloads, History, Music, Files and Settings. Every new tab opens Home, and Ctrl+K opens the command palette. ewe Safe fills logins in it, and it syncs through your ewe account.

I drive it through its command line: web searches, a site's own search ("search men's shoes on zara"), "open github.com", "play …" (your music library first), its history, bookmarks, downloads, tabs and workspaces ("go to the YouTube tab", "bookmark this page", "switch to my work workspace in the browser", "go back a page"), and the open page ("what is this page about?"; the page is data to me, never orders). Without ewe Browser, searches and pages open in your default browser.

## Shaun
keys: shaun, companion, who are you, what are you, what can you do, what do you do, assistant, notch, ai, chat, super a, provider, api key, claude, gemini, ollama
I am Shaun, ewe's companion: the notch under the top bar. Super+A or a click opens me. I show what your AI agents are doing and their requests to approve, what needs a look (a full disk, updates, a low battery), what is playing, quick actions and this chat. I can do things right away, with Undo where there is one: "volume 60", "brightness up", "mute", "next song", "dnd on", "game mode on", "wifi off", "bluetooth on", "connect my airpods", "airplane mode on", "keep the screen awake", "set a timer for 5 minutes", "remind me in 10 minutes to call mum", "what is 15% of 80", "take a screenshot", "lock", "open settings", "open nemo", "open kitty in documents", "install obs studio" (I open Komble's search), "search for …", "search men's shoes on zara", "play …", "open github.com", "show my tabs", "bookmark this page", "switch to my work workspace in the browser", "find my resume", "copy my GitHub password" (ewe Safe copies it; I never see it), "lock my safe".

What changes the system I only offer: "dark mode", "suspend", "restart", "shut down", "log out" and "empty the trash" show the exact command with Run, and so does any shell command that does what you ask; type "$ " and a command to run your own the same way. Nothing runs until you press Run or In terminal.

For other questions, choose who answers in Komble → Plugins → Shaun (click it) → Who answers questions: your Claude Code, Codex or OpenCode login, an API key (Anthropic, OpenAI, Gemini, OpenRouter; the key goes to the keyring), a model on this computer (Ollama, LM Studio), or any OpenAI-compatible server (custom, with its address). Without one I still answer from what I know about ewe. I am optional: the Shaun plugin and the ewe-companion package.

## Game mode
keys: game mode, gaming, games, gamemode, steam, performance mode, fps
Say "game mode on" or press the gamepad in my quick actions: Do not disturb turns on, Hyprland's animations and blur turn off, and GameMode's CPU boost holds. "game mode off" puts everything back. Steam and other games install from Komble.

## Welcome, the first-login guide
keys: welcome, first login, setup, first run, onboarding, tour, getting started
Welcome runs once at the first login: get online, updates, sign in to your Nextcloud, restore a backup if the account has one, choose plugins, then a one-minute tour. Every step can be skipped. To see it again: `qs ipc call welcome toggle`.
