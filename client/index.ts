import {
  Connection,
  Keypair,
  PublicKey,
  sendAndConfirmTransaction,
  SystemProgram,
  Transaction,
  TransactionInstruction,
} from "@solana/web3.js";

async function main() {
  const connection = new Connection("http://127.0.0.1:8899", "confirmed");
  const keypairBytes = Uint8Array.from([
    197, 113, 82, 70, 190, 61, 76, 40, 187, 109, 157, 99, 132, 118, 79, 120, 48,
    81, 192, 136, 197, 113, 119, 110, 18, 175, 44, 12, 242, 206, 148, 95, 49,
    130, 159, 1, 207, 216, 230, 195, 14, 253, 144, 28, 253, 96, 191, 221, 200,
    106, 242, 193, 23, 209, 120, 139, 47, 34, 192, 222, 54, 247, 171, 253,
  ]);

  let program_id = new PublicKey(
    "8hxkpkbhe7jY92ZGxe6nXWZhBLHa3dJxGpDYgh3FMJt5"
  );

  const user = Keypair.fromSecretKey(keypairBytes);

  let seed1 = Buffer.from("vault");
  let seed2 = user.publicKey.toBuffer();

  const [pda_address, bump] = PublicKey.findProgramAddressSync(
    [seed1, seed2],
    program_id
  );

  const data = Buffer.concat([
    Buffer.from([seed1.length]),
    seed1,
    Buffer.from([seed2.length]),
    seed2,
    Buffer.from([bump]),
  ]);

  const ix = new TransactionInstruction({
    programId: program_id,
    keys: [
      { pubkey: pda_address, isSigner: false, isWritable: true },
      { pubkey: user.publicKey, isSigner: true, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data,
  });

  const tx = new Transaction().add(ix);

  const sig = await sendAndConfirmTransaction(connection, tx, [user]);
  console.log("TX:", sig);
}


main()