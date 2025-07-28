pub trait EditArgsBuilder {
    /// Opens the editor with the three files
    fn solution_command(
        &self,
        puzzlefile: &str,
        examplefile: &str,
        libfile: &str,
        inputfile: &str,
    ) -> Vec<String>;
    fn files_command(&self, files: Vec<&str>) -> Vec<String>;
}

pub struct VimArgs {}

impl EditArgsBuilder for VimArgs {
    fn solution_command(
        &self,
        puzzlefile: &str,
        examplefile: &str,
        libfile: &str,
        inputfile: &str,
    ) -> Vec<String> {
        let mut args = Vec::new();
        args.push(libfile.to_string());
        args.push(inputfile.to_string());
        args.push(format!("+vsplit {}", examplefile));
        args.push(format!("+split {}", puzzlefile));
        args
    }
    fn files_command(&self, files: Vec<&str>) -> Vec<String> {
        files.iter().map(|file| file.to_string()).collect()
    }
}

pub struct GenericArgs {}

impl EditArgsBuilder for GenericArgs {
    fn solution_command(
        &self,
        puzzlefile: &str,
        examplefile: &str,
        libfile: &str,
        inputfile: &str,
    ) -> Vec<String> {
        self.files_command(vec![puzzlefile, examplefile, libfile, inputfile])
    }
    fn files_command(&self, files: Vec<&str>) -> Vec<String> {
        files.iter().map(|file| file.to_string()).collect()
    }
}

pub type ArgsBuilder = Box<dyn EditArgsBuilder>;
