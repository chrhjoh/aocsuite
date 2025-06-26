use std::fs;
use std::io;
use std::path::Path;

//TODO: Add option for templating later
pub fn copy_file_from_template(template_file: &Path, output_file: &Path) -> io::Result<()> {
    let content = fs::read_to_string(template_file)?;
    fs::write(output_file, content)
}
