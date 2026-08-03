#define MyAppName "fehrust"
#define MyAppPublisher "rgathub"
#define MyAppURL "https://github.com/rgathub/fehrust"
#define MyAppExeName "fehrust.exe"

#ifndef MyAppVersion
  #define MyAppVersion "0.0.0"
#endif

[Setup]
AppId={{B9F5B37C-8E04-4C0B-9BD5-7E50D80A8F2B}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}/releases
DefaultDirName={localappdata}\Programs\{#MyAppName}
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir=.
OutputBaseFilename=fehrust-{#MyAppVersion}-windows-x86_64-setup
Compression=lzma
SolidCompression=yes
PrivilegesRequired=lowest
UninstallDisplayIcon={app}\{#MyAppExeName}
WizardStyle=modern

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional icons:"

[Files]
Source: "..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\README.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Registry]
Root: HKCU; Subkey: "Software\Classes\Applications\{#MyAppExeName}\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\Applications\{#MyAppExeName}\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\{#MyAppExeName},0"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\.jpg\OpenWithList\{#MyAppExeName}"; ValueType: string; ValueName: ""; ValueData: ""; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.jpeg\OpenWithList\{#MyAppExeName}"; ValueType: string; ValueName: ""; ValueData: ""; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.png\OpenWithList\{#MyAppExeName}"; ValueType: string; ValueName: ""; ValueData: ""; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.bmp\OpenWithList\{#MyAppExeName}"; ValueType: string; ValueName: ""; ValueData: ""; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.gif\OpenWithList\{#MyAppExeName}"; ValueType: string; ValueName: ""; ValueData: ""; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.tif\OpenWithList\{#MyAppExeName}"; ValueType: string; ValueName: ""; ValueData: ""; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.tiff\OpenWithList\{#MyAppExeName}"; ValueType: string; ValueName: ""; ValueData: ""; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.webp\OpenWithList\{#MyAppExeName}"; ValueType: string; ValueName: ""; ValueData: ""; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.heic\OpenWithList\{#MyAppExeName}"; ValueType: string; ValueName: ""; ValueData: ""; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.heif\OpenWithList\{#MyAppExeName}"; ValueType: string; ValueName: ""; ValueData: ""; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.avif\OpenWithList\{#MyAppExeName}"; ValueType: string; ValueName: ""; ValueData: ""; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.ico\OpenWithList\{#MyAppExeName}"; ValueType: string; ValueName: ""; ValueData: ""; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.svg\OpenWithList\{#MyAppExeName}"; ValueType: string; ValueName: ""; ValueData: ""; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.cr2\OpenWithList\{#MyAppExeName}"; ValueType: string; ValueName: ""; ValueData: ""; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.nef\OpenWithList\{#MyAppExeName}"; ValueType: string; ValueName: ""; ValueData: ""; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.arw\OpenWithList\{#MyAppExeName}"; ValueType: string; ValueName: ""; ValueData: ""; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.dng\OpenWithList\{#MyAppExeName}"; ValueType: string; ValueName: ""; ValueData: ""; Flags: uninsdeletevalue
