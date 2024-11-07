
use std::{ cell::RefCell,
           fmt::{ self,
                  Display,
                  Formatter },
           os::raw::c_void,
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
    /// Get a pointer to the buffer's raw bytes.
    fn byte_ptr(&self) -> *const c_void;

    /// Get a mutable pointer to the buffer's raw bytes.
    fn byte_ptr_mut(&mut self) -> *mut c_void;

    /// Resize the buffer to a new size.  If the new size is larger the buffer will be padded with
    /// zeros.  If the new size is smaller the buffer will be truncated.
    fn resize(&mut self, new_size: usize);

    /// Get the length of the buffer.
    fn len(&self) -> usize;


    /// Get the current cursor position in the buffer.
    fn position(&self) -> usize;


    /// Get a mutable pointer to the current cursor position in the buffer.
    fn position_ptr_mut(&mut self) -> *mut c_void;


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



impl Display for dyn Buffer
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        // Print out the buffer in a hex dump format:
        //
        //           00 01 02 03 04 05 06 07  08 09 0a 0b 0c 0d 0e 0f  | 01234567 89abcdef |
        // 00000000  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00  | ........ ........ |
        // 00000010  00 00 00 00 00 00                                 | ......            |

        let bytes =
            unsafe
            {
                let ptr = self.byte_ptr();
                let ptr_u8 = ptr as *const u8;

                std::slice::from_raw_parts(ptr_u8, self.len())
            };

        writeln!(f,
              "          00 01 02 03 04 05 06 07  08 09 0a 0b 0c 0d 0e 0f  | 01234567 89abcdef |")?;

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

            for ( index, &byte ) in chunk.iter().enumerate()
            {
                if index == 8
                {
                    write!(f, " ")?;
                }

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

            for index in chunk.len()..16
            {
                if index == 8
                {
                    write!(f, " ")?;
                }

                write!(f, " ")?;
            }

            writeln!(f, " |")?;
        }

        Ok(())
    }
}



/// Generic pointer to a buffer object.
pub type BufferPtr = Rc<RefCell<dyn Buffer>>;



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
    fn byte_ptr(&self) -> *const c_void
    {
        self.buffer.as_ptr() as *const c_void
    }

    fn byte_ptr_mut(&mut self) -> *mut c_void
    {
        self.buffer.as_mut_ptr() as *mut c_void
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

    fn position_ptr_mut(&mut self) -> *mut c_void
    {
        &mut self.current_position as *mut usize as *mut c_void
    }

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
            for i in position + write_bytes..position + max_size
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


/// Deep copy the byte buffer for the Value type.
impl DeepClone for ByteBufferPtr
{
    fn deep_clone(&self) -> Value
    {
        let new_buffer = ByteBuffer::new_ptr(self.borrow().len());

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
        let buffer = self as &dyn Buffer;
        write!(f, "{}", buffer)
    }
}



impl ByteBuffer
{
    /// Create a new byte buffer of the given size.
    pub fn new(new_len: usize) -> ByteBuffer
    {
        let mut buffer = Vec::new();

        buffer.resize(new_len, 0);

        ByteBuffer
            {
                buffer,
                current_position: 0
            }
    }

    /// Create a new byte buffer reference of the given size.
    pub fn new_ptr(new_len: usize) -> ByteBufferPtr
    {
        Rc::new(RefCell::new(ByteBuffer::new(new_len)))
    }
}



/// A concrete implementation of the Buffer trait.  This buffer is a sub-buffer of another buffer
/// and is meant to be used to read and write data from a specific range of the parent buffer.
pub struct SubBuffer
{
    /// The real backing store for this sub-buffer.
    parent: BufferPtr,


    /// The start location of this buffer within the parent buffer.
    start: usize,

    /// The end location of this buffer within the parent buffer.
    end: usize,


    /// This buffer's cursor position within it's allocated range.
    current_position: usize
}


impl Buffer for SubBuffer
{
    fn byte_ptr(&self) -> *const c_void
    {
        let position = self.parent.borrow().position();

        self.parent.borrow_mut().set_position(self.start);
        let ptr = self.parent.borrow().byte_ptr();
        self.parent.borrow_mut().set_position(position);

        ptr
    }

    fn byte_ptr_mut(&mut self) -> *mut c_void
    {
        let position = self.parent.borrow().position();

        self.parent.borrow_mut().set_position(self.start);
        let ptr = self.parent.borrow_mut().byte_ptr_mut();
        self.parent.borrow_mut().set_position(position);

        ptr
    }

    fn resize(&mut self, new_size: usize)
    {
         let new_end = self.start + new_size;

         if new_end > self.parent.borrow().len()
         {
             panic!("Attempted to resize a sub-buffer to a size larger than the parent buffer.");
         }

         self.end = new_end;
    }

    fn len(&self) -> usize
    {
        self.end - self.start
    }

    fn position(&self) -> usize
    {
        self.current_position
    }

    fn position_ptr_mut(&mut self) -> *mut c_void
    {
        let position = self.parent.borrow().position();

        self.parent.borrow_mut().set_position(self.start + self.current_position);
        let ptr = self.parent.borrow_mut().byte_ptr_mut();
        self.parent.borrow_mut().set_position(position);

        ptr
    }

    fn set_position(&mut self, position: usize)
    {
        if position > self.len()
        {
            panic!("Attempted to set position to {} in a buffer of size {}.",
                   position,
                   self.len());
        }

        self.current_position = position;
    }

    fn increment_position(&mut self, increment: usize)
    {
        self.set_position(self.current_position + increment);
    }

    fn write_int(&mut self, byte_size: usize, value: i64)
    {
        {
            let mut parent = self.parent.borrow_mut();
            let position = parent.position();

            parent.set_position(self.start + self.current_position);
            parent.write_int(byte_size, value);
            parent.set_position(position);
        }

        self.increment_position(byte_size);
    }

    fn read_int(&mut self, byte_size: usize, is_signed: bool) -> i64
    {
        let value =
            {
                let mut parent = self.parent.borrow_mut();
                let position = parent.position();

                parent.set_position(self.start + self.current_position);
                let value = parent.read_int(byte_size, is_signed);
                parent.set_position(position);

                value
            };

        self.increment_position(byte_size);

        value
    }

    fn write_float(&mut self, byte_size: usize, value: f64)
    {
        {
            let mut parent = self.parent.borrow_mut();
            let position = parent.position();

            parent.set_position(self.start + self.current_position);
            parent.write_float(byte_size, value);
            parent.set_position(position);
        }

        self.increment_position(byte_size);
    }

    fn read_float(&mut self, byte_size: usize) -> f64
    {
        let value =
            {
                let mut parent = self.parent.borrow_mut();
                let position = parent.position();

                parent.set_position(self.start + self.current_position);
                let value = parent.read_float(byte_size);
                parent.set_position(position);

                value
            };

        self.increment_position(byte_size);

        value
    }

    fn write_string(&mut self, max_size: usize, value: &String)
    {
        {
            let mut parent = self.parent.borrow_mut();
            let position = parent.position();

            parent.set_position(self.start + self.current_position);
            parent.write_string(max_size, value);
            parent.set_position(position);
        }

        self.increment_position(max_size);
    }

    fn read_string(&mut self, max_size: usize) -> String
    {
        let value =
            {
                let mut parent = self.parent.borrow_mut();
                let position = parent.position();

                parent.set_position(self.start + self.current_position);
                let value = parent.read_string(max_size);
                parent.set_position(position);

                value
            };

        self.increment_position(max_size);

        value
    }
}


/// Display the sub-buffer in a hex dump format.
impl Display for SubBuffer
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result
    {
        let buffer = self as &dyn Buffer;
        write!(f, "{}", buffer)
    }
}


impl SubBuffer
{
    /// Create a new sub-buffer from a parent buffer with a specified range inside of that buffer.
    fn new(parent: BufferPtr, start: usize, end: usize) -> SubBuffer
    {
        let parent_len = parent.borrow().len();

        if    start > parent_len
           || end > parent_len
        {
            panic!("Attempted to create a sub-buffer with a range outside of the parent buffer.");
        }

        SubBuffer
            {
                parent,
                start,
                end,
                current_position: 0
            }
    }

    /// Create a new sub-buffer ptr from a parent buffer within a specified range.
    fn new_ptr(parent: BufferPtr, start: usize, end: usize) -> BufferPtr
    {
        Rc::new(RefCell::new(SubBuffer::new(parent, start, end)))
    }
}
