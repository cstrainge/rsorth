
use std::{ collections::HashMap,
           fs::{ remove_file,
                 File,
                 OpenOptions },
           io::{ BufRead, BufReader, BufWriter, Read, Write, Seek, SeekFrom },
           os::unix::net::UnixStream,
           path::Path,
           sync::{ atomic::{ AtomicI64, Ordering },
                   Mutex } };
use lazy_static::lazy_static;
use crate::{ add_native_word,
             runtime::{ data_structures::value::ToValue,
                        error::{ self,
                                 script_error,
                                 script_error_str },
                        interpreter::Interpreter } };



enum FileObject
{
    File(File),
    Stream(UnixStream)
}


lazy_static!
{
    // The counter for generating new IDs.
    static ref FD_COUNTER: AtomicI64 = AtomicI64::new(4);

    // Keep a table to map generated FDs to file structs.
    static ref FILE_TABLE: Mutex<HashMap<i64, FileObject>> = Mutex::new(HashMap::new());
}

fn generate_fd() -> i64
{
    FD_COUNTER.fetch_add(1, Ordering::SeqCst)
}

fn add_file(fd: i64, file: File)
{
    FILE_TABLE.lock().unwrap().insert(fd, FileObject::File(file));
}

fn add_stream(fd: i64, stream: UnixStream)
{
    FILE_TABLE.lock().unwrap().insert(fd, FileObject::Stream(stream));
}

fn get_file(interpreter: &mut dyn Interpreter, fd: i64) -> error::Result<FileObject>
{
    let table = FILE_TABLE.lock().unwrap();
    let file = table.get(&fd);

    match file
    {
        Some(file) =>
            {
                match file
                {
                    FileObject::File(file)     => Ok(FileObject::File(file.try_clone()?)),
                    FileObject::Stream(stream) => Ok(FileObject::Stream(stream.try_clone()?))
                }
            }

        None => script_error(interpreter, format!("File struct for fd {} not found.", fd))
    }
}

fn unregister_file(interpreter: &mut dyn Interpreter, fd: i64) -> error::Result<()>
{
    let mut table = FILE_TABLE.lock().unwrap();

    if !table.contains_key(&fd)
    {
        script_error(interpreter, format!("File struct not found for fd {}.", fd))?;
    }

    table.remove(&fd);

    Ok(())
}

fn flags_to_options(flags: i64) -> OpenOptions
{
    let mut options = OpenOptions::new();

    if flags & 0b0001 != 0
    {
        options.read(true);
    }

    if flags & 0b0010 != 0
    {
        options.write(true);
    }

    options
}



fn word_file_open(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let flags = interpreter.pop_as_int()?;
    let path = interpreter.pop_as_string()?;

    let options = flags_to_options(flags);

    match options.open(path.clone())
    {
        Ok(file) =>
            {
                let fd = generate_fd();

                add_file(fd, file);
                interpreter.push(fd.to_value());
            },

        Err(error) =>
            {
                script_error(interpreter, format!("Could not open file {}: {}", path, error))?;
            }
    }

    Ok(())
}

fn word_file_create(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let flags = interpreter.pop_as_int()?;
    let path = interpreter.pop_as_string()?;

    let mut options = flags_to_options(flags);

    options.create(true);
    options.truncate(true);

    match options.open(path.clone())
    {
        Ok(file) =>
            {
                let fd = generate_fd();

                add_file(fd, file);
                interpreter.push(fd.to_value());
            },

        Err(error) =>
            {
                script_error(interpreter, format!("Could not open file {}: {}", path, error))?;
            }
    }

    Ok(())
}

fn word_file_create_temp_file(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    script_error_str(interpreter, "Create temp file unimplemented.")
}

fn word_file_close(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let fd = interpreter.pop_as_int()?;

    unregister_file(interpreter, fd)?;

    Ok(())
}

fn word_file_delete(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let path = interpreter.pop_as_string()?;

    remove_file(&path)?;

    Ok(())
}

fn word_socket_connect(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let path = interpreter.pop_as_string()?;

    match UnixStream::connect(&path)
    {
        Ok(stream) =>
            {
                let fd = generate_fd();

                add_stream(fd, stream);
                interpreter.push(fd.to_value());
            },

        Err(error) =>
            {
                script_error(interpreter, format!("Failed to connect to socket {}: {}",
                                                  path,
                                                  error))?;
            }
    }

    Ok(())
}

fn word_file_size_read(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let fd = interpreter.pop_as_int()?;
    let file = get_file(interpreter, fd)?;

    match file
    {
        FileObject::File(file) =>
            {
                let metadata = file.metadata()?;
                let size = metadata.len();

                interpreter.push(size.to_value());
            },

        FileObject::Stream(_) =>
            {
                script_error_str(interpreter, "Can not read size of a socket.")?;
            }
    }

    Ok(())
}

fn word_file_exists(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let path = interpreter.pop_as_string()?;

    interpreter.push(Path::new(&path).exists().to_value());
    Ok(())
}

fn word_file_is_open(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let fd = interpreter.pop_as_int()?;
    let file = get_file(interpreter, fd);

    interpreter.push(file.is_ok().to_value());

    Ok(())
}

fn word_file_is_eof(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let fd = interpreter.pop_as_int()?;
    let file = get_file(interpreter, fd)?;

    match file
    {
        FileObject::File(mut file) =>
            {
                let current_pos = file.seek(SeekFrom::Current(0))?;
                let total_size = file.metadata()?.len();

                interpreter.push((current_pos == total_size).to_value());
            },

        FileObject::Stream(_) =>
            {
                script_error_str(interpreter, "Can not eof status of a socket.")?;
            }
    }

    Ok(())
}

fn word_file_read(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    script_error_str(interpreter, "Unimplemented.")
}

fn word_file_read_character(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    fn read<T>(interpreter: &mut dyn Interpreter, reader: &mut BufReader<T>) -> error::Result<()>
        where T: Read
    {
        let mut buffer = [0; 1];

        match reader.read(&mut buffer)
        {
            Ok(0) =>
                {
                    interpreter.push("".to_string().to_value());
                },

            Ok(_) =>
                {
                    interpreter.push(buffer[0].to_string().to_value());
                },

            Err(error) =>
                {
                    return script_error(interpreter,
                                        format!("Could not read from file: {}.", error))
                }
        }

        Ok(())
    }

    let fd = interpreter.pop_as_int()?;
    let file = get_file(interpreter, fd)?;

    match file
    {
        FileObject::File(file)     => read(interpreter, &mut BufReader::new(file)),
        FileObject::Stream(stream) => read(interpreter, &mut BufReader::new(stream)),
    }
}

fn word_file_read_string(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    fn read<T>(interpreter: &mut dyn Interpreter,
               reader: &mut BufReader<T>) -> error::Result<()>
        where T: Read
    {
        let mut string = String::new();

        match reader.read_to_string(&mut string)
        {
            Ok(0) =>
                {
                    interpreter.push("".to_string().to_value());
                },

            Ok(_) =>
                {
                    interpreter.push(string.to_value());
                },

            Err(error) =>
                {
                    return script_error(interpreter,
                                        format!("Could not read from file: {}.", error))
                }
        }

        Ok(())
    }

    let fd = interpreter.pop_as_int()?;
    let file = get_file(interpreter, fd)?;

    match file
    {
        FileObject::File(file)     => read(interpreter, &mut BufReader::new(file)),
        FileObject::Stream(stream) => read(interpreter, &mut BufReader::new(stream)),
    }
}

fn word_file_write(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    fn write<T>(interpreter: &mut dyn Interpreter,
               string: String,
               writer: &mut BufWriter<T>) -> error::Result<()>
        where T: Write
    {
        let bytes = string.into_bytes();

        match writer.write(bytes.as_slice())
        {
            // TODO: Handle partial writes.
            Ok(_) =>
                {
                    Ok(())
                },

            Err(error) =>
                {
                    script_error(interpreter, format!("Could not read from file: {}.", error))
                }
        }
    }

    // TODO: Implement ByteBuffer and better string conversion.
    let fd = interpreter.pop_as_int()?;
    let string = interpreter.pop_as_string()?;
    let file = get_file(interpreter, fd)?;

    match file
    {
        FileObject::File(file)     => write(interpreter, string, &mut BufWriter::new(file)),
        FileObject::Stream(stream) => write(interpreter, string, &mut BufWriter::new(stream)),
    }
}

fn word_file_line_read(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    fn read<T>(interpreter: &mut dyn Interpreter, buffer: &mut BufReader<T>) -> error::Result<()>
        where T: Read
    {
        let mut line = String::new();

        match buffer.read_line(&mut line)
        {
            Ok(0) =>
                {
                    interpreter.push("".to_string().to_value());
                },

            Ok(_) =>
                {
                    let line = line.trim_end_matches(&['\n', '\r'][..]).to_string();
                    interpreter.push(line.to_value());
                },

            Err(error) =>
                {
                    return script_error(interpreter,
                                        format!("Could not read from file: {}.", error))
                }
        }

        Ok(())
    }

    let fd = interpreter.pop_as_int()?;
    let file = get_file(interpreter, fd)?;

    match file
    {
        FileObject::File(file)     => read(interpreter, &mut BufReader::new(file)),
        FileObject::Stream(stream) => read(interpreter, &mut BufReader::new(stream)),
    }
}

fn word_file_line_write(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    fn write<T>(interpreter: &mut dyn Interpreter,
                string: String,
                writer: &mut BufWriter<T>) -> error::Result<()>
        where T: Write
    {
        let bytes = (string + "\n").into_bytes();

        match writer.write(bytes.as_slice())
        {
            // TODO: Handle partial writes.
            Ok(_) =>
                {
                    Ok(())
                },

            Err(error) =>
                {
                    script_error(interpreter, format!("Could not read from file: {}.", error))
                }
        }
    }

    // TODO: Implement better string conversion.
    let fd = interpreter.pop_as_int()?;
    let string = interpreter.pop_as_string()?;
    let file = get_file(interpreter, fd)?;

    match file
    {
        FileObject::File(file)     => write(interpreter, string, &mut BufWriter::new(file)),
        FileObject::Stream(stream) => write(interpreter, string, &mut BufWriter::new(stream)),
    }
}



pub fn register_io_words(interpreter: &mut dyn Interpreter)
{
    add_native_word!(interpreter, "file.open", word_file_open,
        "Open an existing file and return a fd.",
        "path flags -- fd");

    add_native_word!(interpreter, "file.create", word_file_create,
        "Create/open a file and return a fd.",
        "path flags -- fd");

    add_native_word!(interpreter, "file.create.tempfile", word_file_create_temp_file,
        "Create/open an unique temporary file and return it's fd.",
        "flags -- path fd");

    add_native_word!(interpreter, "file.close", word_file_close,
        "Take a fd and close it.",
        "fd -- ");

    add_native_word!(interpreter, "file.delete", word_file_delete,
        "Delete the specified file.",
        "file_path -- ");


    add_native_word!(interpreter, "socket.connect", word_socket_connect,
        "Connect to Unix domain socket at the given path.",
        "path -- fd");


    add_native_word!(interpreter, "file.size@", word_file_size_read,
        "Return the size of a file represented by a fd.",
        "fd -- size");


    add_native_word!(interpreter, "file.exists?", word_file_exists,
        "Does the file at the given path exist?",
        "path -- bool");

    add_native_word!(interpreter, "file.is_open?", word_file_is_open,
        "Is the fd currently valid?",
        "fd -- bool");

    add_native_word!(interpreter, "file.is_eof?", word_file_is_eof,
        "Is the file pointer at the end of the file?",
        "fd -- bool");


    add_native_word!(interpreter, "file.@", word_file_read,
        "Read from a given file.  (Unimplemented.)",
        " -- ");

    add_native_word!(interpreter, "file.char@", word_file_read_character,
        "Read a character from a given file.",
        "fd -- character");

    add_native_word!(interpreter, "file.string@", word_file_read_string,
        "Read a file to a string.",
        "fd -- string");

    add_native_word!(interpreter, "file.!", word_file_write,
        "Write a value as text to a file, unless it's a ByteBuffer.",
        "value fd -- ");


    add_native_word!(interpreter, "file.line@", word_file_line_read,
        "Read a full line from a file.",
        "fd -- string");

    add_native_word!(interpreter, "file.line!", word_file_line_write,
        "Write a string as a line to the file.",
        "string fd -- ");


    add_native_word!(interpreter, "file.r/o",
        |interpreter|
        {
            interpreter.push(0b0001_i64.to_value());
            Ok(())
        },
        "Constant for opening a file as read only.",
        " -- flag");

    add_native_word!(interpreter, "file.w/o",
        |interpreter|
        {
            interpreter.push(0b0010_i64.to_value());
            Ok(())
        },
        "Constant for opening a file as write only.",
        " -- flag");

    add_native_word!(interpreter, "file.r/w",
        |interpreter|
        {
            interpreter.push(0b0011_i64.to_value());
            Ok(())
        },
        "Constant for opening a file for both reading and writing.",
        " -- flag");
}
