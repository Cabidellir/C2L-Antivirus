//! Narrow, handle-based Windows operations. No privilege elevation or path-based deletion.
use std::{
    fs::{self, File, OpenOptions},
    io,
    os::windows::{
        ffi::OsStrExt,
        fs::{MetadataExt, OpenOptionsExt},
        io::AsRawHandle,
    },
    path::Path,
    ptr,
};
use windows_sys::Win32::{
    Foundation::LocalFree,
    Security::{
        Authorization::{ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1},
        Cryptography::{
            CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN, CryptProtectData, CryptUnprotectData,
        },
        SECURITY_ATTRIBUTES,
    },
    Storage::FileSystem::*,
};

fn wide(value: &std::ffi::OsStr) -> Vec<u16> {
    value.encode_wide().chain(Some(0)).collect()
}
fn err() -> io::Error {
    io::Error::last_os_error()
}

/// Keep every existing directory component from being renamed during a transaction.
/// Reject reparse points, device/UNC paths and alternate streams rather than resolving them.
pub fn lock_parents(path: &Path) -> io::Result<Vec<File>> {
    let text = path.to_string_lossy();
    if !path.is_absolute() || text.starts_with("\\\\") || text.chars().skip(2).any(|c| c == ':') {
        return Err(io::Error::other(
            "Apenas caminhos locais absolutos, sem streams alternativos",
        ));
    }
    let mut guards = Vec::new();
    let mut parents: Vec<_> = path.ancestors().skip(1).collect();
    parents.reverse();
    for parent in parents {
        let file = OpenOptions::new()
            .access_mode(FILE_READ_ATTRIBUTES)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
            .open(parent)?;
        let metadata = file.metadata()?;
        if !metadata.is_dir() || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return Err(io::Error::other("Pasta de ligação/junction não autorizada"));
        }
        guards.push(file);
    }
    Ok(guards)
}

pub fn open_exclusive(path: &Path, delete: bool) -> io::Result<File> {
    let file = OpenOptions::new()
        .access_mode(0x80000000 | if delete { DELETE } else { 0 })
        .share_mode(0)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file()
        || metadata.file_attributes() & (FILE_ATTRIBUTE_REPARSE_POINT | FILE_ATTRIBUTE_ENCRYPTED)
            != 0
    {
        return Err(io::Error::other(
            "Só ficheiros regulares locais sem EFS/links",
        ));
    }
    let mut info = BY_HANDLE_FILE_INFORMATION::default();
    // SAFETY: live File handle and writable correctly sized structure.
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut info) } == 0 {
        return Err(err());
    }
    if info.nNumberOfLinks != 1 {
        return Err(io::Error::other("Ficheiros com hard links não são movidos"));
    }
    // A single unnamed data stream is the only supported shape. Reject overflow too.
    let mut streams = [0_u64; 1024];
    // SAFETY: aligned 8192-byte output buffer; the API writes at most its size.
    if unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle(),
            FileStreamInfo,
            streams.as_mut_ptr().cast(),
            std::mem::size_of_val(&streams) as u32,
        )
    } == 0
    {
        return Err(err());
    }
    // SAFETY: successful call initialized a FILE_STREAM_INFO at an aligned address.
    let stream = unsafe { &*streams.as_ptr().cast::<FILE_STREAM_INFO>() };
    if stream.NextEntryOffset != 0 || stream.StreamNameLength != 14 {
        return Err(io::Error::other(
            "Streams adicionais não suportados; original preservado",
        ));
    }
    Ok(file)
}

/// Called only after durable encrypted copy + journal commit. Deletes this exact handle.
pub fn delete_on_close(file: &File) -> io::Result<()> {
    let info = FILE_DISPOSITION_INFO { DeleteFile: true };
    // SAFETY: valid exclusive DELETE-capable handle, exact immutable input structure.
    if unsafe {
        SetFileInformationByHandle(
            file.as_raw_handle(),
            FileDispositionInfo,
            (&info as *const FILE_DISPOSITION_INFO).cast(),
            std::mem::size_of_val(&info) as u32,
        )
    } == 0
    {
        return Err(err());
    }
    Ok(())
}

/// DPAPI user-bound authenticated encryption. Ciphertext is not an executable image.
pub fn protect(bytes: &[u8], decrypt: bool) -> io::Result<Vec<u8>> {
    let size: u32 = bytes
        .len()
        .try_into()
        .map_err(|_| io::Error::other("Blob demasiado grande"))?;
    let input = CRYPT_INTEGER_BLOB {
        cbData: size,
        pbData: bytes.as_ptr().cast_mut(),
    };
    let mut output = CRYPT_INTEGER_BLOB::default();
    // SAFETY: input is borrowed throughout the synchronous call; DPAPI owns output.
    let ok = unsafe {
        if decrypt {
            CryptUnprotectData(
                &input,
                ptr::null_mut(),
                ptr::null(),
                ptr::null(),
                ptr::null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            )
        } else {
            CryptProtectData(
                &input,
                ptr::null(),
                ptr::null(),
                ptr::null(),
                ptr::null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            )
        }
    };
    if ok == 0 {
        return Err(err());
    }
    // SAFETY: successful DPAPI call returns cbData initialized bytes, freed with LocalFree.
    let result =
        unsafe { std::slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec() };
    unsafe {
        LocalFree(output.pbData.cast());
    }
    Ok(result)
}

/// A new vault is accessible to its owner, SYSTEM and administrators, with no inherited ACEs.
/// Existing vaults are checked for reparses and their root DACL is restricted again. Same-user/admin tampering is outside this boundary.
pub fn create_vault_dir(path: &Path) -> io::Result<()> {
    let _parents = lock_parents(path)?;
    if path.exists()
        && (!fs::symlink_metadata(path)?.is_dir()
            || fs::symlink_metadata(path)?.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0)
    {
        return Err(io::Error::other("Cofre inválido"));
    }
    let sddl = wide(std::ffi::OsStr::new(
        "D:P(A;OICI;FA;;;OW)(A;OICI;FA;;;SY)(A;OICI;FA;;;BA)",
    ));
    let mut descriptor = ptr::null_mut();
    // SAFETY: terminated UTF-16 string and output pointer; freed after CreateDirectoryW.
    if unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl.as_ptr(),
            SDDL_REVISION_1,
            &mut descriptor,
            ptr::null_mut(),
        )
    } == 0
    {
        return Err(err());
    }
    let attrs = SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: descriptor,
        bInheritHandle: 0,
    };
    let name = wide(path.as_os_str());
    let ok = if path.exists() {
        // SAFETY: checked local non-reparse directory, valid self-relative descriptor.
        unsafe {
            windows_sys::Win32::Security::SetFileSecurityW(
                name.as_ptr(),
                windows_sys::Win32::Security::DACL_SECURITY_INFORMATION
                    | windows_sys::Win32::Security::PROTECTED_DACL_SECURITY_INFORMATION,
                descriptor,
            )
        }
    } else {
        unsafe { CreateDirectoryW(name.as_ptr(), &attrs) }
    };
    let error = err();
    unsafe {
        LocalFree(descriptor);
    }
    if ok == 0 { Err(error) } else { Ok(()) }
}

/// Preserve source permissions as evidence; restoration deliberately inherits destination ACLs.
pub fn file_security(file: &File) -> io::Result<Vec<u8>> {
    use windows_sys::Win32::Security::*;
    let flags = OWNER_SECURITY_INFORMATION | GROUP_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION;
    let mut needed = 0;
    // SAFETY: length query with null buffer, valid live file handle.
    unsafe {
        GetKernelObjectSecurity(file.as_raw_handle(), flags, ptr::null_mut(), 0, &mut needed);
    }
    if needed == 0 || needed > 65536 {
        return Err(io::Error::other("Descritor de segurança inválido"));
    }
    let mut bytes = vec![0; needed as usize];
    // SAFETY: buffer exactly sized by the API; descriptor copied as opaque bytes.
    if unsafe {
        GetKernelObjectSecurity(
            file.as_raw_handle(),
            flags,
            bytes.as_mut_ptr().cast(),
            needed,
            &mut needed,
        )
    } == 0
    {
        return Err(err());
    }
    Ok(bytes)
}
