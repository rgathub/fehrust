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
ChangesEnvironment=yes
UninstallDisplayIcon={app}\{#MyAppExeName}
WizardStyle=modern

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Files]
Source: "..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\CHANGELOG.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"

[Registry]
Root: HKCU; Subkey: "Software\Classes\Applications\{#MyAppExeName}\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\Applications\{#MyAppExeName}\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\{#MyAppExeName},0"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\.jpg\OpenWithList\{#MyAppExeName}"; ValueType: string; ValueName: ""; ValueData: ""; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\.jpeg\OpenWithList\{#MyAppExeName}"; ValueType: string; ValueName: ""; ValueData: ""; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\.png\OpenWithList\{#MyAppExeName}"; ValueType: string; ValueName: ""; ValueData: ""; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\.bmp\OpenWithList\{#MyAppExeName}"; ValueType: string; ValueName: ""; ValueData: ""; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\.gif\OpenWithList\{#MyAppExeName}"; ValueType: string; ValueName: ""; ValueData: ""; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\.tif\OpenWithList\{#MyAppExeName}"; ValueType: string; ValueName: ""; ValueData: ""; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\.tiff\OpenWithList\{#MyAppExeName}"; ValueType: string; ValueName: ""; ValueData: ""; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\.webp\OpenWithList\{#MyAppExeName}"; ValueType: string; ValueName: ""; ValueData: ""; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\.heic\OpenWithList\{#MyAppExeName}"; ValueType: string; ValueName: ""; ValueData: ""; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\.heif\OpenWithList\{#MyAppExeName}"; ValueType: string; ValueName: ""; ValueData: ""; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\.avif\OpenWithList\{#MyAppExeName}"; ValueType: string; ValueName: ""; ValueData: ""; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\.ico\OpenWithList\{#MyAppExeName}"; ValueType: string; ValueName: ""; ValueData: ""; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\.svg\OpenWithList\{#MyAppExeName}"; ValueType: string; ValueName: ""; ValueData: ""; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\.jxl\OpenWithList\{#MyAppExeName}"; ValueType: string; ValueName: ""; ValueData: ""; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\.raw\OpenWithList\{#MyAppExeName}"; ValueType: string; ValueName: ""; ValueData: ""; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\.cr2\OpenWithList\{#MyAppExeName}"; ValueType: string; ValueName: ""; ValueData: ""; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\.nef\OpenWithList\{#MyAppExeName}"; ValueType: string; ValueName: ""; ValueData: ""; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\.arw\OpenWithList\{#MyAppExeName}"; ValueType: string; ValueName: ""; ValueData: ""; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\.dng\OpenWithList\{#MyAppExeName}"; ValueType: string; ValueName: ""; ValueData: ""; Flags: uninsdeletekey

[Code]
const
  UserEnvironmentKey = 'Environment';
  UserPathValue = 'Path';

function PathEntryMatches(Entry: string; Target: string): Boolean;
begin
  Result := CompareText(Trim(Entry), Target) = 0;
end;

function UserPathContains(Target: string): Boolean;
var
  ExistingPath: string;
  Entries: TArrayOfString;
  Index: Integer;
begin
  Result := False;
  if not RegQueryStringValue(HKEY_CURRENT_USER, UserEnvironmentKey, UserPathValue, ExistingPath) then
    Exit;

  Entries := StringSplit(ExistingPath, [';'], stExcludeEmpty);
  for Index := 0 to GetArrayLength(Entries) - 1 do
    if PathEntryMatches(Entries[Index], Target) then
    begin
      Result := True;
      Exit;
    end;
end;

procedure AddInstallDirectoryToUserPath;
var
  ExistingPath: string;
  InstallDirectory: string;
  NewPath: string;
begin
  InstallDirectory := RemoveBackslashUnlessRoot(ExpandConstant('{app}'));
  if UserPathContains(InstallDirectory) then
    Exit;

  if RegQueryStringValue(HKEY_CURRENT_USER, UserEnvironmentKey, UserPathValue, ExistingPath) and
     (ExistingPath <> '') then
    NewPath := ExistingPath + ';' + InstallDirectory
  else
    NewPath := InstallDirectory;

  RegWriteStringValue(HKEY_CURRENT_USER, UserEnvironmentKey, UserPathValue, NewPath);
end;

procedure RemoveInstallDirectoryFromUserPath;
var
  ExistingPath: string;
  Entries: TArrayOfString;
  Index: Integer;
  NewPath: string;
  InstallDirectory: string;
begin
  InstallDirectory := RemoveBackslashUnlessRoot(ExpandConstant('{app}'));
  if not RegQueryStringValue(HKEY_CURRENT_USER, UserEnvironmentKey, UserPathValue, ExistingPath) then
    Exit;

  Entries := StringSplit(ExistingPath, [';'], stExcludeEmpty);
  NewPath := '';
  for Index := 0 to GetArrayLength(Entries) - 1 do
    if not PathEntryMatches(Entries[Index], InstallDirectory) then
    begin
      if NewPath <> '' then
        NewPath := NewPath + ';';
      NewPath := NewPath + Entries[Index];
    end;

  if NewPath = '' then
    RegDeleteValue(HKEY_CURRENT_USER, UserEnvironmentKey, UserPathValue)
  else
    RegWriteStringValue(HKEY_CURRENT_USER, UserEnvironmentKey, UserPathValue, NewPath);
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssPostInstall then
    AddInstallDirectoryToUserPath;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then
    RemoveInstallDirectoryFromUserPath;
end;
