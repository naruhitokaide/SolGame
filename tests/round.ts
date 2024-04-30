import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Round } from "../target/types/round";
import { SystemProgram, Keypair, PublicKey, Transaction, SYSVAR_RENT_PUBKEY, SYSVAR_CLOCK_PUBKEY } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createAccount, createAssociatedTokenAccount, getAssociatedTokenAddress , ASSOCIATED_TOKEN_PROGRAM_ID,createMint, mintTo, mintToChecked, getAccount, getMint, getAssociatedTokenAddressSync } from "@solana/spl-token";

describe("round", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Round as Program<Round>;

  let owner = Keypair.fromSecretKey(
    Uint8Array.from([/* private key as uint8array*/])
  );

  let user = Keypair.fromSecretKey(
    Uint8Array.from([/* private key as uint8array*/])  );

  

  const GLOBAL_STATE_SEED = "GLOBAL-STATE-SEED";
  const VAULT_SEED = "VAULT-SEED";
  const ROUND_SEED = "ROUND-SEED";
  const ROUN_USER_INFO_SEED = "ROUND-USER-INFO-SEED";

  let globalState, vault: PublicKey;
  let globalStateBump, vaultBump: number;
  let roundIndex = 1;

  it("GET PDA", async() => {
    [globalState, globalStateBump] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(GLOBAL_STATE_SEED)
      ],
      program.programId
    );

    [vault, vaultBump] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(VAULT_SEED)
      ],
      program.programId
    );

  });

  it("Is initialized!", async () => {
    // Add your test here.
    const slotTokenPrice = 100000000; // 0.1SOL
    const tx = await program.rpc.initialize(
      new anchor.BN(slotTokenPrice),
      {
        accounts: {
          owner: owner.publicKey,
          globalState,
          vault,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId
        },
        signers: [owner]
      }
    );
    const globalStateData = await program.account.globalState.fetch(globalState);
    console.log(globalStateData);
  });

 
  it("create round 1", async() => {
    // Round 1
    const [round, bump] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(ROUND_SEED),
        new anchor.BN(roundIndex).toBuffer('le', 4)
      ],
      program.programId
    );
    const tx = await program.rpc.createRound(
      roundIndex,
      {
        accounts: {
          owner: owner.publicKey,
          globalState,
          round,
          systemProgram:SystemProgram.programId
        },
        signers: [owner]
      }
    );
    const roundData = await program.account.round.fetch(round);
    console.log("roundData->", roundData);
  });
 
  it("buy 1 slot in round 1 and it is finish", async() => {
    roundIndex = 1;
    const [round, bump1] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(ROUND_SEED),
        new anchor.BN(roundIndex).toBuffer('le', 4)
      ],
      program.programId
    );

    const [roundUserInfo, bump2] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(ROUN_USER_INFO_SEED),
        new anchor.BN(roundIndex).toBuffer('le', 4),
        user.publicKey.toBuffer()
      ],
      program.programId
    );

    const amount = 1;

    const tx = await program.rpc.buySlot(
      roundIndex,
      new anchor.BN(amount),
      {
        accounts: {
          user: user.publicKey,
          globalState,
          round,
          vault,
          roundUserInfo,
          systemProgram: SystemProgram.programId
        },
        signers: [user]
      }
    );
    const roundData = await program.account.round.fetch(round);
    console.log("roundData->", roundData);

    const roundUserInfoData = await program.account.roundUserInfo.fetch(roundUserInfo);
    console.log("roundData->", roundUserInfoData);

  });

  it("create round 2", async() => {
    // Round 2
    roundIndex = 2;
    const [round, bump] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(ROUND_SEED),
        new anchor.BN(roundIndex).toBuffer('le', 4)
      ],
      program.programId
    );
    const tx = await program.rpc.createRound(
      roundIndex,
      {
        accounts: {
          owner: owner.publicKey,
          globalState,
          round,
          systemProgram:SystemProgram.programId
        },
        signers: [owner]
      }
    );
    const roundData = await program.account.round.fetch(round);
    console.log("roundData->", roundData);
  });
 
  it("buy 2 slot in round 2 and it is finish", async() => {
    roundIndex = 2;
    const [round, bump1] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(ROUND_SEED),
        new anchor.BN(roundIndex).toBuffer('le', 4)
      ],
      program.programId
    );

    const [roundUserInfo, bump2] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(ROUN_USER_INFO_SEED),
        new anchor.BN(roundIndex).toBuffer('le', 4),
        user.publicKey.toBuffer()
      ],
      program.programId
    );

    const amount = 2;

    const tx = await program.rpc.buySlot(
      roundIndex,
      new anchor.BN(amount),
      {
        accounts: {
          user: user.publicKey,
          globalState,
          round,
          vault,
          roundUserInfo,
          systemProgram: SystemProgram.programId
        },
        signers: [user]
      }
    );
    const roundData = await program.account.round.fetch(round);
    console.log("roundData->", roundData);

    const roundUserInfoData = await program.account.roundUserInfo.fetch(roundUserInfo);
    console.log("roundData->", roundUserInfoData);

  });

  it("claim slot in round 1", async() => {
    roundIndex = 1;
    const nextRoundIndex = roundIndex + 1;

    const [round, bump1] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(ROUND_SEED),
        new anchor.BN(roundIndex).toBuffer('le', 4)
      ],
      program.programId
    );

    const [nextRound, bump2] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(ROUND_SEED),
        new anchor.BN(nextRoundIndex).toBuffer('le', 4)
      ],
      program.programId
    );

    const [roundUserInfo, bump3] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(ROUN_USER_INFO_SEED),
        new anchor.BN(roundIndex).toBuffer('le', 4),
        user.publicKey.toBuffer()
      ],
      program.programId
    );

    
    const tx = await program.rpc.claimSlot(
      roundIndex,
      {
        accounts: {
          user: user.publicKey,
          globalState,
          nextRound,
          round,
          vault,
          roundUserInfo,
          systemProgram: SystemProgram.programId
        },
        signers: [user]
      }
    );
    const roundData = await program.account.round.fetch(round);
    console.log("roundData->", roundData);
  
    const roundUserInfoData = await program.account.roundUserInfo.fetch(roundUserInfo);
    console.log("roundData->", roundUserInfoData);
  
  });


  it("withdraw sol", async() => {
    let balance = await program.provider.connection.getBalance(vault);
    let lamportBalance=(balance / 1000000000);
    console.log("lamportBalance before withdraw->", lamportBalance);

    const tx = await program.rpc.withdrawSol(
      new anchor.BN(balance),
      {
        accounts: {
          owner: owner.publicKey,
          globalState,
          vault,
          systemProgram: SystemProgram.programId
        },
        signers: [owner]
      }
    );
    balance = await program.provider.connection.getBalance(vault);
    lamportBalance=(balance / 1000000000);
    console.log("lamportBalance after withdraw->", lamportBalance);
  });
});
