
use clap::{App, Arg};

#[derive(Debug)]
pub struct Args {
    pub token: String,
    pub owner: String,
    pub repository: String,
    pub branch: String,
    pub context: String,
    pub command: String,
}

pub fn parse_args() -> Args {
    let token_key = "token";
    let token_arg = Arg::with_name(token_key)
        .short("t")
        .value_name("TOKEN")
        .required(true)
        .help("Authentication token to connect to GitHub.")
        .takes_value(true);

    let owner_key = "owner";
    let owner_arg = Arg::with_name(owner_key)
        .short("o")
        .value_name("OWNER")
        .required(true)
        .help("Owner of the repository to watch")
        .takes_value(true);

    let repository_key = "repo";
    let repository_arg = Arg::with_name(repository_key)
        .short("r")
        .value_name("REPOSITORY")
        .required(true)
        .help("Name of the repository to watch")
        .takes_value(true);

    let branch_key = "branch";
    let branch_arg = Arg::with_name(branch_key)
        .short("b")
        .value_name("BRANCH")
        .required(true)
        .help("Name of the branch to watch")
        .takes_value(true);

    let context_key = "context";
    let context_arg = Arg::with_name(context_key)
        .short("c")
        .value_name("CONTEXT")
        .required(true)
        .help("Label to differentiate status from other statuses.")
        .takes_value(true);

    let command_key = "command";
    let command_arg = Arg::with_name(command_key)
        .short("e")
        .value_name("COMMAND")
        .required(true)
        .help("Command to run to test a commit.")
        .takes_value(true);

    let matches = App::new("Crane")
        .version("0.1")
        .author("Zach Bray <zachbray@googlemail.com>")
        .about("Watches, builds and updates GitHub statuses.")
        .arg(token_arg)
        .arg(owner_arg)
        .arg(repository_arg)
        .arg(branch_arg)
        .arg(context_arg)
        .arg(command_arg)
        .get_matches();

    Args {
        token: matches.value_of(&token_key).unwrap().to_string(),
        owner: matches.value_of(&owner_key).unwrap().to_string(),
        repository: matches.value_of(&repository_key).unwrap().to_string(),
        branch: matches.value_of(&branch_key).unwrap().to_string(),
        context: matches.value_of(&context_key).unwrap().to_string(),
        command: matches.value_of(&command_key).unwrap().to_string(),
    }
}
