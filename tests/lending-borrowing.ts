import * as anchor from "@coral-xyz/anchor";

import { LendingBorrowing } from "../target/types/lending_borrowing";
import { Program } from "@coral-xyz/anchor";
import { TOKEN_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";

describe("lending-borrowing", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.LendingBorrowing as Program<LendingBorrowing>;

  const liquidationThreshold = bn(0.8);
  const maxLTV = bn(0.7);
  const usdcAddress = new anchor.web3.PublicKey("");

  it("Should initialize banks!", async () => {
    // Add your test here.
    const usdcBankTx = await program.methods
      .initializeBank(liquidationThreshold, maxLTV)
      // .accounts({
      //   mintAccount: usdcAddress,
      // })
      .rpc();

    const solBankTx = await program.methods
      .initializeBank(liquidationThreshold, maxLTV)
      // .accounts({
      //   mintAccount: solAddress,
      // })
      .rpc();

    console.log("usdcBankTx", usdcBankTx);
    console.log("solBankTx", solBankTx);

    // const fetchUsdcBank = await program.account.bank.fetch(usdcAddress);
  });

  it("Should initialize user", async () => {
    const tx = await program.methods.intializeUser(usdcAddress).accounts({}).rpc();

    console.log("user tx", tx);
  });

  it("Should deposit USDC and SOL", async () => {
    const usdcTx = await program.methods
      .depositToken(bn(1000))
      .accounts({
        mintAccount: usdcAddress,
      })
      .rpc();

    const solTx = await program.methods
      .depositToken(bn(1000))
      // .accounts({
      //   mintAccount: solAddress,
      // })
      .rpc();

    console.log("deposit usdc tx", usdcTx);
    console.log("deposit sol tx", solTx);
  });

  it("Should withdraw USDC and SOL", async () => {
    const usdcTx = await program.methods
      .withdrawToken(bn(900))
      .accounts({
        mintAccount: usdcAddress,
      })
      .rpc();

    const solTx = await program.methods
      .withdrawToken(bn(900))
      // .accounts({
      //   mintAccount: solAddress,
      // })
      .rpc();

    console.log("withdraw usdc tx", usdcTx);
    console.log("withdraw sol tx", solTx);
  });
});

function bn(n: number) {
  return new anchor.BN(n);
}
