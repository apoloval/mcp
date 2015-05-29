#define AppName "MSX CAS Packager"
#define AppVersion "0.1.0"

[setup]
AppCopyright=Copyright (C) 2015 Alvaro Polo
AppName=MSX CAS Packager
AppVersion={#AppVersion}
AppSupportURL=http://github.com/apoloval/mcp/issues
ArchitecturesAllowed={#AppArchitecture}
#if "x64" == AppArchitecture
ArchitecturesInstallIn64BitMode=x64
#endif
DefaultDirName={pf}\{#AppName}
DefaultGroupName={#AppName}
LicenseFile=..\..\COPYING
OutputBaseFilename="mcp-{#AppVersion}_{#AppArchitecture}"
OutputDir=.

[files]
Source: "..\..\target\release\mcp.exe"; DestDir: "{app}"

[icons]
Name: "{group}\MCP Shell"; Filename: "cmd.exe"; Parameters: "/K path {app};%PATH%"; WorkingDir: "{userdocs}"
