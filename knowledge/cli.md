# ewe's command-line tools

<!-- One topic per tool: what it is for, the common commands, what it
     prints. Checked 2026-10-18 against the source, not the website (the
     source wins where they differ): ewe 0.29.0-beta (origin/main d0f84de:
     bin/ewe-conf, ewe-theme, ewe-plugin, ewe-bt, ewe-cloud, ewe-files,
     ewe-drive, ewe-mail, ewe-caldav, ewe-auth, ewe-globalshortcuts,
     packaging/ewe-setup, install.sh, system/bin/ewe-share-picker,
     dotfiles/quickshell IpcHandlers, plugins/*/manifest.json and their
     IpcHandlers, plugins/ewe.passwords/ewe-pass 1.3.0), ewe-os
     iso/airootfs/usr/local/bin/ewe-install, Safe 0.2.0-beta (docs/PROTOCOL.md,
     README), ewe browser 0.3.0-beta (docs/CLI.md, cli/tools.ts), Komble
     0.20.2-beta (src-tauri/src/lib.rs), ewe-sync 0.14.4-beta (lib.rs),
     ewe-settings 0.18.0-beta (takes no arguments), ewe-cast 0.12.3-beta
     (PKGBUILD: only ewe-castd), ewe-companion as of wave 2 (crates/cli/src/verbs.rs).
     Rule 5 holds for most; the exceptions are named where they bite.
     Comments never reach a model. -->

## The ewe command-line tools
keys: command line, terminal commands, cli, command-line tools, command line tools, scripting, script ewe, json output, ewe commands, list of commands
Most ewe tools print one JSON object and exit 0, so anything you click can be scripted. ewe-conf reads and changes settings, ewe-theme the colours and schemes, ewe-plugin the plugins, ewe-bt Bluetooth, ewe-cloud, ewe-files and ewe-drive your account and its folders, ewe-mail and ewe-caldav mail and the calendar, ewe-auth the optional Google sign-in, ewe-globalshortcuts apps' global shortcuts, ewe-safe Safe, ewe-browser ewe browser, komble opens the store, ewe-companion talks to me, and ewe-setup refreshes ewe in your home. The bar and panels answer `qs ipc call <target> <function>`. Ask me about any of them by name.

A few talk to a person instead: `ewe-conf get` prints the bare value and `ewe-conf set` prints nothing, `ewe-plugin` prints text unless you add `--json`, and Safe's script commands (`read`, `run`, `inject`) print what they fetch, so I never run those. ewe-share-picker is the window a screen-sharing app opens to pick a screen, a window or a region; nobody runs it by hand.

## ewe-conf, the settings file's tool
keys: ewe-conf, ewe conf, conf get, conf set, set a setting from the terminal, change a setting from the terminal, ewe.conf command, dotted key
`ewe-conf get <key>` reads a setting (text and numbers come back bare, lists and tables as JSON), `ewe-conf set <key> <value>` changes it and applies it at once (the value is read as JSON, else as text), `ewe-conf unset <key>` removes it, `ewe-conf dump` prints the whole file as JSON and `ewe-conf apply` rebuilds the generated files from it. It is the only thing that writes ~/.config/ewe/ewe.conf: never edit that file or ~/.config/hypr/generated/ by hand. Keys are dotted, like `desktop.theme.accent`; a part with a dot in it is quoted: `ewe-conf set 'plugins.settings."ewe.passwords".provider' ewe`.

With sync on, every `set` uploads the file within about 30 seconds. `ewe-conf sync-status` says why sync waits (remote-newer, remote-exists), `ewe-conf push` uploads and `ewe-conf pull` downloads. A value a setting does not take is refused with an error, and nothing changes.

## ewe-theme, schemes from the terminal
keys: ewe-theme, ewe theme, scheme apply, scheme list, scheme import, import a scheme, base24, base16, theme from the terminal, from-wallpaper
`ewe-theme scheme list` shows the schemes and which one is on; `ewe-theme scheme apply ewe-light` (or `ewe-dark`, or one of your own) switches the whole desktop; `ewe-theme scheme import <file or url>` adds a Base16 or Base24 scheme (`--apply` uses it at once); `ewe-theme scheme from-wallpaper --apply` makes one from your wallpaper (`--light` for a light one); `ewe-theme show` prints every look value. Light or dark is the scheme's own kind: there is no separate mode switch. It changes ewe.conf through ewe-conf, like Settings → Appearance.

## ewe-plugin, plugins from the terminal
keys: ewe-plugin, ewe plugin, plugin command, plugins from the terminal, ewe-plugin install, ewe-plugin list
`ewe-plugin list` shows what is installed and on (`--json` adds the plugins from ewe you can still install), `ewe-plugin install ewe.dock` installs one from ewe, `ewe-plugin add <git url>` someone else's (plugins run unsandboxed: only from people you trust), `ewe-plugin enable|disable|remove <id>` (each restarts the shell), `ewe-plugin get <id>` and `ewe-plugin set <id> <key> <value>` read and change a plugin's options, and `ewe-plugin bar <id> on|off` shows it in the bar. It prints text for a person unless you add `--json`; install, get, set and bar always answer JSON. Plugins live in ~/.config/ewe/plugins/<id>/.

## ewe-bt, Bluetooth from the terminal
keys: ewe-bt, ewe bt, bluetooth from the terminal, bluetooth command, bluetoothctl
`ewe-bt status` lists the adapter and the devices (name, address, paired, connected, battery); `ewe-bt power on|off`; `ewe-bt scan 20` looks for devices for 20 seconds; `ewe-bt pair <address>`, `ewe-bt connect <address>`, `ewe-bt disconnect <address>`, `ewe-bt forget <address>` and `ewe-bt trust <address> on|off`. It takes addresses like AA:BB:CC:DD:EE:FF, never names: `status` gives them. It answers JSON, and an error is a sentence ("No Bluetooth adapter found", "bluetooth.service is not running"). ewe uses it instead of bluetoothctl, whose own pairing helper would take over ewe's pairing prompts. I do the everyday ones for you: "bluetooth off", "connect my airpods".

## ewe-cloud, ewe-files and ewe-drive: your account and its folders
keys: ewe-cloud, ewe cloud, ewe-files, ewe files, ewe-drive, ewe drive, nextcloud folder, google drive folder, mount nextcloud, rclone
`ewe-cloud status` says whether you are signed in to your Nextcloud (server, user, storage, the keyring), `ewe-cloud login <server>` signs in through your browser and `ewe-cloud logout` signs out. `ewe-cloud token` prints the app password: a secret, keep it out of chats and logs. `ewe-files status|setup|mount|unmount|forget` puts your Nextcloud at ~/Nextcloud, and `ewe-drive status|setup|mount|unmount` does the same for Google Drive at ~/Google Drive (with the optional Google sign-in). ewe-sync does all of this with buttons.

## ewe-mail and ewe-caldav: mail and the calendar
keys: ewe-mail, ewe mail, ewe-caldav, ewe caldav, mail from the terminal, calendar from the terminal, unread mail command, calendar events command
`ewe-mail status` shows the mailbox it checks, `ewe-mail unseen` the unread messages (sender, subject, date), `ewe-mail login <host> <user>` reads the password on stdin and keeps it in the keyring, and `ewe-mail logout` forgets it. An "auth-failed" error usually means the provider wants an app password. `ewe-caldav events --days 7` lists your Nextcloud calendar's events (what Quick settings → Calendar shows) and `ewe-caldav calendars` the calendars; without a connection it answers from its cache.

## ewe-auth, the optional Google sign-in
keys: ewe-auth, ewe auth, google sign in, google oauth, oauth client, oauth client file, sign in with google
Google is optional in ewe, and no Google app ships: it needs your own OAuth client file at ~/.config/ewe/oauth-client.json. `ewe-auth status` says whether it is set up and signed in, `ewe-auth login` signs in (`--with-mail` and `--with-drive` add Gmail and Drive) and `ewe-auth logout` signs out; the token stays in the keyring. `ewe-auth keyring-reset` moves a broken keyring aside: log out and back in, then sign in again from ewe-sync.

## ewe-globalshortcuts, apps' global shortcuts
keys: ewe-globalshortcuts, global shortcuts, global shortcut, push to talk shortcut, portal shortcut, app shortcut not working
Apps that ask the desktop for a global shortcut (Discord's push to talk, 1Password's Quick Access) get it bound when the combo is free. `ewe-globalshortcuts list` shows what was bound and what was skipped because ewe already uses that combo, `ewe-globalshortcuts sync` binds them again, and `reset` forgets them.

## ewe-sync from the terminal
keys: ewe-sync command, ewe-sync --check, ewe-sync check, sync from the terminal, sync command
ewe-sync is an app, not a command-line tool: `ewe-sync` opens it, `ewe-sync --hidden` starts it without a window, and `ewe-sync --check` prints the account and sync state as JSON (it exits 1 when something is wrong). The settings sync itself is `ewe-conf sync-status`, `ewe-conf push` and `ewe-conf pull`.

## ewe-setup, install.sh and the installer
keys: ewe-setup, ewe setup, install.sh, ewe-install, installer, reinstall ewe, set up ewe again, install script
`ewe-setup` puts ewe's files in your home again; run it as yourself, never with sudo, when the desktop still looks old after an update and a fresh login. `bash /usr/share/ewe/install.sh --no-packages`, also as yourself (it asks for sudo where it needs it), sets up the system side again: the login screen, the services, the screen-sharing portal; `--check-only` only checks. On the install USB, the Install ewe app (or `ewe-install` in a terminal) installs Arch and ewe; an installed system does not have it.

## ewe-safe, Safe's command line
keys: ewe-safe command, ewe-safe commands, safe command line, safe cli, ewe-safe status, ewe-safe lock
`ewe-safe status` says whether Safe is set up and locked, how many items and how it syncs; `ewe-safe unlock --ask` opens Safe's own prompt; `ewe-safe lock`; `ewe-safe search <words>` and `ewe-safe list` find items; `ewe-safe copy <id> --field password` puts a value on the clipboard and clears it later; `ewe-safe generate --length 24` makes a password; `ewe-safe import 1pux <file>` brings in a 1Password export; `ewe-safe ssh …` runs the SSH agent; `ewe-safe passkey list` lists passkeys. Passwords are typed into Safe's prompt or piped in, never put on the command line. It answers JSON, except the script commands (`read`, `run`, `inject`, `git-credential`), which work like 1Password's `op`. I only ask Safe to copy, lock, unlock and count: nothing that prints a password or a login's name goes through me.

## ewe-browser, the browser's command line
keys: ewe-browser command, ewe-browser cli, browser command line, browser from the terminal, browser cli
`ewe-browser tabs`, `history <words>`, `bookmarks`, `downloads`, `music <words>`, `find <words>` and `page --text` read what the browser knows (most of them with the browser closed too); `ewe-browser open <url or folder>`, `search <words>`, `play <song>`, `home`, `focus` and `tab activate|reload|mute|pin <id>`, `tab reopen` and `bookmark add <url>` act. From ewe browser 0.4: `search <words> --site zara` (a site's own search), `workspaces`, `workspace activate|next|prev|create`, `back`, `forward`, `download open|show <id>` and `open … --workspace <name>`. Every one prints JSON and exits 0; acting ones say how to undo. `ewe-browser open app://history/` (or settings, music, files) opens the browser's own pages. This is how I drive the browser.

## komble from the terminal
keys: komble command, komble --search, komble flags, komble cli, open komble from the terminal, komble updates command
`komble` opens Komble; `komble --updates`, `komble --plugins`, `komble --settings`, `komble --search <words>` and `komble --options=<plugin id>` open those pages (a Komble already open switches page). `komble ~/Downloads/app.AppImage`, or a `.pkg.tar.zst`, opens its install page. It has no JSON commands: installing and updating happen in its window, after you confirm. Updating from a terminal is `sudo pacman -Syu`.

## Settings from the terminal
keys: ewe-settings command, ewe-settings, settings from the terminal, open a settings page from the terminal, hypr-settings
`ewe-settings` opens Settings, like Super+,; it takes no options and always opens on Appearance, so pick the page in the sidebar. To change a setting from a terminal, use `ewe-conf set <key> <value>`.

## ewe-companion, me from the terminal
keys: ewe-companion, ewe companion, companion command, shaun from the terminal, ewe-companion ask, ewe-companion say, ewe-companiond
`ewe-companion ask "volume 60"` does what the chat does (a command runs at once; a question goes to whoever answers), `ewe-companion say "Backup done" --level success` puts a line on the notch, and `ewe-companion status`, `history`, `sessions` and `find <words>` read. `ewe-companion key set anthropic < keyfile` reads an API key piped on stdin (never on the command line, where it would land in your shell history) and keeps it in the keyring, and `ewe-companion agents install claude` hooks up Claude Code's cards (`agents doctor` checks it). It needs the ewe-companion package; its background service is ewe-companiond. JSON, exit 0.

## The shell's own commands (qs ipc)
keys: qs ipc, qs ipc call, ipc, quickshell ipc, ipc target, shell command, keybind command, qs ipc show
The bar, the launcher and the panels answer `qs ipc call <target> <function>`: `applauncher toggle` (the launcher), `overview toggle`, `quicksettings toggle`, `quicksettings tab wifi` (also bt, audio, cal, notifs and the plugins' pages), `quicksettings powerdialog` (the shut-down question), `settings toggle`, `lock lock`, `bar toggle`, `widgets arrange`, `display reset` (every screen back on), `saver toggle`, `store toggle` (Komble), `welcome toggle`. `qs ipc show` lists what answers right now. Use `toggle` rather than `show`: `qs ipc call <target> show` lists targets instead.

Plugins add their own while they are on: `ewe.insomnia on|off|status`, `ewe.screenshot shoot full|region|window`, `ewe.passwords toggle`, `ewe.safe toggle|lock|unlock|status`, `ewe.cast toggle`, `ewe.clipboard toggle`, `ewe.places toggle`, `ewe.mail refresh` and `ewe.companion toggle`. Your own keys can run any of these: `ewe-conf set desktop.keybinds '[{"combo": "SUPER + U", "exec": "qs ipc call ewe.insomnia toggle"}]'`.

## ewe-pass, the Passwords plugin's tool
keys: ewe-pass, ewe pass, passwords plugin command, password fill command
ewe-pass is what Super+P runs; it comes with the Passwords plugin and lives in ~/.config/ewe/plugins/ewe.passwords/, not on your PATH. `./ewe-pass status` says which password manager it uses and why, `./ewe-pass providers` which ones are installed (Safe, 1Password, Bitwarden, pass), and `./ewe-pass use-safe` switches it to Safe. It types a login into a window or copies it, and it never prints a password.
