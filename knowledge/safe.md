# Safe, ewe's password manager

<!-- Topics about Safe. Checked against Safe 0.1.0-beta, ewe 0.27.0-beta,
     ewe.safe 1.0.0 and ewe.passwords 1.2.0 (2026-10-15), and ewe-safe's
     docs/TRIO.md for what is being built: passkeys, SSH keys and Safe
     inside ewe browser are "coming" until they are released. Comments never
     reach a model. -->

## Safe, ewe's password manager
keys: ewe safe, ewe-safe, my safe, what is safe, safe app, password manager, passwords app, my passwords, master password, keepass, kdbx
Safe keeps your logins and notes in one encrypted file in the KeePass format (KDBX 4), on this computer or in your ewe account's Nextcloud. Open the Safe app from the launcher. With the Safe plugin (Komble → Plugins), the lock in the bar shows whether it is locked, Super+Shift+P opens Quick Access, and Super+P fills a login into the window in front. I can use it too: "copy my GitHub password", "lock my safe", "unlock my safe", "is my safe synced?". Safe copies a password itself and clears it later; I never see it.

The master password is typed only in Safe's own window, never in the chat.

## Copy a password, a username or a code
keys: copy password, copy my password, copy a password, copy username, copy the code, one-time code, 2fa code, otp, verification code, what is my password, show my password, tell me my password
Say "copy my GitHub password", "copy my GitHub username" or "copy my GitHub code". Safe copies it to the clipboard and clears it after a while (30 seconds unless you changed it), and the clipboard history skips it. When several logins fit, I list them and a click copies the one you pick. I never see or say a password, so never paste one into the chat. If Safe is locked, press Unlock Safe and type the master password in Safe's window; the copy happens right after.

## Import passwords from 1Password
keys: import from 1password, import passwords, 1pux, move from 1password, leave 1password, export from 1password, bring my passwords, migrate passwords, 1password export
In the Safe app use Import, or in a terminal `ewe-safe import 1pux ~/Downloads/<file>.1pux`. Get the file from the 1Password app: the account menu → Export → 1PUX. A CSV export works too (`ewe-safe import 1password-csv <file>`, logins only), and `ewe-safe import op` reads straight from the 1Password app through its CLI, with no file. Add `--dry-run` to see first what comes in. An export is plain text: delete it when the import is done.

1Password's export leaves passkeys out. Sign in to those sites once with the password; passkeys in Safe are coming.

## Passkeys
keys: passkey, passkeys, webauthn, fido, sign in without a password, passwordless
Coming: Safe will keep passkeys, and ewe browser will sign in with them after a click. They are stored the way KeePassXC stores them, so KeePassXC and KeePassDX can use them too. 1Password's export leaves passkeys out, so after moving, sign in once with the password and add a new passkey on each site when Safe can make one.

## SSH keys in Safe
keys: ssh key, ssh keys, ssh agent, ssh-agent, id_ed25519, 1password ssh agent, git push key, signing key
Coming: Safe will keep SSH keys and answer ssh and git as an SSH agent, asking once per app the first time a key is used, as 1Password does. The keys stay in the KeeAgent format KeePassXC reads. Until then keep the agent you use now. (The SSH tab in Quick settings is the SSH plugin's list of hosts, a different thing.)

## Safe on your phone
keys: passwords on my phone, passwords on phone, safe on phone, safe on my phone, keepassdx, strongbox, keepassium, android passwords, iphone passwords, phone password manager
Safe's file is a normal KeePass file. When it lives in your ewe account's Nextcloud (`ewe/safe/passwords.kdbx`), open that file on the phone with the same master password: KeePassDX on Android, Strongbox or KeePassium on iPhone. Safe merges what the phone changed at its next sync instead of making a conflicted copy.
