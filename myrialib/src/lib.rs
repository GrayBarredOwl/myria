use myria::plugin::*;

#[unsafe(no_mangle)]
pub fn load() -> Vec<FuncInfo> {
    vec![
        ("fs.open".into(), RustFunc::new(Some(2), files::open)),
        ("fs.close".into(), RustFunc::new(Some(1), files::close)),
        ("fs.read".into(), RustFunc::new(Some(2), files::read)),
        ("fs.write".into(), RustFunc::new(Some(2), files::write)),
        ("fs.seek".into(), RustFunc::new(Some(3), files::seek)),
        ("fs.tell".into(), RustFunc::new(Some(1), files::tell)),
    ]
}
#[unsafe(no_mangle)]
pub fn name() -> String {
    "std".into()
}

mod files {
    use super::*;
    use myria::obj::PrimType;
    use myria::{r#gen::MyriaErr, obj::Primitive};

    use std::fs::{self, File};
    use std::io::{Read, Seek, SeekFrom, Write};
    use std::mem::ManuallyDrop;
    use std::os::fd::{FromRawFd, IntoRawFd};

    const MODE_READ: i64 = 1;
    const MODE_WRITE: i64 = 2;
    const MODE_APP: i64 = 4;

    const OFFSET_START: i64 = 1;
    const OFFSET_CURRENT: i64 = 2;
    const OFFSET_END: i64 = 4;

    pub fn open(args: Vec<Object>) -> MyriaRes {
        let [path, mode] = &args[..] else {
            unreachable!();
        };
        let path = path.to_string();

        let mode = match &mode.primitive {
            Primitive::Int(val) => *val,
            _ => {
                return Err(MyriaErr::InvalidOperation("Invalid open mode".into()));
            }
        };

        let file = fs::OpenOptions::new()
            .read(mode & MODE_READ != 0)
            .write(mode & MODE_WRITE != 0)
            .append(mode & MODE_APP != 0)
            .create(mode & MODE_WRITE != 0 || mode & MODE_APP != 0)
            .open(path)
            .map_err(|er| MyriaErr::FileError(er.to_string()))?;
        Ok(Object::make_int(file.into_raw_fd() as i64))
    }
    pub fn close(args: Vec<Object>) -> MyriaRes {
        let _ = fd_obj_to_file(&args[0])?;
        Ok(Object::make_null())
    }
    pub fn read(args: Vec<Object>) -> MyriaRes {
        let mut file = ManuallyDrop::new(fd_obj_to_file(&args[0])?);
        let size = match &args[1].primitive {
            Primitive::Int(val) => *val as usize,
            _ => {
                return Err(MyriaErr::InvalidOperation(format!(
                    "std.fs.read called without an int for length: {}",
                    args[1]
                )));
            }
        };

        let mut vec = vec![0; size];

        file.read(&mut vec)
            .map_err(|err| MyriaErr::FileError(err.to_string()))?;

        Ok(Object::make_list(
            vec.into_iter()
                .map(|b| Object::make_char(b as char))
                .collect(),
        ))
    }
    pub fn write(args: Vec<Object>) -> MyriaRes {
        let mut file = ManuallyDrop::new(fd_obj_to_file(&args[0])?);
        let to_write = match &args[1].primitive {
            Primitive::List(val) => val,
            _ => {
                return Err(MyriaErr::InvalidOperation(format!(
                    "std.fs.read called without an int for length: {}",
                    args[1]
                )));
            }
        };
        if !matches!(to_write.ltype, Some(PrimType::Char)) {
            return Err(MyriaErr::InvalidOperation("Must write string".into()));
        }

        let to_write = to_write
            .elems
            .iter()
            .map(|c| match c.primitive {
                Primitive::Char(c) => c,
                _ => unreachable!(),
            })
            .collect::<String>();

        file.write(to_write.as_bytes())
            .map_err(|err| MyriaErr::FileError(err.to_string()))?;

        Ok(Object::make_null())
    }
    pub fn seek(args: Vec<Object>) -> MyriaRes {
        let mut file = ManuallyDrop::new(fd_obj_to_file(&args[0])?);
        let offset_mode = match &args[1].primitive {
            Primitive::Int(val) => *val,
            _ => {
                return Err(MyriaErr::InvalidOperation(format!(
                    "std.fs.seek called without an int for offset_mode: {}",
                    args[1]
                )));
            }
        };
        let offset = match &args[2].primitive {
            Primitive::Int(val) => *val,
            _ => {
                return Err(MyriaErr::InvalidOperation(format!(
                    "std.fs.seek called without an int for offset: {}",
                    args[1]
                )));
            }
        };
        let sk = match offset_mode {
            OFFSET_START => SeekFrom::Start(offset as u64),
            OFFSET_CURRENT => SeekFrom::Current(offset),
            OFFSET_END => SeekFrom::End(offset),
            _ => {
                return Err(MyriaErr::InvalidOperation(
                    "std.fs.seek called with invalid seek mode".into(),
                ));
            }
        };
        let position = file
            .seek(sk)
            .map_err(|err| MyriaErr::FileError(err.to_string()))?;
        Ok(Object::make_int(position as i64))
    }
    pub fn tell(args: Vec<Object>) -> MyriaRes {
        let mut file = ManuallyDrop::new(fd_obj_to_file(&args[0])?);
        Ok(Object::make_int(
            file.stream_position()
                .map_err(|err| MyriaErr::FileError(err.to_string()))? as i64,
        ))
    }

    fn fd_obj_to_file(obj: &Object) -> Result<File, MyriaErr> {
        let fd = match obj.primitive {
            Primitive::Int(fd) => fd as i32,
            _ => {
                return Err(MyriaErr::InvalidOperation(format!(
                    "std.fs.close called with ({})",
                    obj
                )));
            }
        };
        Ok(unsafe { File::from_raw_fd(fd) })
    }
}
