# Safe, ewe's password manager

<!-- Topics about Safe. Checked 2026-10-18 against Safe 0.2.0-beta
     (origin/main a3fcc5e: README, docs/PROTOCOL.md, docs/IMPORT.md,
     app/src/lib/screens/Settings.svelte, crates/daemon/src/config.rs),
     the ewe.safe plugin 1.0.0 and ewe.passwords 1.3.0 in ewe 0.29.0-beta,
     and ewe browser 0.3.0-beta. Passkeys and the SSH agent shipped in
     0.2.0-beta. Comments never reach a model. -->

## Safe, ewe's password manager
keys: ewe safe, ewe-safe, my safe, what is safe, safe app, password manager, passwords app, my passwords, master password, keepass, kdbx
Safe keeps your logins, notes, SSH keys and passkeys in one encrypted file in the KeePass format (KDBX 4), on this computer or in your ewe account's Nextcloud. Open the Safe app from the launcher: All items, Logins, Notes, SSH keys, Tags, Folders, Trash and Settings. ewe browser fills from it, Super+P (the Passwords plugin) fills any app, and the Safe plugin (Komble → Plugins) adds a lock in the bar, Quick Access on Super+Shift+P and a Quick settings tile. I can use it too: "copy my GitHub password", "lock my safe", "unlock my safe", "is my safe synced?". Safe copies a password itself and clears it later; I never see it.

The master password is typed only in Safe's own window, never in the chat. Safe locks after 30 minutes without use, when the screen locks and before the computer sleeps: Safe → Settings has Lock after and Lock when the screen locks.

## Copy a password, a username or a code
keys: copy password, copy my password, copy a password, copy username, copy the code, one-time code, 2fa code, otp, verification code, what is my password, show my password, tell me my password
Say "copy my GitHub password", "copy my GitHub username" or "copy my GitHub code". Safe copies it to the clipboard and clears it after a while (30 seconds; Safe → Settings → Clear copied passwords after), and the clipboard history skips it. When several logins fit, I list them and a click copies the one you pick. I never see or say a password, so never paste one into the chat. If Safe is locked, press Unlock Safe and type the master password in Safe's window; the copy happens right after.

## Quick Access
keys: quick access, super+shift+p, safe quick access, safe search, find a login, safe in the bar, safe lock in the bar
With the Safe plugin, Super+Shift+P (or the lock in the bar) opens Quick Access: every item, one search away. Enter copies the password, Ctrl+C the username, Ctrl+O the one-time code, Ctrl+Enter opens the item in Safe, Ctrl+N adds a login and Ctrl+L locks. To type a login into the app in front of you, use Super+P instead.

## Import passwords from 1Password
keys: import from 1password, import passwords, 1pux, move from 1password, leave 1password, export from 1password, bring my passwords, migrate passwords, 1password export
In the Safe app use Import, or in a terminal `ewe-safe import 1pux ~/Downloads/<file>.1pux`. Get the file from the 1Password app: the account menu → Export → 1PUX. A CSV export works too (`ewe-safe import 1password-csv <file>`, logins only), and `ewe-safe import op` reads straight from the 1Password app through its CLI, with no file. Add `--dry-run` to see first what comes in. An export is plain text: delete it when the import is done.

1Password's export leaves passkeys out: sign in to those sites once with the password, then add a new passkey there and save it in Safe. SSH keys that came in as notes become real keys with `ewe-safe ssh adopt`.

## Passkeys
keys: passkey, passkeys, webauthn, fido, sign in without a password, passwordless
Safe keeps passkeys, and ewe browser makes and uses them: on a site that offers one, ewe browser asks "Save a passkey for <site> in Safe?", and next time the passkey is at the top of the list under the login field. They are stored the way KeePassXC stores them, so KeePassXC, KeePassDX and Strongbox can use them too. `ewe-safe passkey list` lists them; browser Settings → ewe → Passkeys turns them off.

## SSH keys in Safe
keys: ssh key, ssh keys, ssh agent, ssh-agent, id_ed25519, 1password ssh agent, git push key, signing key, ssh-add
Safe keeps SSH keys and is an SSH agent for ssh and git, asking once per app the first time it uses a key while Safe is unlocked (no answer in a minute means no). In the Safe app: SSH keys → Turn on. In a terminal: `ewe-safe ssh new --title "GitHub key"` makes one and prints the public key to give the site, `ewe-safe ssh add ~/.ssh/id_ed25519` brings in one you have, and `ewe-safe ssh enable` points new sessions at Safe. `ssh-add -L` then lists the keys through Safe. If ~/.ssh/config names another agent (1Password's), put the IdentityAgent line `enable` prints there: Safe never edits that file. (The SSH tab in Quick settings is the SSH plugin's list of hosts, a different thing.)

## Use Safe from scripts
keys: safe from scripts, safe in scripts, ewe-safe read, ewe-safe run, ewe-safe inject, safe:// reference, secrets in env file, git credential helper, op read, op run
Safe answers scripts the way 1Password's `op` does. A reference names one value: `safe://Personal/GitHub/password`. `ewe-safe read "safe://Personal/GitHub/password"` prints it, `ewe-safe run --env-file .env -- npm start` fills the references in .env for that command only, and `ewe-safe inject -i config.tpl -o config.ini` fills `{{ safe://… }}` in a template (the file it writes is private to you). For git over https: `git config --global credential.helper '!ewe-safe git-credential'`. These print secrets, so I never run them: they are for your own scripts and terminal.

## Safe on your phone
keys: passwords on my phone, passwords on phone, safe on phone, safe on my phone, keepassdx, strongbox, keepassium, android passwords, iphone passwords, phone password manager
Safe's file is a normal KeePass file. When it lives in your ewe account's Nextcloud (`ewe/safe/passwords.kdbx`), open that file on the phone with the same master password: KeePassDX on Android, Strongbox or KeePassium on iPhone. Safe merges what the phone changed at its next sync instead of making a conflicted copy.
