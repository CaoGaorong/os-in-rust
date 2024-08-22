
#[derive(Clone, Copy)]
pub enum Cmd<'a> {
    Pwd,
    Cd,
    Ps, 
    Ls,
    Clear,
    Mkdir,
    Rmdir,
    Custom(&'a str)
}
impl <'a> Cmd<'a> {
    // pub fn get_name(&self) -> &str {
    //     match self {
    //         Cmd::Cwd => "cwd",
    //         Cmd::Ps => "ps",
    //         Cmd::Ls => "ls",
    //         Cmd::Cd => "cd",
    //     }
    // }
    pub fn get_by_name(name: &'a str) -> Self {
        match name {
            "pwd" => Self::Pwd,
            "cd" => Self::Cd,
            "ps" => Self::Ps,
            "ls" => Self::Ls,
            "clear" => Self::Clear,
            "mkdir" => Self::Mkdir,
            "rmdir" => Self::Rmdir,
            _ => Cmd::Custom(name),
        }
    }
}



