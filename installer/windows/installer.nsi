; VantisWeb Browser Installer Script for NSIS
; Creates a professional Windows installer with all features

!include "MUI2.nsh"
!include "FileFunc.nsh"
!include "LogicLib.nsh"
!include "x64.nsh"

; Application information
!define PRODUCT_NAME "VantisWeb Browser"
!define PRODUCT_VERSION "1.5.0"
!define PRODUCT_PUBLISHER "Vantis Corp"
!define PRODUCT_WEB_SITE "https://vantis.ai"
!define PRODUCT_DIR_REGKEY "Software\Microsoft\Windows\CurrentVersion\App Paths\VantisWeb.exe"
!define PRODUCT_UNINST_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}"
!define PRODUCT_UNINST_ROOT_KEY "HKLM"

; Installer settings
Name "${PRODUCT_NAME} ${PRODUCT_VERSION}"
OutFile "VantisWeb-Setup-${PRODUCT_VERSION}.exe"
InstallDir "$PROGRAMFILES64\VantisWeb"
InstallDirRegKey HKLM "Software\VantisWeb" "Install_Dir"
RequestExecutionLevel admin
SetCompressor /SOLID lzma
SetCompressorDictSize 64

; Interface settings
!define MUI_ABORTWARNING
!define MUI_ICON "assets\icons\icon.ico"
!define MUI_UNICON "assets\icons\icon.ico"
!define MUI_WELCOMEFINISHPAGE_BITMAP "assets\installer\welcome.bmp"
!define MUI_HEADERIMAGE
!define MUI_HEADERIMAGE_BITMAP "assets\installer\header.bmp"
!define MUI_HEADERIMAGE_RIGHT

; Pages
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "LICENSE"
!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!define MUI_FINISHPAGE_RUN "$INSTDIR\VantisWeb.exe"
!define MUI_FINISHPAGE_LINK "Visit VantisWeb Website" "${PRODUCT_WEB_SITE}"
!insertmacro MUI_PAGE_FINISH

; Uninstaller pages
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

; Language files
!insertmacro MUI_LANGUAGE "English"
!insertmacro MUI_LANGUAGE "Polish"
!insertmacro MUI_LANGUAGE "German"
!insertmacro MUI_LANGUAGE "French"
!insertmacro MUI_LANGUAGE "Spanish"
!insertmacro MUI_LANGUAGE "Japanese"
!insertmacro MUI_LANGUAGE "Korean"
!insertmacro MUI_LANGUAGE "Russian"

; Version information
VIProductVersion "1.5.0.0"
VIAddVersionKey /LANG=${LANG_ENGLISH} "ProductName" "${PRODUCT_NAME}"
VIAddVersionKey /LANG=${LANG_ENGLISH} "Comments" "Next-generation web browser with Liquid Core Architecture"
VIAddVersionKey /LANG=${LANG_ENGLISH} "CompanyName" "${PRODUCT_PUBLISHER}"
VIAddVersionKey /LANG=${LANG_ENGLISH} "LegalCopyright" "Copyright (c) 2024-2026 Vantis Corp"
VIAddVersionKey /LANG=${LANG_ENGLISH} "FileDescription" "${PRODUCT_NAME} Installer"
VIAddVersionKey /LANG=${LANG_ENGLISH} "FileVersion" "${PRODUCT_VERSION}"
VIAddVersionKey /LANG=${LANG_ENGLISH} "ProductVersion" "${PRODUCT_VERSION}"

; Installer sections
Section "!VantisWeb Browser (required)" SecMain
    SectionIn RO
    
    SetOutPath "$INSTDIR"
    
    ; Main executable
    File "target\release\VantisWeb.exe"
    
    ; Core files
    File /r "resources\*.*"
    
    ; Documentation
    File "README.md"
    File "LICENSE"
    File "CHANGELOG.md"
    
    ; Create directories
    CreateDirectory "$INSTDIR\profiles"
    CreateDirectory "$INSTDIR\extensions"
    CreateDirectory "$INSTDIR\cache"
    CreateDirectory "$INSTDIR\logs"
    
    ; Create uninstaller
    WriteUninstaller "$INSTDIR\Uninstall.exe"
    
    ; Registry entries
    WriteRegStr HKLM "Software\VantisWeb" "Install_Dir" "$INSTDIR"
    WriteRegStr HKLM "Software\VantisWeb" "Version" "${PRODUCT_VERSION}"
    
    WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "DisplayName" "$(^Name)"
    WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "UninstallString" "$INSTDIR\Uninstall.exe"
    WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "DisplayVersion" "${PRODUCT_VERSION}"
    WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "URLInfoAbout" "${PRODUCT_WEB_SITE}"
    WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "Publisher" "${PRODUCT_PUBLISHER}"
    WriteRegDWORD ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "NoModify" 1
    WriteRegDWORD ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "NoRepair" 1
    
    ; Calculate installed size
    ${GetSize} "$INSTDIR" "/S=0K" $0
    IntFmt $0 "0x%08X" $0
    WriteRegDWORD ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "EstimatedSize" "$0"
SectionEnd

Section "Desktop Shortcut" SecDesktop
    CreateShortCut "$DESKTOP\VantisWeb Browser.lnk" "$INSTDIR\VantisWeb.exe" "" "$INSTDIR\VantisWeb.exe" 0
SectionEnd

Section "Start Menu Shortcuts" SecStartMenu
    CreateDirectory "$SMPROGRAMS\VantisWeb"
    CreateShortCut "$SMPROGRAMS\VantisWeb\VantisWeb Browser.lnk" "$INSTDIR\VantisWeb.exe" "" "$INSTDIR\VantisWeb.exe" 0
    CreateShortCut "$SMPROGRAMS\VantisWeb\Uninstall.lnk" "$INSTDIR\Uninstall.exe" "" "$INSTDIR\Uninstall.exe" 0
    CreateShortCut "$SMPROGRAMS\VantisWeb\README.lnk" "$INSTDIR\README.md" "" "" 0
SectionEnd

Section "Register as Default Browser" SecDefaultBrowser
    Call RegisterBrowser
SectionEnd

Section "Taskbar Pin" SecTaskbar
    ; Windows 7+ taskbar pin
    ExecWait '"$SYSDIR\shell32.dll",ShellSpecialFolder 0x4000'
SectionEnd

; Section descriptions
!insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
    !insertmacro MUI_DESCRIPTION_TEXT ${SecMain} "Core VantisWeb Browser files (required)"
    !insertmacro MUI_DESCRIPTION_TEXT ${SecDesktop} "Create a desktop shortcut for easy access"
    !insertmacro MUI_DESCRIPTION_TEXT ${SecStartMenu} "Create Start Menu shortcuts"
    !insertmacro MUI_DESCRIPTION_TEXT ${SecDefaultBrowser} "Set VantisWeb as your default web browser"
    !insertmacro MUI_DESCRIPTION_TEXT ${SecTaskbar} "Pin VantisWeb to the taskbar"
!insertmacro MUI_FUNCTION_DESCRIPTION_END

; Register as browser function
Function RegisterBrowser
    ; Register as a web browser
    WriteRegStr HKLM "SOFTWARE\Clients\StartMenuInternet\VantisWeb" "" "VantisWeb Browser"
    WriteRegStr HKLM "SOFTWARE\Clients\StartMenuInternet\VantisWeb\DefaultIcon" "" "$INSTDIR\VantisWeb.exe,0"
    WriteRegStr HKLM "SOFTWARE\Clients\StartMenuInternet\VantisWeb\shell\open\command" "" '"$INSTDIR\VantisWeb.exe"'
    
    ; Register URL protocols
    WriteRegStr HKCR "VantisWeb" "" "VantisWeb Browser Document"
    WriteRegStr HKCR "VantisWeb\shell\open\command" "" '"$INSTDIR\VantisWeb.exe" "%1"'
    
    ; HTTP protocol
    WriteRegStr HKCR "http\shell\VantisWeb" "" "Open with VantisWeb"
    WriteRegStr HKCR "http\shell\VantisWeb\command" "" '"$INSTDIR\VantisWeb.exe" "%1"'
    
    ; HTTPS protocol
    WriteRegStr HKCR "https\shell\VantisWeb" "" "Open with VantisWeb"
    WriteRegStr HKCR "https\shell\VantisWeb\command" "" '"$INSTDIR\VantisWeb.exe" "%1"'
    
    ; HTML file association
    WriteRegStr HKCR ".htm\OpenWithProgids" "VantisWeb" ""
    WriteRegStr HKCR ".html\OpenWithProgids" "VantisWeb" ""
    WriteRegStr HKCR "VantisWeb\shell\open\command" "" '"$INSTDIR\VantisWeb.exe" "%1"'
FunctionEnd

; Uninstaller
Section "Uninstall"
    ; Remove files
    Delete "$INSTDIR\*.*"
    RMDir /r "$INSTDIR\resources"
    RMDir /r "$INSTDIR\profiles"
    RMDir /r "$INSTDIR\extensions"
    RMDir /r "$INSTDIR\cache"
    RMDir /r "$INSTDIR\logs"
    
    ; Remove uninstaller
    Delete "$INSTDIR\Uninstall.exe"
    RMDir "$INSTDIR"
    
    ; Remove shortcuts
    Delete "$DESKTOP\VantisWeb Browser.lnk"
    Delete "$SMPROGRAMS\VantisWeb\*.*"
    RMDir "$SMPROGRAMS\VantisWeb"
    
    ; Remove registry keys
    DeleteRegKey ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}"
    DeleteRegKey HKLM "Software\VantisWeb"
    
    ; Remove browser registration
    DeleteRegKey HKLM "SOFTWARE\Clients\StartMenuInternet\VantisWeb"
    DeleteRegKey HKCR "VantisWeb"
    DeleteRegValue HKCR "http\shell\VantisWeb" ""
    DeleteRegValue HKCR "https\shell\VantisWeb" ""
    DeleteRegValue HKCR ".htm\OpenWithProgids" "VantisWeb"
    DeleteRegValue HKCR ".html\OpenWithProgids" "VantisWeb"
    
    ; Remove from default programs
    DeleteRegKey HKLM "Software\VantisWeb\Capabilities"
    DeleteRegValue HKLM "Software\RegisteredApplications" "VantisWeb"
    
    ; Show success message
    MessageBox MB_OK "VantisWeb Browser has been successfully uninstalled."
SectionEnd

; Custom initialization
Function .onInit
    ; Check Windows version
    ${If} ${AtLeastWin10}
        ; Windows 10+ specific initialization
    ${ElseIf} ${AtLeastWin8.1}
        ; Windows 8.1 specific initialization
    ${ElseIf} ${AtLeastWin8}
        ; Windows 8 specific initialization
    ${ElseIf} ${AtLeastWin7}
        ; Windows 7 specific initialization
    ${Else}
        MessageBox MB_YESNO|MB_ICONQUESTION \
            "Your Windows version may not be fully supported.$\n$\nContinue anyway?" \
            /SD IDYES IDYES continue
        Abort
        continue:
    ${EndIf}
    
    ; Check for existing installation
    ReadRegStr $0 HKLM "Software\VantisWeb" "Install_Dir"
    ${If} $0 != ""
        MessageBox MB_YESNO|MB_ICONQUESTION \
            "An existing installation of VantisWeb was found.$\n$\nWould you like to update it?" \
            /SD IDYES IDYES update
        Abort
        update:
        ; Remove old version
        RMDir /r "$0"
    ${EndIf}
FunctionEnd

; Post-install function
Function .onInstSuccess
    ; Create first-run marker
    FileOpen $0 "$INSTDIR\.first_run" w
    FileWrite $0 "${PRODUCT_VERSION}"
    FileClose $0
    
    ; Ask to launch
    MessageBox MB_YESNO|MB_ICONQUESTION \
        "Installation complete!$\n$\nWould you like to launch VantisWeb now?" \
        /SD IDYES IDYES launch
    Return
    
    launch:
    Exec '"$INSTDIR\VantisWeb.exe"'
FunctionEnd