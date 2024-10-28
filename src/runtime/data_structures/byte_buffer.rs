
use std::{ cell::RefCell,
           fmt::{ self,
                  Display,
                  Formatter },
           rc::Rc };
use crate::runtime::data_structures::value::{ DeepClone,
                                              Value,
                                              ToValue };



/// Trait to represent byte buffers.  It uses a cursor to perform reads and writes.  If a read or
/// write would exceed the bounds of the buffer the operation will panic.
///
/// The byte buffer is a mutable buffer of bytes that is meant for use in the creation of binary
/// data where every byte counts.
///
/// The byte buffer is read from and written to in a linear fashion like a stream.  It includes
/// methods for reading and writing integers, floats, (of various sizes,) and strings of constrained
/// sizes.
///
/// This buffer should be most useful for binary data protocols and file formats.
pub trait Buffer
{
    /// Get a reference to the buffer's raw bytes.
    fn bytes(&self) -> &[u8];


    /// Resize the buffer to a new size.  If the new size is larger the buffer will be padded with
    /// zeros.  If the new size is smaller the buffer will be truncated.
    fn resize(&mut self, new_size: usize);

    /// Get the length of the buffer.
    fn len(&self) -> usize;


    /// Get the current cursor position in the buffer.
    fn position(&self) -> usize;
    // position_ptr


    /// Set the cursor position in the buffer.  If the position is greater than the buffer size the
    /// operation will panic.
    fn set_position(&mut self, position: usize);

    /// Increment the cursor position by a given amount.  If the new position is greater than the
    /// buffer size the operation will panic.
    fn increment_position(&mut self, increment: usize);


    /// Write an integer to the buffer.  The integer will be written in little endian format.
    ///
    /// The byte size must be 1, 2, 4, or 8.  If the byte size is not one of these values the
    /// operation will panic.
    ///
    /// If the write would exceed the bounds of the buffer the operation will panic.
    fn write_int(&mut self, byte_size: usize, value: i64);

    /// Read an integer from the buffer.  The integer will be read in little endian format.
    ///
    /// The byte size must be 1, 2, 4, or 8.  If the byte size is not one of these values the
    /// operation will panic.
    ///
    /// If the read would exceed the bounds of the buffer the operation will panic.
    fn read_int(&mut self, byte_size: usize, is_signed: bool) -> i64;


    /// Write a float to the buffer.  The float will be written in little endian format.
    ///
    /// The byte size must be 4 or 8.  If the byte size is not one of these values the operation
    /// will panic.
    ///
    /// If the write would exceed the bounds of the buffer the operation will panic.
    fn write_float(&mut self, byte_size: usize, value: f64);

    /// Read a float from the buffer.  The float will be read in little endian format.
    ///
    /// The byte size must be a 4 or 8.  If the byte size is not one of these values the operation
    /// will panic.
    ///
    /// If the read would exceed the bounds of the buffer the operation will panic.
    fn read_float(&mut self, byte_size: usize) -> f64;


    /// Write a string to the buffer.  If the string is larger than the given size, it will be
    /// truncated.  If the string is smaller than the given size, it will be padded with zeros.
    ///
    /// If the write would exceed the bounds of the buffer the operation will panic.
    fn write_string(&mut self, max_size: usize, value: &String);

    /// Read a string from the buffer.  The string will be read up to the given size.  If the string
    /// is smaller than the given size it will be terminated with a zero byte.
    fn read_string(&mut self, max_size: usize) -> String;
}



/// A concrete ByteBuffer data structure.  It uses a cursor to perform reads and writes.  If a read
/// or write would exceed the bounds of the buffer the operation will panic.
///
/// The byte buffer is a mutable buffer of bytes that is meant for use in the creation of binary
/// data where every byte counts.
///
/// The byte buffer is read from and written to in a linear fashion like a stream.  It includes
/// methods for reading and writing integers, floats, (of various sizes,) and strings of constrained
/// sizes.
///
/// This buffer should be most useful for binary data protocols and file formats.
#[derive(Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct ByteBuffer
{
    buffer: Vec<u8>,
    current_position: usize
}



/// A reference counted pointer to a byte buffer.
pub type ByteBufferPtr = Rc<RefCell<ByteBuffer>>;



impl Buffer for ByteBuffer
{
    fn bytes(&self) -> &[u8]
    {
        &self.buffer
    }

    fn resize(&mut self, new_size: usize)
    {
        self.buffer.resize(new_size, 0);

        if self.current_position >= new_size
        {
            self.current_position = new_size - 1;
        }
    }

    fn len(&self) -> usize
    {
        self.buffer.len()
    }

    fn position(&self) -> usize
    {
        self.current_position
    }

    // position_ptr

    fn set_position(&mut self, position: usize)
    {
        if position > self.buffer.len()
        {
            panic!("Attempted to set position to {} in a buffer of size {}.",
                   position,
                   self.buffer.len());
        }

        self.current_position = position;
    }

    fn increment_position(&mut self, increment: usize)
    {
        self.set_position(self.current_position + increment);
    }

    // data_ptr
    // data_ptr_mut

    fn write_int(&mut self, byte_size: usize, value: i64)
    {
        let bytes = match byte_size
            {
                1 => value.to_le_bytes()[0..1].to_vec(),
                2 => value.to_le_bytes()[0..2].to_vec(),
                4 => value.to_le_bytes()[0..4].to_vec(),
                8 => value.to_le_bytes()[0..8].to_vec(),
                _ => panic!("Invalid byte size for integer write {}.", byte_size)
            };

        let position = self.current_position;

        self.increment_position(byte_size);
        self.buffer[position..position + byte_size].copy_from_slice(&bytes);
    }

    fn read_int(&mut self, byte_size: usize, is_signed: bool) -> i64
    {
        let position = self.current_position;

        self.increment_position(byte_size);

        match byte_size
        {
            1 =>
                {
                    let mut bytes = [0; 1];

                    bytes.copy_from_slice(&self.buffer[position..position + 1]);
                    bytes[0] as i64
                },
            2 =>
                {
                    let mut bytes = [0; 2];

                    bytes.copy_from_slice(&self.buffer[position..position + 2]);

                    if is_signed
                    {
                        i16::from_le_bytes(bytes) as i64
                    }
                    else
                    {
                        u16::from_le_bytes(bytes) as i64
                    }
                },

            4 =>
                {
                    let mut bytes = [0; 4];

                    bytes.copy_from_slice(&self.buffer[position..position + 4]);

                    if is_signed
                    {
                        i32::from_le_bytes(bytes) as i64
                    }
                    else
                    {
                        u32::from_le_bytes(bytes) as i64
                    }
                },

            8 =>
                {
                    let mut bytes = [0; 8];

                    bytes.copy_from_slice(&self.buffer[position..position + 8]);

                    if is_signed
                    {
                        i64::from_le_bytes(bytes)
                    }
                    else
                    {
                        u64::from_le_bytes(bytes) as i64
                    }
                },

            _ => panic!("Invalid byte size for integer read {}.", byte_size)
        }
    }

    fn write_float(&mut self, byte_size: usize, value: f64)
    {
        let bytes = match byte_size
            {
                4 => (value as f32).to_le_bytes()[0..4].to_vec(),
                8 => value.to_le_bytes()[0..8].to_vec(),
                _ => panic!("Invalid byte size for integer write {}.", byte_size)
            };

        let position = self.current_position;

        self.increment_position(byte_size);
        self.buffer[position..position + byte_size].copy_from_slice(&bytes);
    }

    fn read_float(&mut self, byte_size: usize) -> f64
    {
        let position = self.current_position;

        self.increment_position(byte_size);

        match byte_size
        {
            4 =>
                {
                    let mut bytes = [0; 4];

                    bytes.copy_from_slice(&self.buffer[position..position + 4]);
                    f32::from_le_bytes(bytes) as f64
                },

            8 =>
                {
                    let mut bytes = [0; 8];

                    bytes.copy_from_slice(&self.buffer[position..position + 8]);
                    f64::from_le_bytes(bytes)
                },

            _ => panic!("Invalid byte size for integer read {}.", byte_size)
        }
    }

    fn write_string(&mut self, max_size: usize, value: &String)
    {
        let bytes = value.as_bytes();
        let write_bytes = bytes.len().min(max_size);

        let position = self.current_position;
        self.increment_position(max_size);

        self.buffer[position..position + write_bytes].copy_from_slice(&bytes[0..write_bytes]);

        if write_bytes < max_size
        {
            for i in position..position + max_size - write_bytes
            {
                self.buffer[i] = 0;
            }
        }
    }

    fn read_string(&mut self, max_size: usize) -> String
    {
        let position = self.current_position;
        self.increment_position(max_size);

        let bytes = &self.buffer[position..position + max_size];
        let end = bytes.iter().position(|&byte| byte == 0).unwrap_or(max_size);

        String::from_utf8_lossy(&bytes[0..end]).to_string()
    }
}


impl DeepClone for ByteBufferPtr
{
    fn deep_clone(&self) -> Value
    {
        let new_buffer = ByteBuffer::new(self.borrow().len());

        new_buffer.borrow_mut().buffer
                               .copy_from_slice(&self.borrow().buffer[0..self.borrow().len()]);
        new_buffer.borrow_mut().current_position = self.borrow().current_position;

        new_buffer.to_value()
    }
}



/// Display the byte buffer in a hex dump format.
impl Display for ByteBuffer
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result
    {
        // Print out the buffer in a hex dump format:
        //
        //           00 01 02 03 04 05 06 07  08 09 0a 0b 0c 0d 0e 0f  | 0123456789abcdef |
        // 00000000  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00  | ................ |
        // 00000010  00 00 00 00 00 00                                 | ......           |

        let bytes = self.bytes();

        writeln!(f,
               "          00 01 02 03 04 05 06 07  08 09 0a 0b 0c 0d 0e 0f  | 0123456789abcdef |")?;

        for ( chunk_index, chunk ) in bytes.chunks(16).enumerate()
        {
            let offset = chunk_index * 16;

            write!(f, "{:08x}  ", offset)?;

            for index in 0..16
            {
                if index == 8
                {
                    write!(f, " ")?;
                }

                if index < chunk.len()
                {
                    write!(f, "{:02x} ", chunk[index])?;
                }
                else
                {
                    write!(f, "   ")?;
                }
            }

            write!(f, " | ")?;

            for &byte in chunk
            {
                if    byte.is_ascii_alphanumeric()
                   || byte.is_ascii_punctuation()
                   || byte == ' ' as u8
                {
                    write!(f, "{}", byte as char)?;
                }
                else
                {
                    write!(f, ".")?;
                }
            }

            for _ in chunk.len()..16
            {
                write!(f, " ")?;
            }

            writeln!(f, " |")?;
        }

        Ok(())
    }
}



impl ByteBuffer
{
    /// Create a new byte buffer reference of the given size.
    pub fn new(new_len: usize) -> ByteBufferPtr
    {
        let mut buffer = Vec::new();

        buffer.resize(new_len, 0);

        let buffer = ByteBuffer
            {
                buffer,
                current_position: 0
            };

        Rc::new(RefCell::new(buffer))
    }
}
