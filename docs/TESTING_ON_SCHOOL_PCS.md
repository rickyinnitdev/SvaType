# Testing SvaType on school PCs

Generated lessons, vocabulary, examples and installer translations are unreviewed.
A Khmer teacher must review them before classroom rollout.

## What to copy to USB

`npm run build` creates these files in `target/packages`:

- `SvaType-Setup-x64.exe`: small, per-user installer for PCs with WebView2.
- `SvaType-Setup-Full-x64.exe`: includes Microsoft's offline WebView2 installer.
- `SvaType-Portable-x64.zip`: extract completely; needs an existing WebView2 runtime.
- `SHA256SUMS.txt`: SHA-256 hashes to check after copying to USB.

Build tools may download dependencies on the development PC. Installation and
normal use of these packaged files must work with the network disconnected.
There is no updater, sign-in, remote font, analytics or runtime HTTP client.

## Clean Windows 10 / Windows 11 acceptance

Use a disposable Windows 10 22H2 x64 VM and a Windows 11 x64 machine. In the VM,
disable its network adapter before installation. Confirm no existing WebView2
runtime for the Full installer test. Keep a snapshot before each test.

1. As a standard user, run the Full installer from USB. It must install without
   needing internet. Check the bilingual instructions, Start-menu entry and
   optional desktop shortcut. Launch the app and create two profiles.
2. Visit Learning Home, all paths, Keyboard Intro, a lesson of each step type,
   Vocabulary, Progress, all five Typing Mode screens, History, Result and Settings.
   Later milestones add Games, Teacher and Help; repeat the whole checklist then.
3. Finish a lesson, word exercise and timed test. Close/reopen the app. Confirm
   profiles, stars, words learned, texts, History and preferences survive.
4. During a long test, type for more than 30 seconds, terminate only SvaType in
   Task Manager, restart, resume and finish. At most the last 30 seconds may be
   lost. Prior aggregate accuracy and errors must remain.
5. Export a backup to USB. Restore on the other machine. Confirm History and
   profiles; verify a pre-restore copy exists. Open the UTF-8 CSV in Excel and
   check Khmer text. Do not restore into a real class database during testing.
6. Install a newer package over the existing one. Existing progress must remain,
   and a pre-migration backup must appear before any schema upgrade.
7. Uninstall with **Delete my data unchecked** (the default). Reinstall and confirm
   all data remains. Only in the disposable VM, repeat with the checkbox checked
   and confirm the current user's default data is removed. Shared ProgramData and
   portable data are never deleted by a per-user installer.
8. Test the slim installer and portable ZIP on a machine with WebView2. On a VM
   without WebView2, the slim app must show a native bilingual message directing
   the user to the Full installer; it must not try to download a runtime.
9. Launch twice. Only one app remains; the existing window is brought forward.
   Check browser refresh/print/find shortcuts and the default right-click menu
   cannot expose browser UI. Ctrl+Shift+F9 must still toggle diagnostics.

## Khmer keyboard and rendering

Compare the app's text against Notepad/Word using Windows Khmer NiDA, then legacy
Khmer. Inspect subscripts, coeng-ro, ៉, ៊, ុំ and ោះ at every font size, with
light, dark and high-contrast themes. No glyph may clip vertically.

Use Settings > Keyboard > Check when available to compare the app's layout table
against Windows-produced text for every physical key and Shift/AltGr combination.
A school keyboard with a different printed layout must use the correct profile
layout. Remember that NiDA base Space is ZWSP; Shift+Space gives a visible space.
Normal exercise targets omit ZWSP by design.

## Performance

Use the actual Sandy Bridge Core i3, 8 GB RAM and HD 2000/3000 graphics. Enable
Low-end mode. Sum the app and WebView2 **private working sets**, as shown in the
Task Manager Memory column; target <=150 MB. Record the full working-set sum
separately because it double-counts shared pages. Idle CPU should round to 0%.
Measure cold start after reboot: <3 seconds SSD / <5 seconds HDD.

Type a long paragraph and a 10-minute test, observe Ctrl+Shift+F9, and check every
key stays responsive. Minimize/unfocus during a timer or game: it must pause.
Record the Windows build, WebView2 version, CPU, storage, RAM and result. See
`docs/PERFORMANCE.md` for development measurements and CPU-throttle instructions.

## Data locations and USB updates

Default: `%LOCALAPPDATA%\SvaType\svatype.db`.
Portable: `data\svatype.db` beside the EXE, selected by `portable.txt`.
Shared: `%PROGRAMDATA%\SvaType\svatype.db`, selected by `shared.txt` there; an
administrator must grant the classroom account write permission in advance.

Always close SvaType before copying a data folder. Prefer Settings > Backup,
copy the `.svabak` file to USB, and Restore on the destination PC. Keep the
pre-restore copy until the restored data has been checked. Never replace a live
SQLite file or delete migration tables.

To update from USB, make a backup, close the app and run the newer installer.
For portable mode, keep `data` and `portable.txt`; replace only the EXE/content
files. There is no automatic update check. Unsigned packages may show Windows
SmartScreen: verify the package source and hash, then choose **More info > Run
anyway** if school policy permits. Do not disable Windows security protections.

## Khmer instructions

ដាក់កម្មវិធីដំឡើង Full លើ USB ហើយសាកល្បងដំឡើងនៅពេលផ្ដាច់អ៊ីនធឺណិត។
បង្កើតឈ្មោះអ្នករៀនពីរនាក់ ហាត់វាយ បិទកម្មវិធី ហើយបើកឡើងវិញ។
ពិនិត្យថាប្រវត្តិ ផ្កាយ និងវឌ្ឍនភាពនៅដដែល។ បម្រុងទុកមុនដំឡើងកំណែថ្មី។
ពេលលុបកម្មវិធី កុំធីកប្រអប់លុបទិន្នន័យ បើចង់រក្សាប្រវត្តិនិងវឌ្ឍនភាព។
សូមឱ្យគ្រូពិនិត្យមេរៀន ពាក្យ និងឧទាហរណ៍មុនប្រើក្នុងថ្នាក់។

## Verification status

Development tests run on Windows 11. A clean Windows 10 VM with its adapter
disabled is not available in this workspace. Do not mark that acceptance step
passed until an operator runs it. Development process-scoped host/proxy blocking
is useful evidence, but it does not replace the disconnected-VM installation test.

## Final development regression (M11)

Windows 11 native integration checks passed for all five Typing Mode activities,
the entire 20-lesson Grade 3 path, all four games, Journey stop-five resume,
blur/hidden pause, 30 fps Low-end cap, and the teacher workflow with 40 profiles.
Polish checks cover complete local OFL text, Windows layout match/mismatch,
contrast/XL, Grade 3 target size, bookmarks, unfinished reading History, path
selection persistence and Khmer Settings at 1024 x 640. A native Alt+F4/restart
check retained the unfinished lesson. Tests use isolated disposable data folders.

The automated layout comparison test supplies synthetic Windows input. It does
not replace pressing every physical key with NiDA active on a school keyboard.
Similarly, generated Khmer content remains marked unreviewed until a teacher
checks the language, examples, order and classroom suitability.
