import { DEV_PHRASE } from "@polkadot/keyring";
import { u8aToHex, u8aToU8a } from "@polkadot/util";
import {
  cryptoWaitReady,
  keyExtractSuri,
  keyFromPath,
  mnemonicToMiniSecret,
  secp256k1PairFromSeed,
} from "@polkadot/util-crypto";
import { secp256k1Sign } from "@polkadot/wasm-crypto";
// (when using the API and waiting on `isReady` this is done automatically)
await cryptoWaitReady();
const suri = `${DEV_PHRASE}//Alice`;
const { derivePath, password, path, phrase } = keyExtractSuri(suri);
// create a keyring with some non-default values specified
const seed = mnemonicToMiniSecret(phrase, password);
const keypair = secp256k1PairFromSeed(seed);

const alice = keyFromPath(keypair, path, "ecdsa");
const ALICE_PUBLIC_KEY_HEX = u8aToHex(alice.publicKey, undefined, false);
const SERVICE_ID = 0;

type IKeyType = "Sr25519" | "Ecdsa";
interface ChallengeRequest {
  pub_key: string;
  key_type: IKeyType;
}

interface ChallengeResponse {
  challenge: string;
  expires_at: number;
}

const requestBody = {
  pub_key: ALICE_PUBLIC_KEY_HEX,
  key_type: "Ecdsa",
} satisfies ChallengeRequest;

const challenge = await fetch("http://localhost:8276/v1/auth/challenge", {
  method: "POST",
  headers: {
    "Content-Type": "application/json",
    "X-Service-Id": SERVICE_ID.toString(),
  },
  body: JSON.stringify(requestBody),
});

if (!challenge.ok) {
  console.dir(requestBody, { depth: null });
  const errorText = await challenge.text();
  throw new Error(
    `Failed to get challenge: ${challenge.statusText} - ${errorText}`
  );
}
const challengeResponse = (await challenge.json()) as ChallengeResponse;
const { challenge: challengeString } = challengeResponse;
const challengeBytes = u8aToU8a(`0x${challengeString}`);
const signature = secp256k1Sign(challengeBytes, alice.secretKey);
const signatureHex = u8aToHex(signature, undefined, false);

interface VerifyChallengeRequest {
  pub_key: string;
  key_type: IKeyType;
  challenge: string;
  signature: string;
  expires_at: number;
}

const verifyRequestBody = {
  pub_key: ALICE_PUBLIC_KEY_HEX,
  key_type: "Ecdsa",
  challenge: challengeString,
  signature: signatureHex,
  expires_at: 0,
} satisfies VerifyChallengeRequest;

const verifyChallenge = await fetch("http://localhost:8276/v1/auth/verify", {
  method: "POST",
  headers: {
    "Content-Type": "application/json",
    "X-Service-Id": SERVICE_ID.toString(),
  },
  body: JSON.stringify(verifyRequestBody),
});

if (!verifyChallenge.ok) {
  const errorText = await verifyChallenge.text();
  console.dir(verifyRequestBody, { depth: null });
  throw new Error(
    `Failed to verify challenge: ${verifyChallenge.statusText} - ${errorText}`
  );
}

const verifyChallengeResponse = await verifyChallenge.json();
console.dir(verifyChallengeResponse, { depth: null });

export {};
