# ByteMatch

ByteMatch is a Rust-based tool that facilitates the comparison of on-chain bytecode with the bytecode produced by a contract's source code in a git repository.

## Features:

- Works with any method of deployment (EOA, `CREATE`, `CREATE2`)
- Supports any chain
- Only requires an RPC
- Support for private git repositories

## Prerequisites:

- Ensure you have `git`, `npm` (or `yarn`) and `forge` binaries installed on your system.
- The RPC node provided must support `trace_` calls.

## Installation:

Clone the project and navigate into the directory:

```bash
git clone https://github.com/xBA5ED/bytematch
cd bytematch
```

## Usage:

Run the tool using the following command:

```bash
cargo run -- [OPTIONS]
```

Options:
- `--transaction`: The transaction hash in which the contract was deployed.
- `--contract-address`: Address of the contract that should be checked.
- `--git`: Git URL of the repository to check against.
- `--commit`: (Optional) Commit hash of the git repo. If not provided, the tool uses the latest commit.
- `--contract-name`: Name of the contract (inside the git repository) to check against.
- `--rpc`: HTTP RPC URL. Must support `trace_` calls.

Or you can just execute `cargo run` and you will enter interactive mode.


## Example:
```bash
cargo run -- \
    --rpc=https://eth.llamarpc.com \
    --git=https://github.com/jbx-protocol/juice-721-delegate \
    --commit=f3137a055221931b17eeb09b0e44af933f0f4e3a \
    --contract-name=JBTiered721DelegateProjectDeployer \
    --contract-address=0x6Ec3F54b45BbCc745974aD8c6A2f0d4C586D2445 \
    --transaction=0xf8973b6aa565155ff312fbcaf716b979e6accd240169ffc0db828cdf91416b2d
```

Upon successful execution, the tool will either display "Matching contract deployment!" or "Did not match" based on the bytecode comparison results.

## Future Enhancements:
- Make interactive mode an argument to enable it (`--i`).
- Warning detection for contracts containing `selfdestruct` or `delegatecall`.
- Improve error handling


## Contributions:

If you'd like to contribute to bytematch, please create a pull request or open an issue to discuss potential changes or improvements.

## License:

MIT
