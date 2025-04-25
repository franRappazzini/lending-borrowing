import { BN, Program } from "@coral-xyz/anchor";
import { Connection, PublicKey } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createAccount, mintTo } from "@solana/spl-token";

import { BankrunContextWrapper } from "../bankrun-utils/bankrunConnection";
import { BankrunProvider } from "anchor-bankrun";
import IDL from "../target/idl/lending_borrowing.json";
import { LendingBorrowing } from "../target/types/lending_borrowing";
import { PythSolanaReceiver } from "@pythnetwork/pyth-solana-receiver";
import { createMint } from "spl-token-bankrun";
import { startAnchor } from "solana-bankrun";

describe("Lending & Borrowing Program", async () => {
  const pyth = new PublicKey("7UVimffxr9o0w1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE"); // [?]
  const devnetConnection = new Connection("https://api.devnet.solana.com");
  const accountInfo = await devnetConnection.getAccountInfo(pyth);

  const context = await startAnchor(
    "",
    [
      {
        name: "lending_borrowing",
        programId: new PublicKey(IDL.address),
      },
    ],
    [{ address: pyth, info: accountInfo }]
  );

  const provider = new BankrunProvider(context);

  const bankrunContextWrapper = new BankrunContextWrapper(context);
  const bankrunConnection = bankrunContextWrapper.connection.toConnection();

  //   const SOL_PRICE_FEED_ID = "7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE";
  const SOL_PRICE_FEED_ID = "ef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d";
  //   const USDC_PRICE_FEED_ID = "Dpw1EAVrSB1ibxiDQyTAW6Zip3J4Btk2x4SgApQCeFbX";
  const USDC_PRICE_FEED_ID = "eaa020c61cc479712813461ce153894a96a6c00b21ed0cfc2798d1f9a9e9c94a";

  const pythSolanaReceiver = new PythSolanaReceiver({
    connection: bankrunConnection,
    wallet: provider.wallet,
  });

  const solUsdPriceFeedAccount = pythSolanaReceiver.getPriceFeedAccountAddress(
    0,
    SOL_PRICE_FEED_ID
  );

  const usdcUsdPriceFeedAccount = pythSolanaReceiver.getPriceFeedAccountAddress(
    0,
    USDC_PRICE_FEED_ID
  );

  context.setAccount(solUsdPriceFeedAccount, accountInfo);
  context.setAccount(usdcUsdPriceFeedAccount, accountInfo);

  const program = new Program<LendingBorrowing>(IDL as LendingBorrowing, provider);

  const bankClient = context.banksClient;
  const signer = provider.wallet.payer;

  const mintUSDC = await createMint(bankrunConnection, signer, signer.publicKey, null, 9);
  const mintSOL = await createMint(bankrunConnection, signer, signer.publicKey, null, 9);

  const [bankUsdcAccount] = PublicKey.findProgramAddressSync(
    [Buffer.from("treasury"), mintUSDC.toBuffer()],
    program.programId
  );
  const [bankSolAccount] = PublicKey.findProgramAddressSync(
    [Buffer.from("treasury"), mintSOL.toBuffer()],
    program.programId
  );

  it("Should initialize and fund Bank accounts", async () => {
    const initUsdcBankTx = await program.methods
      .initializeBank(bn(1), bn(1))
      .accounts({ mintAccount: mintUSDC, tokenProgram: TOKEN_PROGRAM_ID })
      .rpc();

    console.log("USDC Bank Account tx signature:", initUsdcBankTx);

    const initSolBankTx = await program.methods
      .initializeBank(bn(1), bn(1))
      .accounts({ mintAccount: mintSOL, tokenProgram: TOKEN_PROGRAM_ID })
      .rpc();

    console.log("SOL Bank Account tx signature:", initSolBankTx);

    const amount = 10_000 * 10 ** 9;

    const usdcMintedTx = await mintTo(
      bankrunConnection,
      signer,
      mintUSDC,
      bankUsdcAccount,
      signer,
      amount
    );
    console.log("Minted USDC tx signature:", usdcMintedTx);

    const solMintedTx = await mintTo(
      bankrunConnection,
      signer,
      mintSOL,
      bankSolAccount,
      signer,
      amount
    );
    console.log("Minted SOL tx signature:", solMintedTx);
  });

  it("Should initialize User accounts and token accounts", async () => {
    const initUserUsdcTx = await program.methods.intializeUser(mintUSDC).rpc();
    console.log("User USDC Account tx signature:", initUserUsdcTx);

    const initUserSolTx = await program.methods.intializeUser(mintSOL).rpc();
    console.log("User SOL Account tx signature:", initUserSolTx);

    const usdcTokenAccount = await createAccount(
      bankrunConnection,
      signer,
      mintUSDC,
      signer.publicKey
    );

    const amount = 1_000 * 10 ** 9;

    const usdcMintedTx = await mintTo(
      bankrunConnection,
      signer,
      mintUSDC,
      usdcTokenAccount,
      signer,
      amount
    );

    console.log("Minted USDC User tx signature:", usdcMintedTx);

    const solTokenAccount = await createAccount(
      bankrunConnection,
      signer,
      mintSOL,
      signer.publicKey
    );

    const solMintedTx = await mintTo(
      bankrunConnection,
      signer,
      mintSOL,
      solTokenAccount,
      signer,
      amount
    );
    console.log("Minted SOL User tx signature:", solMintedTx);
  });

  it("Should deposit", async () => {
    const depositUSDCTx = await program.methods
      .depositToken(bn(1000 * 10 ** 9))
      .accounts({
        mintAccount: mintUSDC,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    console.log("Deposit USDC tx signature:", depositUSDCTx);
  });

  it("Should borrow", async () => {
    const borrowSolTx = await program.methods
      .borrowToken(bn(20 * 10 ** 9))
      .accounts({
        mintAccount: mintSOL,
        tokenProgram: TOKEN_PROGRAM_ID,
        priceUpdate: solUsdPriceFeedAccount,
      })
      .rpc();

    console.log("Borrow SOL tx signature:", borrowSolTx);
  });

  it("Should repay", async () => {
    const repaySolTx = await program.methods
      .repayToken(bn(20 * 10 ** 9))
      .accounts({
        mintAccount: mintSOL,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    console.log("Repay SOL tx signature:", repaySolTx);
  });

  it("Should withdraw", async () => {
    const withdrawUSDCTx = await program.methods
      .withdrawToken(bn(500 * 10 ** 9))
      .accounts({
        mintAccount: mintUSDC,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    console.log("Withdraw USDC tx signature:", withdrawUSDCTx);
  });
});

function bn(n: number) {
  return new BN(n);
}
