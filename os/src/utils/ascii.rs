use core::default;

use num_enum::TryFromPrimitive;

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u8)]
pub enum AsciiCode {
    /// Null; ^@; Ctrl+@
    NUL = 0,
    // Null = 0,
    // CtrlAt = 0,
    /// Start of Heading; ^A; Ctrl+A
    SOH = 1,
    // StartOfHeading = 1,
    // CtrlA = 1,
    /// Start of Text; ^B; Ctrl+B
    STX = 2,
    // StartOfText = 2,
    // CtrlB = 2,
    /// End of Text; ^C; Ctrl+C
    ETX = 3,
    // EndOfText = 3,
    // CtrlC = 3,
    /// End of Transmission; ^D; Ctrl+D
    EOT = 4,
    // EndOfTransmission = 4,
    // CtrlD = 4,
    /// Enquiry; ^E; Ctrl+E
    ENQ = 5,
    // Enquiry = 5,
    // CtrlE = 5,
    /// Acknowledge; ^F; Ctrl+F
    ACK = 6,
    // Acknowledge = 6,
    // CtrlF = 6,
    /// Bell; ^G; Ctrl+G
    BEL = 7,
    // Bell = 7,
    // CtrlG = 7,
    /// Backspace; ^H; Backspace or Ctrl+H
    BS = 8,
    // Backspace = 8,
    // CtrlH = 8,
    /// Horizontal Tab; ^I; Tab or Ctrl+I
    HT = 9,
    // HorizontalTab = 9,
    // Tab = 9,
    // CtrlI = 9,
    /// Line Feed; ^J; Enter or Ctrl+J
    LF = 10,
    // LineFeed = 10,
    // EnterN = 10,
    // CtrlJ = 10,
    /// Vertical Tab; ^K; Ctrl+K
    VT = 11,
    // VerticalTab = 11,
    // CtrlK = 11,
    /// Form Feed; ^L; Ctrl+L
    FF = 12,
    // FormFeed = 12,
    // CtrlL = 12,
    /// Carriage Return; ^M; Enter or Ctrl+M
    CR = 13,
    // CarriageReturn = 13,
    // EnterR = 13,
    // CtrlM = 13,
    /// Shift Out; ^N; Ctrl+N
    SO = 14,
    // ShiftOut = 14,
    // CtrlN = 14,
    /// Shift In; ^O; Ctrl+O
    SI = 15,
    // ShiftIn = 15,
    // CtrlO = 15,
    /// Data Link Escape; ^P; Ctrl+P
    DLE = 16,
    // DataLinkEscape = 16,
    // CtrlP = 16,
    /// Device Control 1 (XON); ^Q; Ctrl+Q
    DC1 = 17,
    // DeviceControl1 = 17,
    // CtrlQ = 17,
    /// Device Control 2 (XOFF); ^R; Ctrl+R
    DC2 = 18,
    // DeviceControl2 = 18,
    // CtrlR = 18,
    /// Device Control 3; ^S; Ctrl+S
    DC3 = 19,
    // DeviceControl3 = 19,
    // CtrlS = 19,
    /// Device Control 4; ^T; Ctrl+T
    DC4 = 20,
    // DeviceControl4 = 20,
    // CtrlT = 20,
    /// Negative Acknowledge; ^U; Ctrl+U
    NAK = 21,
    // NegativeAcknowledge = 21,
    // CtrlU = 21,
    /// Synchronous Idle; ^V; Ctrl+V
    SYN = 22,
    // SynchronousIdle = 22,
    // CtrlV = 22,
    /// End of Transmission Block; ^W; Ctrl+W
    ETB = 23,
    // EndOfTransmissionBlock = 23,
    // CtrlW = 23,
    /// Cancel; ^X; Ctrl+X
    CAN = 24,
    // Cancel = 24,
    // CtrlX = 24,
    /// End of Medium; ^Y; Ctrl+Y
    EM = 25,
    // EndOfMedium = 25,
    // CtrlY = 25,
    /// Substitute; ^Z; Ctrl+Z
    SUB = 26,
    // Substitute = 26,
    // CtrlZ = 26,
    /// Escape; ^[; Ctrl+[
    ESC = 27,
    // Escape = 27,
    // CtrlLBracket = 27,
    /// File Separator; ^\; Ctrl+\
    FS = 28,
    // FileSeparator = 28,
    // CtrlBackslash = 28,
    /// Group Separator; ^]; Ctrl+]
    GS = 29,
    // GroupSeparator = 29,
    // CtrlRBracket = 29,
    /// Record Separator; ^^; Ctrl+^
    RS = 30,
    // RecordSeparator = 30,
    // CtrlCaret = 30,
    /// Unit Separator; ^_; Ctrl+_
    US = 31,
    // UnitSeparator = 31,
    // CtrlUnderscore = 31,
    /// Delete; ^?; Delete or Ctrl+?
    DEL = 127
    // Delete = 127,
    // CtrlQuestion = 127,
}