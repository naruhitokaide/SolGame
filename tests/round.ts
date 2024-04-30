import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Round } from "../target/types/round";
import { SystemProgram, Keypair, PublicKey, Transaction, SYSVAR_RENT_PUBKEY, SYSVAR_CLOCK_PUBKEY } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createAccount, createAssociatedTokenAccount, getAssociatedTokenAddress, ASSOCIATED_TOKEN_PROGRAM_ID, createMint, mintTo, mintToChecked, getAccount, getMint, getAssociatedTokenAddressSync } from "@solana/spl-token";

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

  it("GET PDA", async () => {
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
    const fee = 25;// (2.5%)
    const tx = await program.rpc.initialize(
      new anchor.BN(slotTokenPrice),
      new anchor.BN(fee),
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

  it("update fee", async() => {
     const fee = 26;// (2.6%)
    const tx = await program.rpc.updateFee(
      new anchor.BN(fee),
      {
        accounts: {
          owner: owner.publicKey,
          globalState,
          systemProgram: SystemProgram.programId
        },
        signers: [owner]
      }
    );
  })
 
  it("create round 1", async () => {
    // Round 1
    const [round, bump] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(ROUND_SEED),
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
          systemProgram: SystemProgram.programId
        },
        signers: [owner]
      }
    );
    const roundData = await program.account.roundState.fetch(round);
    console.log("roundData->", roundData);
  }); 

  it("buy 1 slot in round 1 and it is finish and create round 2", async () => {
    roundIndex = 1;

    const [round, bump1] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(ROUND_SEED),
      ],
      program.programId
    );

    const [userInfo, bump4] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(ROUN_USER_INFO_SEED),
        user.publicKey.toBuffer()
      ],
      program.programId
    );

    const amount = 1;
    const globalStateData = await program.account.globalState.fetch(globalState);
    console.log(globalStateData);

    try {
      const tx = await program.rpc.buySlot(
        roundIndex,
        new anchor.BN(amount),
        {
          accounts: {
            user: user.publicKey,
            owner: new PublicKey(globalStateData.owner),
            globalState,
            round,
            vault,
            userInfo,
            systemProgram: SystemProgram.programId,
          },
          signers: [user]
        }
      );
      const roundData = await program.account.roundState.fetch(round);
      console.log("roundData->", roundData);
    } catch (error) {
      console.log(error);
    }
  });

  it("buy 2 slot in round 2 and it is finish", async () => {
    roundIndex = 2;
    const [round, bump1] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(ROUND_SEED),
      ],
      program.programId
    );

    const amount = 2;

    const [userInfo, bump4] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(ROUN_USER_INFO_SEED),
        user.publicKey.toBuffer()
      ],
      program.programId
    );
    const globalStateData = await program.account.globalState.fetch(globalState);

    try {
      const tx = await program.rpc.buySlot(
        roundIndex,
        new anchor.BN(amount),
        {
          accounts: {
            user: user.publicKey,
            owner: new PublicKey(globalStateData.owner),
            globalState,
            round,
            vault,
            userInfo,
            systemProgram: SystemProgram.programId,
          },
          signers: [user]
        }
      );
      const roundData = await program.account.roundState.fetch(round);
      console.log("roundData->", roundData);
    } catch (error) {
      console.log(error);
    }


  }); 
  it("claim slot in round 1", async () => {
    roundIndex = 1;

    const [round, bump1] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(ROUND_SEED),
      ],
      program.programId
    );

    const [userInfo, bump4] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(ROUN_USER_INFO_SEED),
        user.publicKey.toBuffer()
      ],
      program.programId
    );

    try {
      const balance = await program.provider.connection.getBalance(vault);
      const lamportBalance = (balance / 1000000000);
      console.log("lamportBalance after claim->", lamportBalance);
      console.log(vault.toString());
      const [userInfo, bump4] = await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from(ROUN_USER_INFO_SEED),
          user.publicKey.toBuffer()
        ],
        program.programId
      );
      const userInfoData = await program.account.userInfo.fetch(userInfo);
      console.log("roundData->", userInfoData);
      const tx = await program.rpc.claimSlot(
        {
          accounts: {
            user: user.publicKey,
            globalState,
            vault,
            userInfo,
            systemProgram: SystemProgram.programId
          },
          signers: [user]
        }
      );
      const roundData = await program.account.roundState.fetch(round);
      console.log("roundData->", roundData);
    } catch (error) {
      console.log(error);
    }

  });

  it("withdraw sol", async () => {
    let balance = await program.provider.connection.getBalance(vault);
    let lamportBalance = (balance / 1000000000);
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
    lamportBalance = (balance / 1000000000);
    console.log("lamportBalance after withdraw->", lamportBalance);
  });
});
