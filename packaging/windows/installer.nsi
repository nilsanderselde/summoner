; Summoner DAW - Windows Installer Script (NSIS)
; Epic J, Task J2: Complete Windows installer with VC++ redistributable check,
; file associations (.summoner), and Add/Remove Programs registry integration.

!include "LogicLib.nsh"

!define APP_NAME "Summoner DAW"
!define APP_VERSION "1.0.0"
!define PUBLISHER "Summoner Team"
!define EXE_NAME "summon.exe"
!define UNINST_REG_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\SummonerDAW"

Name "${APP_NAME}"
OutFile "SummonerDAW-Installer-v${APP_VERSION}-x64.exe"
InstallDir "$PROGRAMFILES64\SummonerDAW"
InstallDirRegKey HKLM "Software\SummonerDAW" "InstallPath"
RequestExecutionLevel admin

Page directory
Page instfiles
UninstPage uninstConfirm
UninstPage instfiles

Function .onInit
    ; Check for Microsoft Visual C++ 2015-2022 Redistributable (x64)
    ClearErrors
    ReadRegDWORD $0 HKLM "SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\X64" "Installed"
    ${If} ${Errors}
        ClearErrors
        ReadRegDWORD $0 HKLM "SOFTWARE\WOW6432Node\Microsoft\VisualStudio\14.0\VC\Runtimes\x64" "Installed"
    ${EndIf}
    ${If} $0 != 1
        MessageBox MB_YESNO|MB_ICONEXCLAMATION \
            "Summoner DAW requires the Microsoft Visual C++ 2015-2022 Redistributable (x64) which does not appear to be installed.$\r$\n$\r$\nWould you like to proceed with the installation anyway?" \
            IDYES +2
        Abort
    ${EndIf}
FunctionEnd

Section "Main Application" Section1
    SetOutPath "$INSTDIR"
    File "..\..\target\release\summon.exe"
    File "..\..\README.md"
    File "..\..\LICENSE"
    
    ; Start menu shortcuts
    CreateDirectory "$SMPROGRAMS\Summoner DAW"
    CreateShortcut "$SMPROGRAMS\Summoner DAW\Summoner DAW.lnk" "$INSTDIR\summon.exe" "" "$INSTDIR\summon.exe" 0
    CreateShortcut "$SMPROGRAMS\Summoner DAW\Uninstall Summoner DAW.lnk" "$INSTDIR\uninstall.exe"
    CreateShortcut "$DESKTOP\Summoner DAW.lnk" "$INSTDIR\summon.exe" "" "$INSTDIR\summon.exe" 0
    
    ; Register file associations (.summoner and .summoner_proj)
    WriteRegStr HKCR ".summoner" "" "Summoner.Project"
    WriteRegStr HKCR ".summoner_proj" "" "Summoner.Project"
    WriteRegStr HKCR "Summoner.Project" "" "Summoner DAW Project"
    WriteRegStr HKCR "Summoner.Project\DefaultIcon" "" "$INSTDIR\summon.exe,0"
    WriteRegStr HKCR "Summoner.Project\shell" "" "open"
    WriteRegStr HKCR "Summoner.Project\shell\open\command" "" '"$INSTDIR\summon.exe" "%1"'

    ; Notify shell of association change
    System::Call 'shell32.dll::SHChangeNotify(i, i, i, i) v (0x08000000, 0, 0, 0)'
    
    ; Installation location registry key
    WriteRegStr HKLM "Software\SummonerDAW" "InstallPath" "$INSTDIR"

    ; Add/Remove Programs (Control Panel & Windows Settings)
    WriteRegStr HKLM "${UNINST_REG_KEY}" "DisplayName" "${APP_NAME}"
    WriteRegStr HKLM "${UNINST_REG_KEY}" "DisplayVersion" "${APP_VERSION}"
    WriteRegStr HKLM "${UNINST_REG_KEY}" "Publisher" "${PUBLISHER}"
    WriteRegStr HKLM "${UNINST_REG_KEY}" "UninstallString" '"$INSTDIR\uninstall.exe"'
    WriteRegStr HKLM "${UNINST_REG_KEY}" "QuietUninstallString" '"$INSTDIR\uninstall.exe" /S'
    WriteRegStr HKLM "${UNINST_REG_KEY}" "DisplayIcon" "$INSTDIR\summon.exe,0"
    WriteRegStr HKLM "${UNINST_REG_KEY}" "URLInfoAbout" "https://summoner.audio"
    WriteRegDWORD HKLM "${UNINST_REG_KEY}" "NoModify" 1
    WriteRegDWORD HKLM "${UNINST_REG_KEY}" "NoRepair" 1

    WriteUninstaller "$INSTDIR\uninstall.exe"
SectionEnd

Section "Uninstall"
    ; Remove files
    Delete "$INSTDIR\summon.exe"
    Delete "$INSTDIR\README.md"
    Delete "$INSTDIR\LICENSE"
    Delete "$INSTDIR\uninstall.exe"
    RMDir "$INSTDIR"
    
    ; Remove shortcuts
    Delete "$SMPROGRAMS\Summoner DAW\Summoner DAW.lnk"
    Delete "$SMPROGRAMS\Summoner DAW\Uninstall Summoner DAW.lnk"
    RMDir "$SMPROGRAMS\Summoner DAW"
    Delete "$DESKTOP\Summoner DAW.lnk"
    
    ; Remove file associations
    DeleteRegKey HKCR ".summoner"
    DeleteRegKey HKCR ".summoner_proj"
    DeleteRegKey HKCR "Summoner.Project"

    ; Notify shell of association change
    System::Call 'shell32.dll::SHChangeNotify(i, i, i, i) v (0x08000000, 0, 0, 0)'
    
    ; Remove Add/Remove Programs and app registry keys
    DeleteRegKey HKLM "${UNINST_REG_KEY}"
    DeleteRegKey HKLM "Software\SummonerDAW"
SectionEnd
