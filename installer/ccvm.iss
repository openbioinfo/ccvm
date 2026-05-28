#define AppName "ccvm"
#define AppPublisher "ccvm contributors"
#define AppURL "https://github.com/openbioinfo/ccvm"
#ifndef Version
  #define Version "0.1.0"
#endif

[Setup]
AppId={{4B8D6F2E-9A3C-4E1B-B7D5-8F6A2C3E1B9D}
AppName={#AppName}
AppVersion={#Version}
AppPublisher={#AppPublisher}
AppPublisherURL={#AppURL}
AppSupportURL={#AppURL}
AppUpdatesURL={#AppURL}
DefaultDirName={localappdata}\ccvm
DisableDirPage=no
DisableProgramGroupPage=yes
OutputBaseFilename=ccvm-setup-{#Version}
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=lowest
UninstallDisplayIcon={app}\ccvm.exe
ChangesEnvironment=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Files]
Source: "..\target\release\ccvm.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\target\release\ccvm-shim.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\target\release\ccvm-codex-shim.exe"; DestDir: "{app}"; Flags: ignoreversion

[Run]
Filename: "{app}\ccvm.exe"; Parameters: "setup"; Flags: nowait postinstall skipifsilent; Description: "Run ccvm setup (initialize directories and shim)"

[Code]
const
  EnvironmentKey = 'Environment';
  PathValue = 'PATH';

function DirInPath(const Path, Dir: string): Boolean;
var
  I, Start: Integer;
  Part: string;
  DirLower: string;
begin
  Result := False;
  DirLower := LowerCase(Dir);
  Start := 1;
  for I := 1 to Length(Path) do
  begin
    if Path[I] = ';' then
    begin
      Part := Trim(Copy(Path, Start, I - Start));
      if LowerCase(Part) = DirLower then
      begin
        Result := True;
        Exit;
      end;
      Start := I + 1;
    end;
  end;
  if Start <= Length(Path) then
  begin
    Part := Trim(Copy(Path, Start, Length(Path) - Start + 1));
    Result := LowerCase(Part) = DirLower;
  end;
end;

procedure RemoveFromPath(const Dir: string);
var
  CurrentPath: string;
  NewPath: string;
  I, Start: Integer;
  Part: string;
  DirLower: string;
begin
  DirLower := LowerCase(Dir);
  if not RegQueryStringValue(HKCU, EnvironmentKey, PathValue, CurrentPath) then
    Exit;

  NewPath := '';
  Start := 1;
  for I := 1 to Length(CurrentPath) do
  begin
    if CurrentPath[I] = ';' then
    begin
      Part := Trim(Copy(CurrentPath, Start, I - Start));
      if (Length(Part) > 0) and (LowerCase(Part) <> DirLower) then
      begin
        if Length(NewPath) > 0 then
          NewPath := NewPath + ';';
        NewPath := NewPath + Part;
      end;
      Start := I + 1;
    end;
  end;
  if Start <= Length(CurrentPath) then
  begin
    Part := Trim(Copy(CurrentPath, Start, Length(CurrentPath) - Start + 1));
    if (Length(Part) > 0) and (LowerCase(Part) <> DirLower) then
    begin
      if Length(NewPath) > 0 then
        NewPath := NewPath + ';';
      NewPath := NewPath + Part;
    end;
  end;

  if NewPath <> CurrentPath then
  begin
    if Length(NewPath) > 0 then
      RegWriteExpandStringValue(HKCU, EnvironmentKey, PathValue, NewPath)
    else
      RegDeleteValue(HKCU, EnvironmentKey, PathValue);
  end;
end;

procedure AddToPath(const Dir: string);
var
  CurrentPath: string;
begin
  if RegQueryStringValue(HKCU, EnvironmentKey, PathValue, CurrentPath) then
  begin
    if DirInPath(CurrentPath, Dir) then
      Exit;
  end;

  if Length(CurrentPath) > 0 then
    CurrentPath := CurrentPath + ';';
  CurrentPath := CurrentPath + Dir;
  RegWriteExpandStringValue(HKCU, EnvironmentKey, PathValue, CurrentPath);
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssPostInstall then
  begin
    AddToPath(ExpandConstant('{app}'));
  end;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then
  begin
    RemoveFromPath(ExpandConstant('{app}'));
  end;
end;
