import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { LaunchlabsBuy } from "../target/types/launchlabs_buy";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { BN } from "bn.js";
import { NATIVE_MINT } from "@solana/spl-token";

describe("launchlabs-buy", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.launchlabsBuy as Program<LaunchlabsBuy>;

  it("Buy!", async () => {
    const amountIn = new BN(100_000_000);
    const minimumAmountOut = new BN(0);
    const shareFeeRate = new BN(0);

    const platformConfig = new PublicKey(
      "BuM6KDpWiTcxvrpXywWFiw45R2RNH8WURdvqoTDV1BW4"
    );
    const baseTokenMint = new PublicKey(
      "Fr7Bx4jbcHm9B4rRvWjAasmaWEZgr2MQCXh6X3Dmbonk"
    );
    const quoteTokenMint = NATIVE_MINT;

    // let platform_fee_vault = Pubkey::find_program_address(
    //     &[platform_config.key().as_ref(), quote_token_mint.key().as_ref()],
    //     &launch_program_id,
    // ).0;
    const platformFeeVault = new PublicKey(
      "84FqPoha4BJCn4LrXtzFQ73ZEHkbFbXparZMPg7wdDzS"
    );
    // let creator_fee_vault = Pubkey::find_program_address(
    //         &[pool_creator.as_ref(), quote_token_mint.key().as_ref()],
    //         &launch_program_id,
    //     ).0;
    const creatorFeeVault = new PublicKey(
      "DJaA1XD9rGudGrQcF4avMWrEGRSorunSgiJWTvUBHugj"
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
      .remainingAccounts(remainingAccounts)
      .rpc();

    console.log(`https://explorer.solana.com/tx/${tx}?cluster=custom`);
  });
});
