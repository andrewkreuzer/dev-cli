use std::{
    fs::File,
    io::{self, Write},
};

pub fn write_tmp_file<P>(path: P, contents: &str, set_excecute: bool) -> Result<(), io::Error>
where
    P: AsRef<std::path::Path>,
{
    let mut file = File::create(path.as_ref())?;
    file.write_all(contents.as_bytes())?;

    if set_excecute {
        let mut permissions = file.metadata()?.permissions();
        use std::os::unix::fs::PermissionsExt;
        permissions.set_mode(0o755);
        std::fs::set_permissions(path.as_ref(), permissions)?;
    }

    Ok(())
}
