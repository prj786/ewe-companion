# What's new in ewe

<!-- One topic per ewe minor release, newest first; a patch release
     (0.40.1) folds into its minor. Sources, checked 2026-11-21:
     `gh release list -R prj786/ewe` and `gh release view vX.Y.Z-beta`
     (the dates), the pull requests behind each release (prj786/ewe #77,
     #79, #81, #82, #83, #84, #85, #86, #87: their descriptions say what
     changed), the vault's 12-Reference/Version Ledger.md on origin/main (the
     apps' versions in each wave) and the repos themselves for the labels.
     0.46 and 0.47 are not released: they come from the 2026-11-21 wave
     plan (Phase A + L, and Phase S with Shaun 0.3 from ewe-companion#8);
     fix their versions and words at release. scripts/news.sh helps
     refresh this file from the release notes. Keys stay version spellings
     only: the companion picks the topic for the installed version itself.
     Comments never reach a model. -->

## What's new in ewe 0.47
keys: ewe 0.47, version 0.47, 0.47 update
ewe 0.47.0-beta brings Shaun 0.3 (the ewe-companion package 0.3.0-beta and the Shaun plugin 0.6.0). With Claude Code, answers start in about a second and stream in as they are written (Claude Code runs lean and stays warm), and Shaun looks things up before it answers and does up to three things in one reply. With no AI at all it tells the time anywhere, converts units and counts days, reads the battery, memory, CPU and disks, moves windows and switches workspaces, and keeps what you ask it to ("remember that …"). It looks things up offline in the ArchWiki and the manual pages, and in your own notes in ~/Documents/Shaun ("Shaun, learn this: …"). It plays music from your library or a link, says what is playing and why something won't play, switches the keyboard language or adds typing in another one, translates what you copied (with a model), finds a PDF about something, folds the bar or moves the dock when you ask, and tells you what is new after an update.
<!-- from the wave plan 2026-11-21 (Phase S): check the versions and the powers at release -->

## What's new in ewe 0.46
keys: ewe 0.46, version 0.46, 0.46 update
ewe 0.46.0-beta comes with ewe apps 0.2, ewe Settings 0.26, ewe Browser 0.9 and Komble 0.24. ewe Player plays links from YouTube and about a thousand other sites (Ctrl+L, or Play in ewe Player in ewe Browser's right-click menu), and when something does not play it says why in plain words. Komble's Popular has the official Stremio, and Stremio Enhanced gets its streaming server with one click. iPhone photos (HEIC) open in ewe Viewer and ewe Files, subtitles and text files in any encoding read right, and audio and subtitle tracks show their language's own name. ewe Settings → Keyboard and mouse → Typing in other languages adds Chinese, Japanese, Korean, Vietnamese and Indian languages (Ctrl+Space), and the new Language page checks your spelling and offers Firefox, Thunderbird and LibreOffice in your language. Time and place → Formats sets how dates and numbers are written, and the bar's clock follows it (24 hours where your region uses them).
<!-- from the wave plan 2026-11-21 (Phase A + L): check the versions and the labels at release -->

## What's new in ewe 0.45
keys: ewe 0.45, version 0.45, 0.45 update
Released 20 November 2026 as ewe 0.45.0-beta, with ewe Settings 0.25, ewe apps 0.1.1, Komble 0.23.1, ewe Sync 0.17.1, ewe Safe 0.5.1 and the Dock plugin 1.5. A crowded bar can fold: ewe Settings → Layout → Top bar → Fold the extras tucks tray icons, plugin buttons and the tiling switch behind an arrow. The dock can show on the main display, on every display or on none (Komble → Plugins → Dock → Show the dock on). The bar's cards (System monitor, clipboard, Places, Music) look alike, and Quick settings lost its Screenshot button: the bar's camera and the Print key take screenshots. Fullscreen video no longer makes an external monitor blink, and ewe Files, ewe Player and ewe Viewer fit narrow windows.

## What's new in ewe 0.44
keys: ewe 0.44, version 0.44, 0.44 update
Released 19 November 2026 as ewe 0.44.0-beta, with ewe apps 0.1 (ewe Files, ewe Player, ewe Viewer), ewe Settings 0.24, ewe Sync 0.17, Komble 0.23 and ewe Browser 0.8.4. ewe's own ewe Files, ewe Player and ewe Viewer replace Nemo, Engrampa, imv, Zathura and mpv, and the codecs for nearly every video and music format come with ewe. The install is lighter (no NVIDIA driver on Intel or AMD computers), Komble quits when you close it, and ewe Sync uses far less memory in the background. ewe Sync signs in with your Nextcloud name and password, `ewe-settings --page=<page>` opens a Settings page, and Default apps gained Music player, Document viewer and Archives. ewe Browser sends videos to ewe Player.

## What's new in ewe 0.43
keys: ewe 0.43, version 0.43, 0.43 update
Released 19 November 2026 as ewe 0.43.0-beta, with Komble 0.22.1, ewe Browser 0.8.3 and nine updated plugins. A polish pass: apps that open on another workspace (Window rules) and Steam's notifications no longer pull you away, clicking a minimized app brings it back, and Super+B brings the open browser forward. Alt+F4 closes a window, Ctrl+Alt+T opens a terminal and Ctrl+Alt+Delete asks to sign out. The pointer follows ewe Settings → Cursor in every app, the dock, the launcher, the Overview, the desktop and Places got right-click menus, and notifications show their buttons. Right-click a file to compress it, extract an archive (RAR too) or set a picture as wallpaper.

## What's new in ewe 0.42
keys: ewe 0.42, version 0.42, 0.42 update
Released 8 November 2026 as ewe 0.42.0-beta, with ewe Settings 0.23 and Shaun 0.2.4. Chinese, Japanese and Korean text shows instead of boxes (the Source Han Sans font comes with ewe). ewe Settings → Keyboard and mouse → Cursor sets the pointer's size and colour. Game mode does something now: Balanced quiets notifications and turns effects off, and Ultra also raises the power profile and closes ewe's background apps. It is a Quick settings tile, ewe Settings → Power → Gaming and `ewe-game` in a terminal, and it can turn on by itself while a Steam game runs.

## What's new in ewe 0.41
keys: ewe 0.41, version 0.41, 0.41 update
Released 8 November 2026 as ewe 0.41.0-beta, with Komble 0.22, ewe Sync 0.16.2, the System monitor 2.0.1 and VPN 1.0.3. A diet for memory: ewe tunes the computer's compressed swap, keeps the system journal under 1 GB, and switches the keyboard layout for each window far more cheaply. The System monitor no longer counts the graphics cache as used memory, and it reads less often while only the bar shows it. AppImages from Komble start faster (Chrome in half a second instead of 1.2).

## What's new in ewe 0.40
keys: ewe 0.40, version 0.40, 0.40 update, ewe 0.40.1
Released 4 November 2026 as ewe 0.40.0-beta (0.40.1 followed on 5 November), with ewe Safe 0.5, ewe Sync 0.16.1 and ewe Browser 0.8.2. Sharing this computer works from another network with nobody at it: the connection survives a Wi-Fi drop, the terminal reconnects by itself, and a paired computer still works after it restarts (0.40.1). ewe Safe asks for its password once per start of the computer and no longer locks by itself, and its SSH keys ask once per start too. Wi-Fi power saving is off while connected, so connections hold.

## What's new in ewe 0.39
keys: ewe 0.39, version 0.39, 0.39 update
Released 1 November 2026 as ewe 0.39.0-beta, with ewe Settings 0.22, Komble 0.21, ewe Sync 0.16, ewe Safe 0.4, ewe Browser 0.8, ewe Cast 0.13 and the System monitor 2.0. A new look across ewe: Quick settings is one page (header, tiles, shortcuts, sliders, notifications, battery) whose pages open in place, with a Power page. ewe Settings has four groups and Find a setting (Ctrl+F), and Komble shows one row per plugin, with a sheet for its options. ewe Safe became the SSH agent every ssh asks (SSH → Make ewe Safe your SSH agent), and the System monitor moved into the top bar. The apps are named ewe and a word: ewe Settings, ewe Sync, ewe Safe, ewe Browser, ewe Cast.

## What's new in ewe 0.38
keys: ewe 0.38, version 0.38, 0.38 update
Released 1 November 2026 as ewe 0.38.0-beta, with ewe Sync 0.15.1 and ewe Browser 0.7. The network says what is true: the bar shows the connection that carries your traffic and whether there is internet, and the wired row says when a cable waits for an address or the network does not answer, with Reconnect. Share this computer (Quick settings → Sharing) lets your other ewe computers pair once with an 8-digit code, then open a terminal, reach the files and browse the web from it (ewe Browser's remote workspace). Tailscale comes with ewe, set up from the Wi-Fi page, to reach your computers from anywhere. Every computer gets compressed swap in memory (zram).
