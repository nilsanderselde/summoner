; Summoner DAW - Windows Installer Script (NSIS)
; Step 530: Windows NSIS installer definition

!define APP_NAME "Summoner DAW"
!define APP_VERSION "1.0.0"
!define PUBLISHER "Summoner Team"
!define EXE_NAME "summon.exe"

Name "${APP_NAME}"
OutFile "SummonerDAW-Installer-v${APP_VERSION}-x64.exe"
InstallDir "$PROGRAMFILES64\SummonerDAW"
RequestExecutionLevel admin

Page directory
Page instfiles
UninstPage uninstConfirm
UninstPage instfiles

Section "Main Application" Section1
    SetOutPath "$INSTDIR"
    File "..\..\target\release\summon.exe"
    File "..\..\README.md"
    File "..\..\LICENSE"
    
    CreateDirectory "$SMPROGRAMS\Summoner DAW"
    CreateShortcut "$SMPROGRAMS\Summoner DAW\Summoner DAW.lnk" "$INSTDIR\summon.exe"
    CreateShortcut "$DESKTOP\Summoner DAW.lnk" "$INSTDIR\summon.exe"
    
    WriteUninstaller "$INSTDIR\uninstall.exe"
SectionEnd

Section "Uninstall"
    Delete "$INSTDIR\summon.exe"
    Delete "$INSTDIR\README.md"
    Delete "$INSTDIR\LICENSE"
    Delete "$INSTDIR\uninstall.exe"
    RMDir "$INSTDIR"
    Delete "$SMPROGRAMS\Summoner DAW\Summoner DAW.lnk"
    RMDir "$SMPROGRAMS\Summoner DAW"
    Delete "$DESKTOP\Summoner DAW.lnk"
SectionEnd
