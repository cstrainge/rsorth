
use std::{ mem::zeroed, io::Error };
use winapi::{ shared::minwindef::{ DWORD, UINT },
              um::{ consoleapi::{ GetConsoleMode,
                                  GetNumberOfConsoleInputEvents,
                                  ReadConsoleInputA,
                                  SetConsoleMode },
                    handleapi::INVALID_HANDLE_VALUE,
                    processenv::GetStdHandle,
                    winbase::{ STD_INPUT_HANDLE, STD_OUTPUT_HANDLE},
                    wincon::{ GetConsoleScreenBufferInfo,
                              SetConsoleCP,
                              SetConsoleOutputCP,
                              CONSOLE_SCREEN_BUFFER_INFO,
                              ENABLE_ECHO_INPUT,
                              ENABLE_INSERT_MODE,
                              ENABLE_LINE_INPUT,
                              ENABLE_PROCESSED_INPUT,
                              ENABLE_PROCESSED_OUTPUT,
                              ENABLE_VIRTUAL_TERMINAL_INPUT,
                              ENABLE_VIRTUAL_TERMINAL_PROCESSING,
                              KEY_EVENT_RECORD },
                    wincontypes::INPUT_RECORD } };
use crate::runtime::{ data_structures::value::ToValue,
                      error::{self, script_error},
                      interpreter::Interpreter };



static mut INPUT_MODE: DWORD = 0;
static mut OUTPUT_MODE: DWORD = 0;
static mut IS_IN_RAW_MODE: bool = false;

const CP_UTF8: UINT = 65001;

const VK_ESCAPE: u16 = 0x1B;
const KEY_EVENT: u16 = 1;



fn flush_events(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    unsafe
    {
        let std_in_handle = GetStdHandle(STD_INPUT_HANDLE);

        if std_in_handle == INVALID_HANDLE_VALUE
        {
            script_error(interpreter, format!("Get console handle failed: {}",
                                              Error::last_os_error()))?;
        }

        let mut number_of_events: DWORD = 0;

        if GetNumberOfConsoleInputEvents(std_in_handle, &mut number_of_events) == 0
        {
            script_error(interpreter, format!("Could not read input events: {}",
                                              Error::last_os_error()))?;
        }

        if number_of_events > 0
        {
            loop
            {
                let mut input: INPUT_RECORD = zeroed();
                let mut read: DWORD = 0;

                if ReadConsoleInputA(std_in_handle, &raw mut input, 1, &mut read) == 0
                {
                    script_error(interpreter, format!("Reading from console input failed: {}",
                                                      Error::last_os_error()))?;
                }

                if input.EventType == KEY_EVENT
                {
                    let key_event: KEY_EVENT_RECORD = *input.Event.KeyEvent();

                    if key_event.wVirtualKeyCode == VK_ESCAPE
                    {
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn init_win_console()
{
    unsafe
    {
        SetConsoleCP(CP_UTF8);
        SetConsoleOutputCP(CP_UTF8);
    }
}

pub fn word_term_raw_mode(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let requested_on = interpreter.pop_as_bool()?;

    unsafe
    {
        let std_in_handle = GetStdHandle(STD_INPUT_HANDLE);
        let std_out_handle = GetStdHandle(STD_OUTPUT_HANDLE);

        if requested_on && !IS_IN_RAW_MODE
        {
            if GetConsoleMode(std_in_handle, &raw mut INPUT_MODE) == 0
            {
                script_error(interpreter, format!("Get console input mode failed: {}",
                                                  Error::last_os_error()))?;
            }

            if GetConsoleMode(std_out_handle, &raw mut OUTPUT_MODE) == 0
            {
                script_error(interpreter, format!("Get console output mode failed: {}",
                                                  Error::last_os_error()))?;
            }

            let mut new_input_mode = INPUT_MODE;
            let mut new_output_mode = OUTPUT_MODE;

            new_input_mode &= !(  ENABLE_ECHO_INPUT
                                | ENABLE_INSERT_MODE
                                | ENABLE_LINE_INPUT
                                | ENABLE_PROCESSED_INPUT);
            new_input_mode |= ENABLE_VIRTUAL_TERMINAL_INPUT;

            new_output_mode |=   ENABLE_PROCESSED_OUTPUT
                               | ENABLE_VIRTUAL_TERMINAL_PROCESSING;

            if SetConsoleMode(std_in_handle, new_input_mode) == 0
            {
                script_error(interpreter, format!("Set console input mode failed: {}",
                                                  Error::last_os_error()))?;
            }

            if SetConsoleMode(std_out_handle, new_output_mode) == 0
            {
                script_error(interpreter, format!("Set console output mode failed: {}",
                                                  Error::last_os_error()))?;
            }

            IS_IN_RAW_MODE = true;

            flush_events(interpreter)?;
        }
        else if !requested_on && IS_IN_RAW_MODE
        {
            if SetConsoleMode(std_in_handle, INPUT_MODE) == 0
            {
                script_error(interpreter, format!("Set console input mode failed: {}",
                                                  Error::last_os_error()))?;
            }

            if SetConsoleMode(std_in_handle, INPUT_MODE) == 0
            {
                script_error(interpreter, format!("Set console output mode failed: {}",
                                                  Error::last_os_error()))?;
            }

            IS_IN_RAW_MODE = false;

            flush_events(interpreter)?;
        }
    }

    Ok(())
}

pub fn word_term_size(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    unsafe
    {
        let std_out_handle = GetStdHandle(STD_OUTPUT_HANDLE);

        if std_out_handle == INVALID_HANDLE_VALUE
        {
            script_error(interpreter, format!("Get console handle failed: {}",
                                              Error::last_os_error()))?;
        }

        let mut info: CONSOLE_SCREEN_BUFFER_INFO = zeroed();

        if GetConsoleScreenBufferInfo(std_out_handle, &mut info) == 0
        {
            script_error(interpreter, format!("Get console information failed: {}",
                                              Error::last_os_error()))?;
        }

        interpreter.push((info.dwSize.X as i64).to_value());
        interpreter.push((info.dwSize.Y as i64).to_value());
    }

    Ok(())
}

pub fn word_term_key(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    unsafe
    {
        let std_in_handle = GetStdHandle(STD_INPUT_HANDLE);

        if std_in_handle == INVALID_HANDLE_VALUE
        {
            script_error(interpreter, format!("Get console handle failed: {}",
                                              Error::last_os_error()))?;
        }

        let mut buffer: INPUT_RECORD = zeroed();
        let mut read: DWORD = 0;

        loop
        {
            if ReadConsoleInputA(std_in_handle, &mut buffer, 1, &mut read) == 0
            {
                script_error(interpreter, format!("Reading from console input failed: {}",
                                                  Error::last_os_error()))?;
            }

            if buffer.EventType == KEY_EVENT
            {
                let key_event = buffer.Event.KeyEvent();

                if key_event.bKeyDown != 0
                {
                    let character = *key_event.uChar.AsciiChar() as u8 as char;
                    interpreter.push(character.to_string().to_value());
                    break;
                }
            }
        }
    }

    Ok(())
}
