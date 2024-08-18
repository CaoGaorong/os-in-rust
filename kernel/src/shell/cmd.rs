
#[derive(Clone, Copy)]
pub enum Cmd {
    Pwd,
    Cd,
    Ps, 
    Ls,
}
impl Cmd {
    // pub fn get_name(&self) -> &str {
    //     match self {
    //         Cmd::Cwd => "cwd",
    //         Cmd::Ps => "ps",
    //         Cmd::Ls => "ls",
    //         Cmd::Cd => "cd",
    //     }
    // }
    pub fn get_by_name(name: &str) -> Option<Self> {
        match name {
            "pwd" => Option::Some(Self::Pwd),
            "cd" => Option::Some(Self::Cd),
            "ps" => Option::Some(Self::Ps),
            "ls" => Option::Some(Self::Ls),
            _ => Option::None,
        }
    }
}



