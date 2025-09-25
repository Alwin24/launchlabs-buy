import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { createAssociatedTokenAccountIdempotentInstructionWithDerivation } from "@solana/spl-token";
import {
  ComputeBudgetProgram,
  Keypair,
  PublicKey,
  SystemProgram,
} from "@solana/web3.js";
import { BN } from "bn.js";
import {
  RaydiumLaunchpad,
  IDL as RaydiumLaunchpadIDL,
} from "../idl-types/raydium_launchpad";
import { LaunchlabsBuy } from "../target/types/launchlabs_buy";

describe("launchlabs-buy", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.launchlabsBuy as Program<LaunchlabsBuy>;
  const raydiumProgram = new Program<RaydiumLaunchpad>(
    RaydiumLaunchpadIDL,
    program.provider
  );
  const platformConfig = new PublicKey(
    "BuM6KDpWiTcxvrpXywWFiw45R2RNH8WURdvqoTDV1BW4"
  );

  const baseTokenKeypair = new Keypair();
  const baseTokenMint = baseTokenKeypair.publicKey;

  const quoteTokenMint = new PublicKey(
    "USD1ttGY1N17NEEHLmELoaybftRBUSErhqYiQzvEmuB"
  );

  it("Create!", async () => {
    const computeUnitIx = ComputeBudgetProgram.setComputeUnitLimit({
      units: 500_000,
    });

    const tx = await program.methods
      .create()
      .accounts({
        platformConfig,
        baseTokenMint,
        quoteTokenMint,
      })
      .preInstructions([computeUnitIx])
      .signers([baseTokenKeypair])
      .rpc();

    console.log(`https://explorer.solana.com/tx/${tx}?cluster=custom`);
  });

  it("Buy!", async () => {
    const amountIn = new BN(100_000_000);
    const minimumAmountOut = new BN(0);
    const shareFeeRate = new BN(0);

    const [platformFeeVault] = PublicKey.findProgramAddressSync(
      [platformConfig.toBuffer(), quoteTokenMint.toBuffer()],
      raydiumProgram.programId
    );

    const creator = program.provider.wallet.publicKey;

    const [creatorFeeVault] = PublicKey.findProgramAddressSync(
      [creator.toBuffer(), quoteTokenMint.toBuffer()],
      raydiumProgram.programId
    );

    const createBaseAta =
      createAssociatedTokenAccountIdempotentInstructionWithDerivation(
        creator,
        creator,
        baseTokenMint
      );

    const remainingAccounts = [
      {
        pubkey: SystemProgram.programId,
        isWritable: false,
        isSigner: false,
      },
      {
        pubkey: platformFeeVault,
        isWritable: true,
        isSigner: false,
      },
      {
        pubkey: creatorFeeVault,
        isWritable: true,
        isSigner: false,
      },
    ];

    const tx = await program.methods
      .buy(amountIn, minimumAmountOut, shareFeeRate)
      .accounts({
        platformConfig,
        baseTokenMint,
        quoteTokenMint,
      })
      .preInstructions([createBaseAta])
      .remainingAccounts(remainingAccounts)
      .rpc();

    console.log(`https://explorer.solana.com/tx/${tx}?cluster=custom`);
  });
});
