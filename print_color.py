#!/usr/bin/env python
# -*- coding: utf-8 -*-

import sys
import os, ctypes, platform

console_encoding = sys.getfilesystemencoding()

class print_style:
    engine = None

    FC_BLACK   = 0
    FC_BLUE    = 1
    FC_GREEN   = 2
    FC_CYAN    = 3
    FC_RED     = 4
    FC_MAGENTA = 5
    FC_YELLOW  = 6
    FC_WHITE    = 7

    BC_BLACK   = 8
    BC_BLUE    = 9
    BC_GREEN   = 10
    BC_CYAN    = 11
    BC_RED     = 12
    BC_MAGENTA = 13
    BC_YELLOW  = 14
    BC_WHITE    = 15

    FW_BOLD    = 16

class Win32ConsoleColor:
    STD_INPUT_HANDLE        = -10
    STD_OUTPUT_HANDLE       = -11
    STD_ERROR_HANDLE        = -12

    FOREGROUND_BLACK        = 0x0
    FOREGROUND_BLUE         = 0x01 # text color contains blue.
    FOREGROUND_GREEN        = 0x02 # text color contains green.
    FOREGROUND_RED          = 0x04 # text color contains red.
    FOREGROUND_INTENSITY    = 0x08 # text color is intensified.

    BACKGROUND_BLUE         = 0x10 # background color contains blue.
    BACKGROUND_GREEN        = 0x20 # background color contains green.
    BACKGROUND_RED          = 0x40 # background color contains red.
    BACKGROUND_INTENSITY    = 0x80 # background color is intensified.

    COLOR_MAP = {
        print_style.FC_BLACK: FOREGROUND_BLACK,
        print_style.FC_BLUE: FOREGROUND_BLUE | FOREGROUND_INTENSITY,
        print_style.FC_GREEN: FOREGROUND_GREEN | FOREGROUND_INTENSITY,
        print_style.FC_CYAN: FOREGROUND_GREEN | FOREGROUND_BLUE | FOREGROUND_INTENSITY,
        print_style.FC_RED: FOREGROUND_RED | FOREGROUND_INTENSITY,
        print_style.FC_MAGENTA: FOREGROUND_RED | FOREGROUND_BLUE | FOREGROUND_INTENSITY,
        print_style.FC_YELLOW: FOREGROUND_RED | FOREGROUND_GREEN | FOREGROUND_INTENSITY,
        print_style.FC_WHITE: FOREGROUND_BLUE | FOREGROUND_GREEN | FOREGROUND_RED,

        print_style.BC_BLACK: FOREGROUND_BLACK,
        print_style.BC_BLUE: BACKGROUND_BLUE,
        print_style.BC_GREEN: BACKGROUND_GREEN,
        print_style.BC_CYAN: BACKGROUND_BLUE | BACKGROUND_GREEN,
        print_style.BC_RED: BACKGROUND_RED,
        print_style.BC_MAGENTA: BACKGROUND_RED | BACKGROUND_BLUE,
        print_style.BC_YELLOW: BACKGROUND_RED | BACKGROUND_GREEN,
        print_style.BC_WHITE: BACKGROUND_RED | BACKGROUND_GREEN | BACKGROUND_BLUE,

        print_style.FW_BOLD: BACKGROUND_INTENSITY
    }

    std_out_handle = None
    std_err_handle = None

    def get_cmd_color(self, handle=std_out_handle):
        return Win32ConsoleColor.FOREGROUND_RED | Win32ConsoleColor.FOREGROUND_GREEN | Win32ConsoleColor.FOREGROUND_BLUE

    def set_cmd_color(self, color, handle=std_out_handle):
        """(color) -> bit
        Example: set_cmd_color(FOREGROUND_RED | FOREGROUND_GREEN | FOREGROUND_BLUE | FOREGROUND_INTENSITY)
        """
        bool = ctypes.windll.kernel32.SetConsoleTextAttribute(handle, color)
        return bool

    def stdout_with_color(self, options, text):
        style = Win32ConsoleColor.FOREGROUND_BLACK
        for opt in options:
            style = style | Win32ConsoleColor.COLOR_MAP[opt]
        if style == Win32ConsoleColor.FOREGROUND_BLACK:
            sys.stdout.write(text)
        else:
            old_style = self.get_cmd_color()
            self.set_cmd_color(style, self.std_out_handle)
            sys.stdout.write(text)
            self.set_cmd_color(old_style, self.std_out_handle)

    def stderr_with_color(self, options, text):
        style = Win32ConsoleColor.FOREGROUND_BLACK
        for opt in options:
            style = style | Win32ConsoleColor.COLOR_MAP[opt]
        if style == Win32ConsoleColor.FOREGROUND_BLACK:
            sys.stderr.write(text)
        else:
            old_style = self.get_cmd_color()
            self.set_cmd_color(style, self.std_err_handle)
            sys.stderr.write(text)
            self.set_cmd_color(old_style, self.std_err_handle)

class TermColor:
    COLOR_MAP = {
        print_style.FC_BLACK:   '30',
        print_style.FC_BLUE:    '34',
        print_style.FC_GREEN:   '32',
        print_style.FC_CYAN:    '36',
        print_style.FC_RED:     '31',
        print_style.FC_MAGENTA: '35',
        print_style.FC_YELLOW:  '33',
        print_style.FC_WHITE:   '37',

        print_style.BC_BLACK:   '40',
        print_style.BC_BLUE:    '44',
        print_style.BC_GREEN:   '42',
        print_style.BC_CYAN:    '46',
        print_style.BC_RED:     '41',
        print_style.BC_MAGENTA: '45',
        print_style.BC_YELLOW:  '43',
        print_style.BC_WHITE:   '47',

        print_style.FW_BOLD: '1'
    }

    def stdout_with_color(self, options, text):
        style = []
        for opt in options:
            style.append(TermColor.COLOR_MAP[opt])

        if len(style) > 0 and os.getenv('ANSI_COLORS_DISABLED') is None:
            sys.stdout.write('\033[' + ';'.join(style) + 'm' + text + '\033[0m')
        else:
            sys.stdout.write(text)

    def stderr_with_color(self, options, text):
        style = []
        for opt in options:
            style.append(TermColor.COLOR_MAP[opt])

        if len(style) > 0 and os.getenv('ANSI_COLORS_DISABLED') is None:
            sys.stderr.write('\033[' + ';'.join(style) + 'm' + text + '\033[0m')
        else:
            sys.stderr.write(text)

if 'windows' == platform.system().lower():
    ''''' See http://msdn.microsoft.com/library/default.asp?url=/library/en-us/winprog/winprog/windows_api_reference.asp
    for information on Windows APIs.'''
    Win32ConsoleColor.std_out_handle = ctypes.windll.kernel32.GetStdHandle(Win32ConsoleColor.STD_OUTPUT_HANDLE)
    Win32ConsoleColor.std_err_handle = ctypes.windll.kernel32.GetStdHandle(Win32ConsoleColor.STD_ERROR_HANDLE)

    print_style.engine = Win32ConsoleColor
else:
    print_style.engine = TermColor

def cprintf_unpack_text(fmt, text):
    if len(text) > 0:
        try:
            ret = fmt.format(*text)
            return ret
        except Exception:
            ret = fmt.decode('utf-8').encode(console_encoding).format(*text)
            return ret
    else:
        return fmt

def cprintf_stdout(options, fmt, *text):
    cp = print_style.engine()
    cp.stdout_with_color(options, cprintf_unpack_text(fmt, text))

def cprintf_stderr(options, fmt, *text):
    cp = print_style.engine()
    cp.stderr_with_color(options, cprintf_unpack_text(fmt, text))

if __name__ == "__main__":
    pass