
/// The ByteBuffer data structure.  The byte buffer is a mutable buffer of bytes that is meant for
/// use in the creation of binary data where every byte counts.
///
/// The byte buffer is read from and written to in a linear fashion like a stream.  It includes
/// methods for reading and writing integers, floats, (of various sizes,) and strings of constrained
/// sizes.
///
/// This buffer should be most useful for binary data protocols and file formats.
pub struct _ByteBuffer
{
    buffer: Vec<u8>
}
