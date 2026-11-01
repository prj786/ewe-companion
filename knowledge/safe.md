# ewe Safe, ewe's password manager

<!-- Topics about ewe Safe. Checked 2026-11-01 against ewe Safe 0.4.0-beta
     (origin/main 01448b2: docs/GATE.md S21–S28 the SSH gate, app/src
     Sidenav.svelte, Main.svelte's one header, screens/Ssh.svelte and
     Settings.svelte, crates/cli/src/ssh.rs), the ewe.safe plugin 1.1.1
     and ewe.passwords 1.3.1 in ewe 0.39.0-beta, and ewe Browser 0.8
     (settings/passwords.tsx). Earlier: README, PROTOCOL.md, IMPORT.md
     (0.2.0-beta, 2026-10-18). Comments never reach a model. -->

## ewe Safe, ewe's password manager
keys: ewe safe, ewe-safe, my safe, what is safe, safe app, password manager, passwords app, my passwords, master password, keepass, kdbx
ewe Safe keeps your logins, notes, SSH keys, passkeys, cards and addresses in one encrypted file in the KeePass format (KDBX 4), on this computer or in your ewe account's Nextcloud. Open the ewe Safe app from the launcher: its list has All items, Logins, Notes, SSH, Cards, Addresses, your Tags and Folders, Trash and Settings, and one header on top with Search ewe Safe, New and Lock (Ctrl+L). ewe Browser fills from it, Super+P (the Passwords plugin) fills any app, and the ewe Safe plugin (Komble → Plugins) adds a lock in the bar, Quick Access on Super+Shift+P and a Quick settings tile. I can use it too: "copy my GitHub password", "lock my safe", "unlock my safe", "is my safe synced?". ewe Safe copies a password itself and clears it later; I never see it.

The master password is typed only in ewe Safe's own window, never in the chat. ewe Safe locks after 30 minutes without use, when the screen locks and before the computer sleeps: ewe Safe → Settings → Security has Lock after, Lock when the screen locks and Lock before sleep.

## Copy a password, a username or a code
keys: copy password, copy my password, copy a password, copy username, copy the code, one-time code, 2fa code, otp, verification code, what is my password, show my password, tell me my password
Say "copy my GitHub password", "copy my GitHub username" or "copy my GitHub code". ewe Safe copies it to the clipboard and clears it after a while (30 seconds; ewe Safe → Settings → Clear copied passwords after), and the clipboard history skips it. When several logins fit, I list them and a click copies the one you pick. I never see or say a password, so never paste one into the chat. If ewe Safe is locked, press Unlock ewe Safe and type the master password in ewe Safe's window; the copy happens right after.

## Quick Access
keys: quick access, super+shift+p, safe quick access, safe search, find a login, safe in the bar, safe lock in the bar
With the ewe Safe plugin, Super+Shift+P (or the lock in the bar) opens Quick Access: every item, one search away. Enter copies the password, Ctrl+C the username, Ctrl+O the one-time code, Ctrl+Enter opens the item in ewe Safe, Ctrl+N adds a login and Ctrl+L locks. To type a login into the app in front of you, use Super+P instead.

## Import passwords from 1Password
keys: import from 1password, import passwords, 1pux, move from 1password, leave 1password, export from 1password, bring my passwords, migrate passwords, 1password export
In the ewe Safe app: Settings → Import and export → Import…, or in a terminal `ewe-safe import 1pux ~/Downloads/<file>.1pux`. Get the file from the 1Password app: the account menu → Export → 1PUX. A CSV export works too (`ewe-safe import 1password-csv <file>`, logins only), and `ewe-safe import op` reads straight from the 1Password app through its CLI, with no file. Add `--dry-run` to see first what comes in. An export is plain text: delete it when the import is done.

1Password's export leaves passkeys out: sign in to those sites once with the password, then add a new passkey there and save it in ewe Safe. SSH keys that came in as notes become real keys with `ewe-safe ssh adopt`.

## Passkeys
keys: passkey, passkeys, webauthn, fido, sign in without a password, passwordless
ewe Safe keeps passkeys, and ewe Browser makes and uses them: on a site that offers one, ewe Browser asks "Save a passkey for <site> in ewe Safe?", and next time the passkey is at the top of the list under the login field. They are stored the way KeePassXC stores them, so KeePassXC, KeePassDX and Strongbox can use them too. `ewe-safe passkey list` lists them; ewe Browser's Settings → Passwords → Passkeys turns them off.

## SSH keys in ewe Safe
keys: ssh key, ssh agent, id_ed25519, 1password ssh agent, git push key, ssh-add, move ssh, ssh gate, ssh takeover
ewe Safe is ewe's SSH gate: it keeps your SSH keys, and every ssh and git on this computer asks it for them, even for a host whose own line names another agent (1Password's). In the ewe Safe app, open SSH: "Make ewe Safe your SSH agent" → Turn on. It puts two lines of its own at the top of ~/.ssh/config (an Include of ~/.ssh/ewe-safe/config) and changes nothing else there; turned on once, it follows your ewe account to your other computers. "Ask before a key is used" says when it asks: Once per app (the default), Every time or Never, while ewe Safe is unlocked.

If 1Password or key files in ~/.ssh still answer ssh, the SSH page says so: Move SSH to ewe Safe shows what it will do, then brings the key files in (it never deletes or changes them), stops your ewe hosts naming another agent, and lists the keys only 1Password has: import those first (Settings → Import and export → Import…, or `ewe-safe import op`), then `ewe-safe ssh adopt`. In a terminal: `ewe-safe ssh enable`, `ewe-safe ssh takeover --dry-run`, `ewe-safe ssh new --title "GitHub key"`, `ewe-safe ssh add ~/.ssh/id_ed25519`, and `ewe-safe ssh disable` (which takes its two lines out again). `ssh-add -L` lists the keys through ewe Safe. (The SSH page in Quick settings is the SSH plugin's list of hosts, a different thing.)

## Connect to GitHub, GitLab or Azure with ewe Safe
keys: connect my github, connect to github, connect github, github ssh, gitlab ssh, azure devops, bitbucket ssh, codeberg ssh, ssh key for github, add ssh key to github, server ssh key, git commit, sign commits, commit signing, signing key, verified commits
In the ewe Safe app: SSH → Connect…, then pick the place: GitHub, GitLab, Azure DevOps, Bitbucket, Codeberg, Gitea or Forgejo, an Azure, AWS, Google Cloud, DigitalOcean or Hetzner virtual machine, or a server you can log in to. ewe Safe makes a new key (or uses one of yours), copies its public key and opens the site's page where you paste it; then Check says whether it works ("Connected to GitHub as …"). It also adds the host to your ewe settings, so `git clone git@github.com:…` just works. For a server, type its address and press Open a terminal, which puts the key there with ssh-copy-id. In a terminal: `ewe-safe ssh connect github`, then `ewe-safe ssh test github.com`.

ewe Safe can sign your Git commits with an SSH key, as 1Password does: tick "Also sign my Git commits with this key" when you connect GitHub or GitLab, or open a key's ⋯ menu on the SSH page → Sign Git commits with this key (Stop signing commits turns it off). It uses your Git email (`git config --global user.email`) and asks for one when Git has none. In a terminal: `ewe-safe ssh sign-commits <key id>` and `ewe-safe ssh sign-commits --off`. On GitHub, add the same key once more as a Signing Key.

## Use ewe Safe from scripts
keys: safe from scripts, safe in scripts, ewe-safe read, ewe-safe run, ewe-safe inject, safe:// reference, secrets in env file, git credential helper, op read, op run
ewe Safe answers scripts the way 1Password's `op` does. A reference names one value: `safe://Personal/GitHub/password`. `ewe-safe read "safe://Personal/GitHub/password"` prints it, `ewe-safe run --env-file .env -- npm start` fills the references in .env for that command only, and `ewe-safe inject -i config.tpl -o config.ini` fills `{{ safe://… }}` in a template (the file it writes is private to you). For git over https: `git config --global credential.helper '!ewe-safe git-credential'`. These print secrets, so I never run them: they are for your own scripts and terminal.

## ewe Safe on your phone
keys: passwords on my phone, passwords on phone, safe on phone, safe on my phone, keepassdx, strongbox, keepassium, android passwords, iphone passwords, phone password manager
ewe Safe's file is a normal KeePass file. When it lives in your ewe account's Nextcloud (`ewe/safe/passwords.kdbx`), open that file on the phone with the same master password: KeePassDX on Android, Strongbox or KeePassium on iPhone. ewe Safe merges what the phone changed at its next sync instead of making a conflicted copy.
