
use std::{ io::{ Error,
                 ErrorKind::Interrupted,
                 Read,
                 stdin },
           mem::zeroed };
use libc::{ BRKINT,
            CS8,
            ECHO,
            ICANON,
            ICRNL,
            IEXTEN,
            INPCK,
            ioctl,
            ISIG,
            ISTRIP,
            IXON,
            OPOST,
            tcgetattr,
            TCSAFLUSH,
            tcsetattr,
            termios,
            winsize,
            STDIN_FILENO,
            STDOUT_FILENO,
            TIOCGWINSZ };
use crate::runtime::{ data_structures::value::ToValue,
                      error::{ self,
                               script_error,
                               script_error_str },
                      interpreter::Interpreter };



static mut ORIGINAL_TERMIOS: Option<termios> = None;
static mut IS_IN_RAW_MODE: bool = false;



pub fn word_term_raw_mode(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let requested_on = interpreter.pop_as_bool()?;

    unsafe
    {
        if requested_on && !IS_IN_RAW_MODE
        {
            let stdin_fd = STDIN_FILENO;
            let mut original_terminos = zeroed();

            if tcgetattr(stdin_fd, &mut original_terminos) == -1
            {
                script_error(interpreter,
                             format!("Could not get terminal mode information: {}",
                                     Error::last_os_error()))?;
            }

            ORIGINAL_TERMIOS = Some(original_terminos);
            let mut raw = original_terminos;

            raw.c_iflag &= !(BRKINT | ICRNL | INPCK | ISTRIP | IXON);
            raw.c_oflag &= !(OPOST);
            raw.c_cflag |= CS8;
            raw.c_lflag &= !(ECHO | ICANON | IEXTEN | ISIG);

            if tcsetattr(stdin_fd, TCSAFLUSH, &raw) == -1
            {
                script_error(interpreter,
                             format!("Could not set terminal mode: {}",
                                     Error::last_os_error()))?;
            }

            IS_IN_RAW_MODE = true;
        }
        else if !requested_on && IS_IN_RAW_MODE
        {
            if let Some(ref original_termios) = ORIGINAL_TERMIOS
            {
                if tcsetattr(STDIN_FILENO, TCSAFLUSH, original_termios) == -1
                {
                    script_error(interpreter,
                                 format!("Could not restore terminal mode: {}",
                                         Error::last_os_error()))?;
                }

                IS_IN_RAW_MODE = false;
            }
            else
            {
                script_error_str(interpreter, "Original terminal mode was not saved.")?;
            }
        }
    }

    Ok(())
}

pub fn word_term_size(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let mut size: winsize = unsafe { zeroed() };
    let result = unsafe { ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut size) };

    if result == -1
    {
        script_error_str(interpreter, "Failed to get the terminal size.")?;
    }

    interpreter.push((size.ws_col as i64).to_value());
    interpreter.push((size.ws_row as i64).to_value());

    Ok(())
}

pub fn word_term_key(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let mut buffer = [0; 1];
    let stdin = stdin();
    let mut handle = stdin.lock();

    loop
    {
        match handle.read_exact(&mut buffer)
        {
            Ok(()) =>
                {
                    let character = buffer[0] as char;
                    interpreter.push(character.to_string().to_value());

                    break;
                },

            Err(ref e) if e.kind() == Interrupted => continue,

            Err(e) => script_error(interpreter, format!("Failed to read from stdin: {}", e))?
        }
    }

    Ok(())
}
