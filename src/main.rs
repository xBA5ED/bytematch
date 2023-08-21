use ethers::{
    providers::{Http, Middleware, Provider},
    types::{ActionType, Address, Create, CreateResult, Trace, H256},
};
use eyre::Result;
use which::which;
use std::{env, path::{PathBuf, Path}, process::Command, sync::Arc};
use spinoff::{Spinner, spinners, Color};
use clap::Parser;
use std::str;
use interactive_clap::{ResultFromCli, ToCliArgs};

#[derive(Parser, Debug, interactive_clap::InteractiveClap)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Transaction hash in which the contract was deployed
    #[interactive_clap(long)]
    transaction: String,

    /// Address of the contract that should be checked
    #[interactive_clap(long)]
    contract_address: String,

    /// Git url of the repository to check against
    #[interactive_clap(long)]
    git: String,

    /// Optional: commit hash of the git repo
    #[interactive_clap(long)]
    commit: String,

    /// Name of the contract (in the git repository) to check against
    #[interactive_clap(long)]
    contract_name: String,

    /// HTTP RPC url (has to support `trace` calls)
    #[interactive_clap(long)]
    rpc: String,
}

#[tokio::main]
async fn main() -> Result<()> { 
    let mut cli_args = Args::parse();

    let context = ();
    let args = <Args as interactive_clap::FromCli>::from_cli(Some(cli_args.clone()), context);
    match args {
        ResultFromCli::Ok(interactive_args) => {
            cli_args = interactive_args;

            println!(
                "Your arguments:  {}",
                shell_words::join(&cli_args.to_cli_args())
            );
        },
        ResultFromCli::Back => {
            return Ok(());
        }
        ResultFromCli::Cancel(_) => {
            return Ok(());
        }
        ResultFromCli::Err(_, err) => {
            return Err(err);
        }
    }

    // The deployment transaction
    let tx_hash =
        cli_args.clone().transaction.unwrap().parse::<H256>()?;

    // The contract to verify
    let contract = cli_args.contract_address.unwrap().parse::<Address>()?;

    // Build the RPC client
    let client = Provider::<Http>::try_from(cli_args.rpc.unwrap())?;
    let client = Arc::new(client);

    // Could be set to Some("") instead of None, if thats the case we force it to be None
    let mut commit: Option<String> = None;
    if let Some(hash) = cli_args.commit{
        if hash != "" {
            commit = Some(hash);
        }
    }

    let mut spinner = Spinner::new(spinners::Dots, "Fetching traces from the transaction", Color::Blue); 

    // Get the trace call to the contract
    let trace_result = client.trace_transaction(tx_hash).await?;

    // Look through the trace call to find a `CREATE` call
    let create_trace: Vec<&Trace> = trace_result
        .iter()
        .filter(|trace_item| {
            if trace_item.action_type != ActionType::Create {
                return false;
            }

            // For some reason has no result type
            if trace_item.result.is_none() {
                return false;
            }

            // Check that this is the correct address
            if let ethers::types::Res::Create(CreateResult {
                gas_used: _,
                code: _,
                address,
            }) = trace_item.result.clone().unwrap()
            {
                return address == contract;
            }

            // It was not the correct address
            return false;
        })
        .collect();

    // The number of items matching should never be more than `1`
    if create_trace.len() != 1 {
        // TODO: Error
        println!(
            "An unexpected amount of traces were found, {} traces found",
            create_trace.len()
        );
    }

    spinner.update(spinners::Dots, "Cloning project and installing dependencies", Color::Blue);

    // Get a temp folder where we can clone the project to
    let tmp_folder = &mut env::temp_dir();
    tmp_folder.push(cli_args.contract_name.clone().unwrap());

    // Clone and configure the project
    let project_path = configure_project(tmp_folder, String::from(cli_args.git.unwrap()), commit)?;

    spinner.update(spinners::Dots, "Compiling contract", Color::Blue);

    // Use forge inspect to build the bytecode and get the result
    let compile_output = Command::new("forge")
            .args(["inspect", "--force", cli_args.contract_name.unwrap().as_str(), "bytecode"])
            .current_dir(project_path.clone())
            .output()?;

    let compile_init: String = match str::from_utf8(&compile_output.stdout) {
        Ok(v) => remove_metadata(v.to_string()),
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };

    let trace_init: String;
    if let ethers::types::Action::Create(Create { init, .. }) = create_trace[0].action.clone() {
        trace_init = remove_metadata(init.to_string());
    } else {
        panic!("Could not find trace!");
    }

    spinner.stop();

    // Compare the two results
    if compile_init ==  trace_init{
        println!("Matching contract deployment!")
    } else {
        println!("Did not match")
    }


    if false {
        // Check that it contains no selfdestruct
        // if it does, display a warning

        // Check that it contains no delegatecall
        // if it does, display a warning
    }

    Ok(())
}

fn remove_metadata(
    bytecode: String, 
) -> String {
    // Strip all metadata after the metadata delimiter
    if let Some(index) = bytecode.rfind("a264"){
        return bytecode.clone().split_at(index).0.to_string();
    }

    return bytecode;
}

/**
 * Clones and configures a project ready to be compiled, installs needed dependencies such as npm packages and git submodules
 */
fn configure_project(
    tmp_folder: &mut PathBuf,
    git_url: String,
    commit: Option<String>,
) -> Result<PathBuf> {
    // If a commit hash is set we append it to the path
    if let Some(hash) = commit.clone() {
        tmp_folder.push(hash.clone());
    }

    // Clone the repository
    Command::new("git")
        .args(["clone", &git_url, tmp_folder.to_str().unwrap()])
        .output()?;

    // Checkout to the commit hash
    if let Some(hash) = commit { 
        Command::new("git")
            .args(["checkout", &hash])
            .current_dir(tmp_folder.clone())
            .output()?;
    }
    
    // Check if "package.json" exists
    let mut packages_path = tmp_folder.clone();
    packages_path.push("package.json");
    if Path::new(&packages_path).exists() {
        // Install NPM packages
        if which("yarn").is_ok() {
            // Install using yarn
            Command::new("yarn")
                .args(["install"])
                .current_dir(tmp_folder.clone())
                .output()?;
        } else if which("npm").is_ok() {
            // Install using NPM
            Command::new("npm")
                .args(["install"])
                .current_dir(tmp_folder.clone())
                .output()?;
        } else {
            // TODO: error
        }
    }

    // Check if "foundry.toml" exists
    // println!("Output is {:?}", output);
    let mut foundry_toml_path = tmp_folder.clone();
    foundry_toml_path.push("foundry.toml");
    if Path::new(&foundry_toml_path).exists() {
        // Install git submodules
        if which("forge").is_ok() {
            Command::new("forge")
                .args(["install"])
                .current_dir(tmp_folder.clone())
                .output()?;
        } else {
            // TODO: error
        }
    }

    // Return the path
    Ok(tmp_folder.clone())
}
