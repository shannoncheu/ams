[Setup]
AppId=AMS.GPUI.MusicPlayer
AppName=AMS Music Player
AppVersion=0.1.0-preview.2
DefaultDirName={localappdata}\Programs\AMS Music Player
DefaultGroupName=AMS Music Player
PrivilegesRequired=lowest
OutputDir=..\dist
OutputBaseFilename=MusicPlayer-Setup-x64
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
Compression=lzma2
SolidCompression=yes
UninstallDisplayIcon={app}\GPUI-Music-Player.exe
[Tasks]
Name: desktopicon; Description: "Create a desktop shortcut"; Flags: unchecked
[Files]
Source: "..\dist\GPUI-Music-Player-Windows-x64\*"; DestDir: "{app}"; Excludes: "portable.flag"; Flags: ignoreversion recursesubdirs createallsubdirs
[Icons]
Name: "{group}\AMS Music Player"; Filename: "{app}\GPUI-Music-Player.exe"
Name: "{autodesktop}\AMS Music Player"; Filename: "{app}\GPUI-Music-Player.exe"; Tasks: desktopicon
