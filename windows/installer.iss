; ============================================================================
;  Toolbox - Windows 安装包脚本 (Inno Setup 6.3+)
;
;  用法：
;      1. cargo build --release
;      2. ISCC.exe windows\installer.iss
;         或直接用 Inno Setup IDE 打开本文件按 F9
;
;  安装包输出到 dist\Toolbox-<版本>-windows-x64-setup.exe
;
;  可用 /D 覆盖的宏（均可省略）：
;      AppVersion  应用版本号，缺省 1.4.1
;      SourceExe   待打包的 exe 路径，缺省 ..\target\release\toolbox.exe
;      OutputDir   输出目录，缺省 ..\dist
;      ChineseIsl  简体中文语言文件路径；传入后向导切换为中文
;                  例：/D"ChineseIsl=C:\Program Files (x86)\Inno Setup 6\Languages\ChineseSimplified.isl"
;
;  可选素材：
;      windows\toolbox.ico               有则用作安装包与快捷方式图标
;      windows\redist\vc_redist.x64.exe  有则在缺少 VC++ 运行库时自动安装
; ============================================================================

#define AppName "Toolbox"
#define AppExeName "toolbox.exe"
#define AppPublisher "LittleGuest"
#define AppUrl "https://github.com/LittleGuest/toolbox"

#ifndef AppVersion
  #define AppVersion "1.4.1"
#endif

#ifndef SourceExe
  #define SourceExe "..\target\release\toolbox.exe"
#endif

#ifndef OutputDir
  #define OutputDir "..\dist"
#endif

; 以脚本所在目录为基准解析仓库内其它资源
#define RepoDir AddBackslash(SourcePath) + ".."
#define LicensePath RepoDir + "\LICENSE"
#define IconPath RepoDir + "\icons\toolbox.ico"
#define VCRedistPath AddBackslash(SourcePath) + "redist\vc_redist.x64.exe"

[Setup]
; AppId 是升级/卸载的识别依据，一旦发布不要再改动
AppId={{8F2A7C41-6E3B-4C9D-AE45-2B7A1F6D8C30}
AppName={#AppName}
AppVersion={#AppVersion}
AppVerName={#AppName} {#AppVersion}
AppPublisher={#AppPublisher}
AppPublisherURL={#AppUrl}
AppSupportURL={#AppUrl}/issues
AppUpdatesURL={#AppUrl}/releases
VersionInfoVersion={#AppVersion}
VersionInfoDescription={#AppName} 安装程序
VersionInfoProductName={#AppName}
VersionInfoProductVersion={#AppVersion}

DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
AllowNoIcons=yes
LicenseFile={#LicensePath}

OutputDir={#OutputDir}
OutputBaseFilename={#AppName}-{#AppVersion}-windows-x64-setup
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern

; GPUI 依赖 DirectX/DirectComposition，仅支持 64 位 Windows 10+
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0

; 默认按当前用户安装（免 UAC），允许用户切换为全机器安装
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog

; 覆盖升级时自动关闭正在运行的旧版本
CloseApplications=yes
RestartApplications=no

UninstallDisplayName={#AppName} {#AppVersion}
UninstallDisplayIcon={app}\{#AppExeName}

#if FileExists(IconPath)
SetupIconFile={#IconPath}
#endif

[Languages]
Name: "en"; MessagesFile: "compiler:Default.isl"
; 官方安装包未内置简体中文语言文件，传入 /DChineseIsl=<路径> 后启用
#ifdef ChineseIsl
Name: "zh"; MessagesFile: "{#ChineseIsl}"
#endif

[CustomMessages]
; 未启用中文语言文件时，向导外框为英文，但这些自定义文案仍是中文
CreateDesktopIcon=创建桌面快捷方式(&D)
AdditionalIcons=附加快捷方式：
RemoveUserData=是否同时删除 {#AppName} 的配置与用户数据？位置：%1 。选择"是"将永久删除该目录；选择"否"则保留，方便以后重装后继续使用。
#ifdef ChineseIsl
en.CreateDesktopIcon=Create a &desktop shortcut
en.AdditionalIcons=Additional shortcuts:
en.RemoveUserData=Also delete {#AppName} settings and user data? Location: %1 . Choose Yes to permanently delete this folder, or No to keep it for a future reinstall.
#endif

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "{#SourceExe}"; DestDir: "{app}"; DestName: "{#AppExeName}"; Flags: ignoreversion
Source: "{#LicensePath}"; DestDir: "{app}"; DestName: "LICENSE"; Flags: ignoreversion
#if FileExists(VCRedistPath)
; 可选：把 vc_redist.x64.exe 放到 windows\redist\ 下，缺少运行库时自动安装
Source: "{#VCRedistPath}"; DestDir: "{tmp}"; Flags: deleteafterinstall; Check: NeedsVCRedist
#endif

[Icons]
; 快捷方式图标自动取自 exe 内嵌图标（由 crates/toolbox/build.rs 注入）
Name: "{group}\{#AppName}"; Filename: "{app}\{#AppExeName}"; WorkingDir: "{app}"
Name: "{group}\{cm:UninstallProgram,{#AppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExeName}"; WorkingDir: "{app}"; Tasks: desktopicon

[Run]
#if FileExists(VCRedistPath)
Filename: "{tmp}\vc_redist.x64.exe"; Parameters: "/install /quiet /norestart"; StatusMsg: "正在安装 Microsoft Visual C++ 运行库..."; Flags: waituntilterminated; Check: NeedsVCRedist
#endif
Filename: "{app}\{#AppExeName}"; Description: "{cm:LaunchProgram,{#AppName}}"; WorkingDir: "{app}"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
; 安装目录内的缓存/日志（用户配置在 %APPDATA%\toolbox，卸载时单独询问）
Type: filesandordirs; Name: "{app}\logs"

[Code]
{ Toolbox 由 MSVC 工具链构建，动态链接 VC++ 运行库，缺少时需要先安装 }
function NeedsVCRedist(): Boolean;
var
  Installed: Cardinal;
begin
  if RegQueryDWordValue(HKLM64, 'SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64', 'Installed', Installed)
     and (Installed = 1) then
  begin
    Result := False;
    Exit;
  end;
  Result := not FileExists(ExpandConstant('{win}\System32\vcruntime140.dll'));
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  DataDir: String;
  Msg: String;
begin
  if CurUninstallStep <> usPostUninstall then
    Exit;

  { 静默卸载（/SILENT、/VERYSILENT，含 winget 等批量场景）绝不删用户数据 }
  if UninstallSilent then
    Exit;

  { 与 config_store.rs 保持一致：dirs::config_dir()/toolbox }
  DataDir := ExpandConstant('{userappdata}\toolbox');
  if not DirExists(DataDir) then
    Exit;

  Msg := CustomMessage('RemoveUserData');
  if Msg = '' then
    Msg := '{#AppName} 的用户配置目录：%1 ，是否一并删除？';
  Msg := FmtMessage(Msg, [DataDir]);

  if MsgBox(Msg, mbConfirmation, MB_YESNO or MB_DEFBUTTON2) = IDYES then
    DelTree(DataDir, True, True, True);
end;
