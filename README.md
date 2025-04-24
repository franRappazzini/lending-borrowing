# Lending & Borrowing DApp with Anchor framework

This project is a backend developed to interact with smart contracts on the Solana blockchain using the **Anchor** framework. It includes a complete setup for automated testing.

## Description

This is a decentralized application (DApp) for lending and borrowing between the SOL and USDC tokens. The DApp utilizes the Pyth oracle (https://pyth.network/) to obtain the price of the tokens.

## Technologies Used

- **Solana**: High-performance blockchain.
- **Anchor**: Framework for developing and deploying smart contracts on Solana.
- **Rust**: Programming language used for smart contracts.
- **Mocha/Chai** (or any testing framework you are using): For automated testing.

## Prerequisites

Before getting started, make sure you have the following components installed:

- [Rust](https://www.rust-lang.org/tools/install)
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools)
- [Anchor CLI](https://book.anchor-lang.com/chapter_2/installation.html)
- Node.js and npm (for testing, if applicable)

## Installation

1. Clone this repository:

   ```bash
   git clone <REPOSITORY_URL>
   cd <PROJECT_NAME>
   ```

2. Install Anchor dependencies:

   ```bash
   anchor build
   ```

3. Configure your Solana network (e.g., devnet):

   ```bash
   solana config set --url https://api.devnet.solana.com
   ```

4. Deploy the program to the network:

   ```bash
   anchor deploy
   ```

## Running Tests

To run the project tests, use the following command:

```bash
anchor test
```

This command will compile the smart contracts, deploy a test environment, and execute the defined tests.

## Project Structure

- [`programs/`](command:_github.copilot.openRelativePath?%5B%7B%22scheme%22%3A%22file%22%2C%22authority%22%3A%22%22%2C%22path%22%3A%22%2FUsers%2Ffranrappazzini%2Fpersonal%2Fsolana%2Fsol%20dev%20bootcamp%202024%2Flending-borrowing%2Fprograms%2F%22%2C%22query%22%3A%22%22%2C%22fragment%22%3A%22%22%7D%5D "/Users/franrappazzini/personal/solana/sol dev bootcamp 2024/lending-borrowing/programs/"): Contains the smart contracts written in Rust.
- [`tests/`](command:_github.copilot.openRelativePath?%5B%7B%22scheme%22%3A%22file%22%2C%22authority%22%3A%22%22%2C%22path%22%3A%22%2FUsers%2Ffranrappazzini%2Fpersonal%2Fsolana%2Fsol%20dev%20bootcamp%202024%2Flending-borrowing%2Ftests%2F%22%2C%22query%22%3A%22%22%2C%22fragment%22%3A%22%22%7D%5D "/Users/franrappazzini/personal/solana/sol dev bootcamp 2024/lending-borrowing/tests/"): Contains the test scripts for interacting with the contracts.
- [`migrations/`](command:_github.copilot.openRelativePath?%5B%7B%22scheme%22%3A%22file%22%2C%22authority%22%3A%22%22%2C%22path%22%3A%22%2FUsers%2Ffranrappazzini%2Fpersonal%2Fsolana%2Fsol%20dev%20bootcamp%202024%2Flending-borrowing%2Fmigrations%2F%22%2C%22query%22%3A%22%22%2C%22fragment%22%3A%22%22%7D%5D "/Users/franrappazzini/personal/solana/sol dev bootcamp 2024/lending-borrowing/migrations/"): Scripts for deploying the contracts.
- [`Anchor.toml`](command:_github.copilot.openRelativePath?%5B%7B%22scheme%22%3A%22file%22%2C%22authority%22%3A%22%22%2C%22path%22%3A%22%2FUsers%2Ffranrappazzini%2Fpersonal%2Fsolana%2Fsol%20dev%20bootcamp%202024%2Flending-borrowing%2FAnchor.toml%22%2C%22query%22%3A%22%22%2C%22fragment%22%3A%22%22%7D%5D "/Users/franrappazzini/personal/solana/sol dev bootcamp 2024/lending-borrowing/Anchor.toml"): Anchor configuration file.

## Contributions

Contributions are welcome! If you would like to contribute, please open an issue or submit a pull request.

## License

This project is licensed under the [MIT License](LICENSE).
