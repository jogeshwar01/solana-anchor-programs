import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Basics } from "../target/types/basics";
import { assert } from "chai";

describe("basics", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.MyProject as Program<Basics>;

  it("Is initialized!", async () => {
    // Create a signer account
    const signer = anchor.web3.Keypair.generate();

    // Airdrop 1 SOL to the signer account
    const provider = anchor.getProvider();
    const connection = provider.connection;
    await connection.requestAirdrop(
      signer.publicKey,
      anchor.web3.LAMPORTS_PER_SOL
    );

    // Generate the new account that will be created
    const newAccount = anchor.web3.Keypair.generate();

    // Get the system program, which is required for account creation
    // const systemProgram = anchor.web3.SystemProgram.programId;

    await connection.confirmTransaction(
      await connection.requestAirdrop(
        signer.publicKey,
        anchor.web3.LAMPORTS_PER_SOL
      )
    );

    const tx = await program.methods
      .initialize(new anchor.BN(1))
      .accounts({
        newAccount: newAccount.publicKey,
        signer: signer.publicKey,
      })
      .signers([signer, newAccount]) // need newAccount as we created it beforehand (not init by anchor)
      .rpc();

    console.log("Your transaction signature", tx);

    const newAccountData = await program.account.newAccount.fetch(
      newAccount.publicKey
    );

    assert.strictEqual(
      newAccountData.data.toNumber(),
      1,
      "The data in new_account should be 1"
    );
  });
});
