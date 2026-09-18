; Inno Setup Script for TNotes Desktop
; Defines 64-bit per-user installer (avoids UAC elevation during self-updates)

#ifndef AppVersion
#define AppVersion "0.2.0-alpha.2"
#endif

[Setup]
AppId={{D37E88A1-80DF-4AE2-9BD1-6F88BD31278C}
AppName=TNotes
AppVersion={#AppVersion}
AppPublisher=Toonion
AppPublisherURL=https://github.com/ToonionOfficial/tnotes
AppSupportURL=https://github.com/ToonionOfficial/tnotes/issues
AppUpdatesURL=https://github.com/ToonionOfficial/tnotes/releases
DefaultDirName={localappdata}\Programs\TNotes
DefaultGroupName=TNotes
OutputDir=..\..\dist
OutputBaseFilename=TNotes-Windows-x64-Setup
SetupIconFile=..\..\apps\desktop\assets\app-icon.ico
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
CloseApplications=yes
RestartApplications=yes
UninstallDisplayIcon={app}\tnotes-desktop.exe

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "..\..\target\release\tnotes-desktop.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\apps\desktop\assets\app-icon.ico"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\TNotes"; Filename: "{app}\tnotes-desktop.exe"; IconFilename: "{app}\app-icon.ico"
Name: "{group}\{cm:UninstallProgram,TNotes}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\TNotes"; Filename: "{app}\tnotes-desktop.exe"; IconFilename: "{app}\app-icon.ico"; Tasks: desktopicon

[Run]
Filename: "{app}\tnotes-desktop.exe"; Description: "{cm:LaunchProgram,TNotes}"; Flags: nowait postinstall skipifsilent
