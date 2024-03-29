
use clap::{App, Arg};

#[derive(Debug)]
pub struct Args {
    pub user: String,
    pub token: String,
    pub owner: String,
    pub repository: String,
    pub branch: String,
    pub context: String,
    pub script: String,
    pub region: String,
    pub bucket: String,
}

pub fn parse_args() -> Args {
    let user_key = "user";
    let user_arg = Arg::with_name(user_key)
        .short("u")
        .value_name("GITHUB_USERNAME")
        .required(true)
        .help("User to connect to GitHub as.")
        .takes_value(true);

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
        .value_name("DEFAULT_BRANCH")
        .required(true)
        .help("Name of the default branch to watch (typically master or develop). This must match the default branch on GitHub.")
        .takes_value(true);

    let context_key = "context";
    let context_arg = Arg::with_name(context_key)
        .short("c")
        .value_name("CONTEXT")
        .required(true)
        .help("Label to differentiate status from other statuses.")
        .takes_value(true);

    let script_key = "script";
    let script_arg = Arg::with_name(script_key)
        .short("e")
        .value_name("FILE")
        .required(true)
        .help("Bash script to run to test a commit.")
        .takes_value(true);

    let region_key = "region";
    let region_arg = Arg::with_name(region_key)
        .long(region_key)
        .value_name("AWS_REGION")
        .required(true)
        .help("AWS region of S3 bucket to save build logs.")
        .takes_value(true);

    let bucket_key = "bucket";
    let bucket_arg = Arg::with_name(bucket_key)
        .long(bucket_key)
        .value_name("S3_BUCKET")
        .required(true)
        .help("AWS bucket for build logs.")
        .takes_value(true);

    let matches = App::new("Crane")
        .version("0.1")
        .author("Zach Bray <zachbray@googlemail.com>")
        .about("Watches, builds and updates GitHub statuses.")
        .arg(user_arg)
        .arg(token_arg)
        .arg(owner_arg)
        .arg(repository_arg)
        .arg(branch_arg)
        .arg(context_arg)
        .arg(script_arg)
        .arg(region_arg)
        .arg(bucket_arg)
        .get_matches();

    Args {
        user: matches.value_of(&user_key).unwrap().to_string(),
        token: matches.value_of(&token_key).unwrap().to_string(),
        owner: matches.value_of(&owner_key).unwrap().to_string(),
        repository: matches.value_of(&repository_key).unwrap().to_string(),
        branch: matches.value_of(&branch_key).unwrap().to_string(),
        context: matches.value_of(&context_key).unwrap().to_string(),
        script: matches.value_of(&script_key).unwrap().to_string(),
        region: matches.value_of(&region_key).unwrap().to_string(),
        bucket: matches.value_of(&bucket_key).unwrap().to_string(),
    }
}
