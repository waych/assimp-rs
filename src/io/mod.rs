//! The `io` module contains functionality for interfacing custom resource loading.
//!
//! Implement the FileIO trait for your custom resource loading, with its open() method returning
//! objects satisfying the File trait.
use std::convert::TryInto;
use std::ffi::CStr;
pub use std::io::SeekFrom;

use ffi::*;

/// Implement this trait along with the associated File type to use custom resource loading using
/// the with_io() loading methods.
pub trait FileIO {
    fn open(&self, file_path: &str, mode: &str) -> Option<Box<dyn File>>;
}

/// Implement this for a given resource to support custom resource loading.
pub trait File {
    /// Should return the number of bytes read, or Err if read unsuccessful.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()>;
    /// Should return the number of bytes written, or Err if write unsuccessful.
    fn write(&mut self, buf: &[u8]) -> Result<usize, ()>;
    fn tell(&mut self) -> u64;
    fn size(&mut self) -> u64;
    fn seek(&mut self, seek_from: SeekFrom) -> Result<(), ()>;
    fn flush(&mut self);
    fn close(&mut self);
}

/// This type allows us to generate C stubs for whatever trait object the user supplies.
struct FileWrapper<T: FileIO> {
    _phantom_t: std::marker::PhantomData<T>,
}

impl<T: FileIO> FileWrapper<T> {
    /// Implementation for aiFileIO::OpenProc.
    unsafe extern "C" fn io_open(
        ai_file_io: *mut aiFileIO,
        file_path: *const ::std::os::raw::c_char,
        mode: *const ::std::os::raw::c_char,
    ) -> *mut aiFile {
        let file_io = Box::leak(Box::from_raw((*ai_file_io).UserData as *mut &mut dyn FileIO));

        let file_path = CStr::from_ptr(file_path).to_str().unwrap_or("Invalid UTF-8 Filename");
        let mode = CStr::from_ptr(mode).to_str().unwrap_or("Invalid UTF-8 Mode");
        let file = match file_io.open(file_path, mode) {
            None => return std::ptr::null_mut(),
            Some(file) => file,
        };

        // Take the returned file, and double box it here so that it can be converted to a single
        // raw pointer.
        let double_box = Box::new(file);
        let user_data = Box::into_raw(double_box) as *mut i8;
        let ai_file = aiFile {
            ReadProc: Some(Self::io_read),
            WriteProc: Some(Self::io_write),
            TellProc: Some(Self::io_tell),
            FileSizeProc: Some(Self::io_size),
            SeekProc: Some(Self::io_seek),
            FlushProc: Some(Self::io_flush),
            UserData: user_data,
        };
        Box::into_raw(Box::new(ai_file))
    }

    /// Implementation for aiFileIO::CloseProc.
    unsafe extern "C" fn io_close(_ai_file_io: *mut aiFileIO, ai_file: *mut aiFile) {
        // Given that this is close, we are careful to not leak, but instead drop the file when we
        // exit this scope.
        let mut file: Box<Box<dyn File>> = Box::from_raw((*ai_file).UserData as *mut Box<dyn File>);
        file.close();
    }
    unsafe fn get_file<'a>(ai_file: *mut aiFile) -> &'a mut Box<dyn File> {
        Box::leak(Box::from_raw((*ai_file).UserData as *mut Box<dyn File>))
    }
    unsafe extern "C" fn io_read(
        ai_file: *mut aiFile,
        buffer: *mut std::os::raw::c_char,
        size: size_t,
        count: size_t,
    ) -> size_t {
        let file = Self::get_file(ai_file);
        let mut buffer =
            std::slice::from_raw_parts_mut(buffer as *mut u8, (size * count).try_into().unwrap());
        if size == 0 {
            panic!("Size 0 is invalid");
        }
        if count == 0 {
            panic!("Count 0 is invalid");
        }
        if size > std::usize::MAX as u64 {
            panic!("huge read size not supported");
        }
        let size = size as usize;
        if size == 1 {
            // This looks like a memcpy.
            if count > std::usize::MAX as u64 {
                panic!("huge read not supported");
            }
            let count = count as usize;

            let (buffer, _) = buffer.split_at_mut(count);
            match file.read(buffer) {
                Ok(size) => size as u64,
                Err(_) => std::u64::MAX,
            }
        } else {
            // We have to copy in strides. Implement this by looping for each object and tally the
            // count of full objects read.
            let mut total: u64 = 0;
            for _ in 0..count {
                let split = buffer.split_at_mut(size as usize);
                buffer = split.1;
                let bytes_read = match file.read(split.0) {
                    Err(_) => break,
                    Ok(bytes_read) => bytes_read,
                };
                if bytes_read != size {
                    break;
                }
                total = total + 1;
            }
            total
        }
    }
    unsafe extern "C" fn io_write(
        ai_file: *mut aiFile,
        buffer: *const std::os::raw::c_char,
        size: size_t,
        count: size_t,
    ) -> size_t {
        let file = Self::get_file(ai_file);
        let mut buffer =
            std::slice::from_raw_parts(buffer as *mut u8, (size * count).try_into().unwrap());
        if size == 0 {
            panic!("Write of size 0");
        }
        if count == 0 {
            panic!("Write of count 0");
        }
        if size > std::usize::MAX as u64 {
            panic!("huge write size not supported");
        }
        let size = size as usize;
        if size == 1 {
            if count > std::usize::MAX as u64 {
                panic!("huge write not supported");
            }
            let count = count as usize;

            let (buffer, _) = buffer.split_at(count);
            match file.write(buffer) {
                Ok(size) => size as u64,
                Err(_) => std::u64::MAX,
            }
        } else {
            // Write in strides. Implement this by looping for each object and tally the
            // count of full objects written.
            let mut total: u64 = 0;
            for _ in 0..count {
                let split = buffer.split_at(size as usize);
                buffer = split.1;
                let bytes_written = match file.write(split.0) {
                    Err(_) => break,
                    Ok(bytes_written) => bytes_written,
                };
                if bytes_written != size {
                    break;
                }
                total = total + 1;
            }
            total
        }
    }
    unsafe extern "C" fn io_tell(ai_file: *mut aiFile) -> size_t {
        let file = Self::get_file(ai_file);
        file.tell()
    }
    unsafe extern "C" fn io_size(ai_file: *mut aiFile) -> size_t {
        let file = Self::get_file(ai_file);
        file.size()
    }
    unsafe extern "C" fn io_seek(ai_file: *mut aiFile, pos: size_t, origin: aiOrigin) -> aiReturn {
        let file = Self::get_file(ai_file);
        let seek_from = match origin {
            autogenerated_assimp_sys::aiOrigin_aiOrigin_SET => SeekFrom::Start(pos),
            autogenerated_assimp_sys::aiOrigin_aiOrigin_CUR => SeekFrom::Current(pos as i64),
            autogenerated_assimp_sys::aiOrigin_aiOrigin_END => SeekFrom::End(pos as i64),
            _ => panic!("Assimp passed invalid origin"),
        };
        match file.seek(seek_from) {
            Ok(()) => 0,
            Err(()) => autogenerated_assimp_sys::aiReturn_aiReturn_FAILURE,
        }
    }
    unsafe extern "C" fn io_flush(ai_file: *mut aiFile) {
        let file = Self::get_file(ai_file);
        file.flush();
    }
}

/// Returns a constructed aiFileIO that can be used with assimp.
/// Now that while this can be copied, the lifetime of the UserData must span the use of this
/// aiFileIO object.
pub fn wrap_file_io<T>(file_io: &T) -> aiFileIO
where
    T: FileIO,
{
    let trait_obj: &dyn FileIO = file_io;
    let user_data = Box::into_raw(Box::new(trait_obj)) as *mut i8;
    aiFileIO {
        OpenProc: Some(FileWrapper::<T>::io_open),
        CloseProc: Some(FileWrapper::<T>::io_close),
        UserData: user_data,
    }
}
